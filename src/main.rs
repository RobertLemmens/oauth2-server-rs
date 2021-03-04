mod models;
mod db;

use warp::{Filter, Rejection, Reply, reply::json, hyper::StatusCode, reject, reject::Reject};
use warp::http::Response;
use rand::Rng;
use rand::distributions::Alphanumeric;
use crate::models::{TokenParams, Config, User};
use dotenv::dotenv;
use std::{net::{Ipv4Addr, SocketAddrV4}, str::from_utf8};
use tokio_postgres::NoTls;
use deadpool_postgres::{Pool, Client};
use std::convert::Infallible;
use std::str;

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

async fn get_access_token(params: Option<TokenParams>, client_authorization: String, db_pool: deadpool_postgres::Pool) -> std::result::Result<impl Reply, Rejection> {
    let client: Client = db_pool.get().await.expect("Error connecting to database");

    let mut validation = 0;
    let mut client_db_id = 0;

    match params {
        Some(obj) => {
            match obj.grant_type.as_str() {
                "password" => {
                    if obj.username.is_some() && obj.password.is_some() {
                        if client_authorization.len() > 0 {
                            let f1: Vec<&str> = client_authorization.split(" ").collect();
                            let split = base64::decode(f1[1]).unwrap();
                            let res: Vec<&str> = str::from_utf8(&split).unwrap().split(":").collect();
                            client_db_id = db::validate_client_credentials(&client, res[0].to_string(), res[1].to_string()).await;
                            println!("Client id is {}", client_db_id);
                            validation = db::validate_password_credentials(&client, obj.username.unwrap(), obj.password.unwrap()).await;
                        }
                    }
                },
                "client_credentials" => {
                    if obj.client_id.is_some() && obj.client_secret.is_some() {
                        validation = db::validate_client_credentials(&client, obj.client_id.unwrap(), obj.client_secret.unwrap()).await;
                    }
                },
                _ => {

                }
            }
        }
        None => {
        }
    }

    if client_db_id == 0 {
        return Err(warp::reject::not_found())
    }

    if validation > 0 {
        let token = generate_token();
        let res = db::insert_token(&client, token.clone(), validation, client_db_id).await;
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

// validate client auth header
fn with_auth() -> impl Filter<Extract = (String,), Error = Rejection> + Clone {
    warp::header::<String>("Authorization").map(|token: String|{
        token 
    })
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

    let auth = warp::header::<String>("Authorization").or(warp::any().map(|| String::new())).unify();

    let opt_query = warp::query::<TokenParams>()
        .map(Some)
        .or_else(|_| async { Ok::<(Option<TokenParams>,), std::convert::Infallible>((None,)) });

    let introspect_route = warp::post().and(warp::path("oauth2")).and(warp::path("introspect"));

    let logout_route = warp::post().and(warp::path("oauth2")).and(warp::path("logout"));

    let token_route = warp::post()
        .and(warp::path("oauth2"))
        .and(warp::path("token"))
        .and(opt_query)
        .and(auth)
        .and(with_db(pool.clone()))
        .and_then(get_access_token)
        .recover(custom_errors);

    let crud_route = warp::path("users").and(with_db(pool.clone())).and_then(get_users);

    let routes = warp::post().and(token_route.or(crud_route));

    // TODO regel een from_string voor het adres
    let adrr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), config.server.port);

    warp::serve(routes).run(adrr).await

}
