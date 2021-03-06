mod models;
mod db;

use models::ServerConfig;
use warp::{Filter, Rejection, Reply, hyper::StatusCode, post, reject, reject::Reject, reply::json};
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
use std::collections::HashMap;

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

fn decode_client_auth(client_authorization: String) -> Vec<String> {
    let f1: Vec<&str> = client_authorization.split(" ").collect();
    let split = base64::decode(f1[1]).unwrap();
    str::from_utf8(&split).unwrap().split(":").map(|c: &str| c.to_string()).collect()
}

// Introspect a token
async fn introspect_token(client_authorization: String, access_token: String, db_pool: deadpool_postgres::Pool) -> std::result::Result<impl Reply, Rejection> {
    let client: Client = db_pool.get().await.expect("Error connecting to database");
    let client_credentials = decode_client_auth(client_authorization);
    let client_db_id = db::validate_client_credentials(&client, client_credentials[0].to_string(), client_credentials[1].to_string()).await;
    if client_db_id == 0 {
        return Err(warp::reject::not_found())
    }
    let result = db::validate_access_token(&client, access_token, client_credentials[0].clone());

    Ok(json(&result.await))
}

// Request an access token
async fn get_access_token(params: Option<TokenParams>, client_authorization: String, db_pool: deadpool_postgres::Pool, server_config: ServerConfig) -> std::result::Result<impl Reply, Rejection> {
    let client: Client = db_pool.get().await.expect("Error connecting to database");

    if client_authorization.is_empty() {
        return  Err(warp::reject::not_found())
    }

    match params {
        Some(obj) => {
            match obj.grant_type.as_str() {
                "password" => {
                    if obj.username.is_some() && obj.password.is_some() {
                        let client_credentials = decode_client_auth(client_authorization);
                        let client_db_id = db::validate_client_credentials(&client, client_credentials[0].to_string(), client_credentials[1].to_string()).await;
                        let validation = db::validate_password_credentials(&client, obj.username.unwrap(), obj.password.unwrap()).await;
                        if validation > 0 && client_db_id > 0{
                            let token = generate_token();
                            let res = db::insert_token(&client, token.clone(), obj.scope, Some(validation), client_db_id, server_config.name).await;
                            return Ok(json(&res))
                        } else {
                            return Err(warp::reject::not_found())
                        }
                    }
                },
                "client_credentials" => {
                    let client_credentials = decode_client_auth(client_authorization);
                    let client_db_id = db::validate_client_credentials(&client, client_credentials[0].to_string(), client_credentials[1].to_string()).await;
                    if client_db_id > 0 {
                        let token = generate_token();
                        let res = db::insert_token(&client, token.clone(), obj.scope, None, client_db_id, server_config.name).await;
                        return Ok(json(&res))
                    } else {
                        return Err(warp::reject::not_found())
                    }
                },
                _ => {

                }
            }
        }
        None => {
        }
    }

    return Err(warp::reject::not_found())
}

async fn create_user(db_pool: deadpool_postgres::Pool) -> std::result::Result<impl Reply, Rejection> {

    Ok("")
}

fn with_db(db_pool: deadpool_postgres::Pool) -> impl Filter<Extract = (deadpool_postgres::Pool,), Error = Infallible> + Clone {
    warp::any().map(move || db_pool.clone())
}

fn with_config(config: Config) -> impl Filter<Extract = (crate::models::ServerConfig,), Error = Infallible> + Clone {
    warp::any().map(move || config.server.clone())
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

    // TODO betere manier om te falen
    let introspect_body = warp::body::form()
        .map(|form: HashMap<String, String>| form.get("token").unwrap().to_string());

    let token_body = warp::body::form().map(|form: TokenParams| Some(form));

    let opt_query = warp::query::<TokenParams>()
        .map(Some)
        .or_else(|_| async { Ok::<(Option<TokenParams>,), std::convert::Infallible>((None,)) });

    let oauth_route = warp::post().and(warp::path("oauth2"));

    let introspect_route = oauth_route 
        .and(warp::path("introspect"))
        .and(auth)
        .and(introspect_body)
        .and(with_db(pool.clone()))
        .and_then(introspect_token);

    let logout_route = warp::post().and(warp::path("oauth2")).and(warp::path("logout"));

    let token_route = oauth_route
        .and(warp::path("token"))
        .and(token_body)
        .and(auth)
        .and(with_db(pool.clone()))
        .and(with_config(config.clone()))
        .and_then(get_access_token)
        .recover(custom_errors);

    let routes = warp::post().and(introspect_route.or(token_route));

    // TODO regel een from_string voor het adres
    let adrr = SocketAddrV4::new(Ipv4Addr::new(127, 0, 0, 1), config.server.port);

    warp::serve(routes).run(adrr).await

}
