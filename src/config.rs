use secrecy::{ExposeSecret, SecretString};

#[derive(serde::Deserialize)]
pub struct Settings {
    pub service: Service,
    pub database: Database,
}

#[derive(serde::Deserialize)]
pub struct Service {
    pub host: String,
}

#[derive(serde::Deserialize)]
pub struct Database {
    pub username: String,
    pub password: SecretString,
    pub host: String,
    pub port: String,
    pub database_name: String,
}

pub fn get_config() -> Result<Settings, config::ConfigError> {
    let settings = config::Config::builder()
        .add_source(config::File::new("config.toml", config::FileFormat::Toml))
        .build()?;

    settings.try_deserialize::<Settings>()
}

impl Database {
    pub fn conn_str(&self) -> SecretString {
        SecretString::new(
            format!(
                "postgres://{}:{}@{}:{}/{}",
                self.username,
                self.password.expose_secret(),
                self.host,
                self.port,
                self.database_name
            )
            .into(),
        )
    }

    pub fn conn_str_without_db(&self) -> SecretString {
        format!(
            "postgres://{}:{}@{}:{}",
            self.username,
            self.password.expose_secret(),
            self.host,
            self.port
        )
        .into()
    }
}
