use config::ConfigError;
use serde::{Deserialize, Serialize};
use tokio_pg_mapper_derive::PostgresMapper;

#[derive(Deserialize, Serialize)]
pub struct TokenParams {
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub pcke: Option<String>,
    pub device: Option<String>,
    pub grant_type: String,
    pub redirect_uri: Option<String>,
    pub scope: Option<String>,
    pub code: Option<String>,
    pub refresh_token: Option<String>,
}

#[derive(Deserialize)]
pub struct AuthorizationParams {
    pub client_id: String,
    pub response_type: String,
    pub redirect_uri: String,
    pub scope: String,
    pub state: Option<String>,
}

#[derive(Serialize, Deserialize, PostgresMapper)]
#[pg_mapper(table = "users")]
pub struct User {
    pub id: i32,
    pub username: Option<String>,
    pub password: Option<String>,
}

#[derive(Serialize, Deserialize, PostgresMapper)]
#[pg_mapper(table = "access_tokens")]
pub struct AccessToken {
    pub access_token: String,
    pub token_type: String,
    pub scope: Option<String>,
    pub expires_in: i64,
}

#[derive(Serialize, Deserialize)]
pub struct Introspection {
    pub active: bool,
    pub client_id: String,
    pub username: Option<String>,
    pub user_id: Option<i32>,
    pub scope: Option<String>,
    pub token_type: String,
    pub issuer: String,
    pub exp: i64,
    pub iat: i64,
}

#[derive(Deserialize, Debug, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub name: String,
}

#[derive(Deserialize, Debug, Clone)]
pub struct Config {
    pub server: ServerConfig,
    pub bootstrap: bool,
    pub pg: deadpool_postgres::Config,
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let mut cfg = config::Config::new();
        cfg.merge(config::Environment::new())?;
        cfg.try_into() // probeer te deserializen in het geselecteerde object
    }
}
