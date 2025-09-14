use std::net::TcpListener;

#[tokio::test]
async fn healthz_works() {
    let host = spawn_app();
    let client = reqwest::Client::new();

    let resp = client
        .get(&format!("{}/healthz", &host))
        .send()
        .await
        .expect("failed to execute request");

    assert!(resp.status().is_success());
    assert_eq!(Some(0), resp.content_length());
}

fn spawn_app() -> String {
    let listener = TcpListener::bind("127.0.0.1:0").expect("failed to bind random port");
    let port = listener.local_addr().unwrap().port();

    let srv = zero2prod::run(listener).expect("failed to bind address");
    tokio::spawn(srv);

    format!("http://127.0.0.1:{}", port)
}
