mod db;
mod errors;
mod handlers;
mod models;
mod response;

use crate::db::*;
use crate::models::{AuthorizationParams, Config, TokenParams};
use deadpool_postgres::{Client, PoolError};
use dotenv::dotenv;
use std::collections::HashMap;
use std::convert::Infallible;
use std::{fs, io};
use std::net::{Ipv4Addr, SocketAddrV4};
use tokio_postgres::NoTls;
use native_tls::{Certificate, TlsConnector};
use postgres_native_tls::MakeTlsConnector;
use warp::http::Response;
use warp::{hyper::StatusCode, Filter, Rejection, Reply};
use thiserror::Error;


fn with_db(
    db_pool: deadpool_postgres::Pool,
) -> impl Filter<Extract = (deadpool_postgres::Pool,), Error = Infallible> + Clone {
    warp::any().map(move || db_pool.clone())
}

fn with_config(
    config: Config,
) -> impl Filter<Extract = (crate::models::ServerConfig,), Error = Infallible> + Clone {
    warp::any().map(move || config.server.clone())
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("could not read pem file for tls")]
    Read(#[from] io::Error),
    #[error("tls error")]
    Tls(#[from] native_tls::Error),
    #[error("unknown config error")]
    Unknown,
}

fn setup_tls() -> Result<MakeTlsConnector, ConfigError> {
    let cert = fs::read("root.pem")?;
    let cert = Certificate::from_pem(&cert)?;
    let connector = TlsConnector::builder()
        .add_root_certificate(cert)
        .build()?;
    let connector = MakeTlsConnector::new(connector);
    return Ok(connector);
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let config: Config = crate::models::Config::from_env().unwrap();

    let tls = setup_tls();
    let pool = match tls {
        Ok(res) => config.pg.create_pool(res).unwrap(),
        Err(msg) =>  { 
            println!("Error setting up TLS, continuing without. Message: {:?}", msg);
            config.pg.create_pool(NoTls).unwrap() 
        },
    };

    if config.bootstrap {
        println!("Bootstrapping postgresql database...");
        match pool.get().await {
            Ok(client) => {
                let file = fs::read_to_string("database_init.sql").expect("Could not load bootstrap script. The bootstrap script should be in the root of the run context");
                db::create_tables(&client, file.as_str()).await;
            }
            Err(PoolError::Backend(e)) => {
                println!("Backend Error {}", e);
            }
            Err(_) => {
                println!("Error - unknown");
            }

        }
    }



    println!(
        "Starting oauth server on http://{}:{}/",
        config.server.host, config.server.port
    );

    let auth = warp::header::<String>("Authorization")
        .or(warp::any().map(|| String::new()))
        .unify();

    let introspect_body = warp::body::form()
        .map(|form: HashMap<String, String>| form.get("token").unwrap().to_string());

    let token_body = warp::body::form().map(|form: TokenParams| Some(form));

    let authorization_params = warp::query().map(|params: AuthorizationParams| {
        println!("Mappiong params");
        params
    });

    let oauth_route = warp::post().and(warp::path("oauth2"));
    let oauth_get_route = warp::get().and(warp::path("oauth2"));

    let introspect_route = oauth_route
        .and(warp::path("introspect"))
        .and(warp::path::end())
        .and(auth)
        .and(introspect_body)
        .and(with_db(pool.clone()))
        .and_then(handlers::introspect_token);

    let logout_route = oauth_route
        .and(warp::path("logout"))
        .and(warp::path::end())
        .and(with_db(pool.clone()))
        .and_then(handlers::invalidate_token);

    let authorize_route = oauth_get_route
        .and(warp::path("authorize"))
        .and(warp::path::end())
        .and(authorization_params)
        .and(with_db(pool.clone()))
        .and_then(handlers::get_authorization);

    let token_route = oauth_route
        .and(warp::path("token"))
        .and(warp::path::end())
        .and(token_body)
        .and(auth)
        .and(with_db(pool.clone()))
        .and(with_config(config.clone()))
        .and_then(handlers::get_access_token);

    let health_route = warp::get()
        .and(warp::path("q"))
        .and(warp::path("health"))
        .and(warp::path::end())
        .and_then(handlers::get_health)
        .recover(errors::handle_get_notallowed);

    let routes = authorize_route
        .or(health_route)
        .or(introspect_route)
        .or(token_route)
        .or(logout_route)
        .recover(errors::handle_rejection);

    // TODO regel een from_string voor het adres
    let adrr = SocketAddrV4::new(Ipv4Addr::new(0, 0, 0, 0), config.server.port);

    warp::serve(routes).run(adrr).await
}
