use serde::{Deserialize, Serialize};
use config::ConfigError;
use tokio_pg_mapper_derive::PostgresMapper;

#[derive(Deserialize, Serialize)]
pub struct TokenParams {
    pub client_id: Option<String>,
    pub client_secret: Option<String>,
    pub username: Option<String>,
    pub password: Option<String>,
    pub grant_type: String,
    pub redirect_uri: Option<String>,
    pub scope: Option<String>,
    pub refresh_token: Option<String>,
}

#[derive(Serialize, Deserialize, PostgresMapper)]
#[pg_mapper(table="users")]
pub struct User {
    pub id: i32,
    pub username: Option<String>,
    pub password: Option<String>
}

#[derive(Serialize, Deserialize, PostgresMapper)]
#[pg_mapper(table="access_tokens")]
pub struct AccessToken {
    pub access_token: String,
    pub token_type: String,
    pub expire_time: String,
}

#[derive(Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16
}

#[derive(Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub pg: deadpool_postgres::Config
}

impl Config {
    pub fn from_env() -> Result<Self, ConfigError> {
        let mut cfg = config::Config::new();
        cfg.merge(config::Environment::new())?;
        cfg.try_into() // probeer te deserializen in het geselecteerde object
    }

}
