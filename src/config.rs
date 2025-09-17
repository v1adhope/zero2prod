use secrecy::{ExposeSecret, SecretString};
use serde_aux::prelude::deserialize_number_from_string;
use sqlx::postgres::PgConnectOptions;

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
    #[serde(deserialize_with = "deserialize_number_from_string")]
    pub port: u16,
    pub database_name: String,
}

impl Database {
    pub fn with_db(&self) -> PgConnectOptions {
        self.without_db().database(&self.database_name)
    }

    pub fn without_db(&self) -> PgConnectOptions {
        PgConnectOptions::new()
            .username(&self.username)
            .password(self.password.expose_secret())
            .host(&self.host)
            .port(self.port)
    }
}

pub enum Environment {
    Local,
    Prod,
}

impl Environment {
    pub fn as_str(&self) -> &'static str {
        match self {
            Environment::Local => "local",
            Environment::Prod => "prod",
        }
    }
}

impl TryFrom<String> for Environment {
    type Error = String;

    fn try_from(s: String) -> Result<Self, Self::Error> {
        match s.to_lowercase().as_str() {
            "local" => Ok(Self::Local),
            "prod" => Ok(Self::Prod),
            other => Err(format!("{} is not supported environment", other)),
        }
    }
}

pub fn get_config() -> Result<Settings, config::ConfigError> {
    let base_path = std::env::current_dir().expect("failed to determine the current directory");
    let configs_dir = base_path.join("configs");

    let env: Environment = std::env::var("APP_ENV")
        .unwrap_or_else(|_| "local".into())
        .try_into()
        .expect("failed to parse APP_ENV");

    let settings = config::Config::builder()
        .add_source(config::File::from(configs_dir.join("base.toml")))
        .add_source(config::File::from(
            configs_dir.join(format!("{}.toml", env.as_str())),
        ))
        .build()?;

    settings.try_deserialize::<Settings>()
}
