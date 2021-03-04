mod models;
mod db;

use db::insert_token;
use warp::{Filter, Rejection, Reply, reply::json, hyper::StatusCode, reject, reject::Reject};
use warp::http::Response;
use rand::Rng;
use rand::distributions::Alphanumeric;
use crate::models::{TokenParams, Config, User};
use dotenv::dotenv;
use std::net::{Ipv4Addr, SocketAddrV4};
use tokio_postgres::NoTls;
use deadpool_postgres::{Pool, Client};
use std::convert::Infallible;

/*
 *
 * RFC 6749 implementation of an oauth server.
 *
 * */
fn generate_token() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(128)
        .map(char::from)
        .collect()
}

async fn get_users(db_pool: deadpool_postgres::Pool) -> std::result::Result<impl Reply, Rejection> {
    let client: Client = db_pool.get().await.expect("Error connecting to database");

    // TODO pattern match?
    let result = db::get_users(&client);

    Ok(json(&result.await.map_err(|e| warp::reject::reject())?))
}

async fn get_access_token(params: Option<TokenParams>, db_pool: deadpool_postgres::Pool) -> std::result::Result<impl Reply, Rejection> {
    let client: Client = db_pool.get().await.expect("Error connecting to database");

    let mut validation = 0;

    match params {
        Some(obj) => {
            match obj.grant_type.as_str() {
                "password" => {
                    if obj.username.is_some() && obj.password.is_some() {
                        validation = db::validate_credentials(&client, obj.username.unwrap(), obj.password.unwrap()).await;
                    }
                },
                _ => {

                }
            }
        }
        None => {
        }
    }


    if validation > 0 {
        let token = generate_token();
        let res = db::insert_token(&client, token.clone(), validation).await;
        return Ok(json(&res))
    } else {
        return Err(warp::reject::not_found())
    }
}

async fn create_user(db_pool: deadpool_postgres::Pool) -> std::result::Result<impl Reply, Rejection> {

    Ok("")
}

fn with_db(db_pool: deadpool_postgres::Pool) -> impl Filter<Extract = (deadpool_postgres::Pool,), Error = Infallible> + Clone {
    warp::any().map(move || db_pool.clone())
}

async fn custom_errors(err: Rejection) -> Result<impl Reply, Rejection> {
    if err.is_not_found() {
        return Ok(Response::builder().status(StatusCode::UNAUTHORIZED).body("Invalid credentials"))
    } else {
        return Ok(Response::builder().status(StatusCode::INTERNAL_SERVER_ERROR).body("Something went wrong..."))
    }
}

#[tokio::main]
async fn main() {
    dotenv().ok();

    let config: Config = crate::models::Config::from_env().unwrap();
    let pool = config.pg.create_pool(NoTls).unwrap();

    println!("Starting oauth server on http://{}:{}/", config.server.host, config.server.port);

    let opt_query = warp::query::<TokenParams>()
        .map(Some)
        .or_else(|_| async { Ok::<(Option<TokenParams>,), std::convert::Infallible>((None,)) });

    let token_route = warp::post()
        .and(warp::path("oauth"))
        .and(warp::path("token"))
        .and(opt_query)
        .and(with_db(pool.clone()))
        .and_then(get_access_token)
        .recover(custom_errors);

    let crud_route = warp::path("users").and(with_db(pool.clone())).and_then(get_users);

    let routes = warp::post().and(token_route.or(crud_route));

    // TODO regel een from_string voor het adres
    let adrr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), config.server.port);

    warp::serve(routes).run(adrr).await

}
