mod db;
mod models;
mod handlers;
mod errors;

use crate::models::{Config, TokenParams};
use dotenv::dotenv;
use std::convert::Infallible;
use std::net::{Ipv4Addr, SocketAddrV4};
use tokio_postgres::NoTls;
use std::collections::HashMap;
use warp::http::Response;
use warp::{hyper::StatusCode, Filter, Rejection, Reply};

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

#[tokio::main]
async fn main() {
    dotenv().ok();

    let config: Config = crate::models::Config::from_env().unwrap();
    let pool = config.pg.create_pool(NoTls).unwrap();

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

    let oauth_route = warp::post().and(warp::path("oauth2"));

    let introspect_route = oauth_route
        .and(warp::path("introspect"))
        .and(auth)
        .and(introspect_body)
        .and(with_db(pool.clone()))
        .and_then(handlers::introspect_token);

    let logout_route = oauth_route
        .and(warp::path("logout"))
        .and(with_db(pool.clone()))
        .and_then(handlers::invalidate_token);

    let token_route = oauth_route
        .and(warp::path("token"))
        .and(token_body)
        .and(auth)
        .and(with_db(pool.clone()))
        .and(with_config(config.clone()))
        .and_then(handlers::get_access_token);

    let routes = warp::post().and(introspect_route.or(token_route).or(logout_route).recover(errors::handle_rejection));

    // TODO regel een from_string voor het adres
    let adrr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), config.server.port);

    warp::serve(routes).run(adrr).await
}
