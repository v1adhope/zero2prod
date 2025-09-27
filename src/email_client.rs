use reqwest::Client;
use secrecy::{ExposeSecret, SecretString};

use crate::domain::Email;

#[derive(Clone)]
pub struct EmailClient {
    http_client: Client,
    base_url: reqwest::Url,
    sender: Email,
    auth_token: SecretString,
}

impl EmailClient {
    pub fn new(
        base_url: String,
        sender: Email,
        auth_token: SecretString,
        timeout: std::time::Duration,
    ) -> Self {
        Self {
            http_client: Client::builder().timeout(timeout).build().unwrap(),
            base_url: reqwest::Url::parse(base_url.as_ref()).unwrap(),
            sender,
            auth_token,
        }
    }

    pub async fn send_email(
        &self,
        recipient: Email,
        subject: &str,
        html_content: &str,
        text_content: &str,
    ) -> Result<(), reqwest::Error> {
        let req_body = SendEmailReq {
            from: self.sender.as_ref(),
            to: recipient.as_ref(),
            subject,
            html_body: html_content,
            text_body: text_content,
        };

        self.http_client
            .post(self.base_url.join("/email").unwrap())
            .header("X-Postmark-Server-Token", self.auth_token.expose_secret())
            .json(&req_body)
            .send()
            .await?
            .error_for_status()?;

        Ok(())
    }
}

#[derive(serde::Serialize)]
#[serde(rename_all = "PascalCase")]
struct SendEmailReq<'a> {
    from: &'a str,
    to: &'a str,
    subject: &'a str,
    html_body: &'a str,
    text_body: &'a str,
}

#[cfg(test)]
mod tests {
    use crate::{domain::Email, email_client::EmailClient};
    use claims::{assert_err, assert_ok};
    use fake::{
        Fake, Faker,
        faker::{
            internet::en::SafeEmail,
            lorem::en::{Paragraph, Sentence},
        },
    };
    use reqwest::StatusCode;
    use secrecy::SecretString;
    use wiremock::{
        Mock, MockServer, Request, ResponseTemplate,
        matchers::{any, header, header_exists, method, path},
    };

    fn subject() -> String {
        Sentence(1..2).fake()
    }

    fn content() -> String {
        Paragraph(1..10).fake::<String>()
    }

    fn email() -> Email {
        Email::parse(SafeEmail().fake()).unwrap()
    }

    fn email_client(base_url: String) -> EmailClient {
        EmailClient::new(
            base_url,
            email(),
            SecretString::from(Faker.fake::<String>()),
            std::time::Duration::from_millis(200),
        )
    }

    struct SendEmailBodyMatcher;

    impl wiremock::Match for SendEmailBodyMatcher {
        fn matches(&self, req: &Request) -> bool {
            match serde_json::from_slice::<serde_json::Value>(&req.body) {
                Ok(body) => {
                    body.get("From").is_some()
                        && body.get("To").is_some()
                        && body.get("Subject").is_some()
                        && body.get("HtmlBody").is_some()
                        && body.get("TextBody").is_some()
                }
                Err(_) => false,
            }
        }
    }

    #[tokio::test]
    async fn send_email_fires_a_request_to_base_url() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(header_exists("X-Postmark-Server-Token"))
            .and(header("Content-Type", "application/json"))
            .and(path("/email"))
            .and(method("POST"))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(StatusCode::OK))
            .expect(1)
            .mount(&mock_server)
            .await;

        let sut = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        assert_ok!(sut);
    }

    #[tokio::test]
    async fn send_email_fails_if_the_server_returns_500() {
        let mock_server = MockServer::start().await;
        let email_client = email_client(mock_server.uri());

        Mock::given(any())
            .respond_with(ResponseTemplate::new(StatusCode::INTERNAL_SERVER_ERROR))
            .expect(1)
            .mount(&mock_server)
            .await;

        let sut = email_client
            .send_email(email(), &subject(), &content(), &content())
            .await;

        assert_err!(sut);
    }
}
