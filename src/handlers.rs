use crate::db;
use crate::errors::Error::*;
use crate::models::{AuthorizationParams, ServerConfig, TokenParams};
use crate::response::Response;
use deadpool_postgres::Client;
use rand::distributions::Alphanumeric;
use rand::Rng;
use std::str;
use std::str::FromStr;
use warp::http::Uri;
use warp::{reply::json, Rejection, Reply};

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
    str::from_utf8(&split)
        .unwrap()
        .split(":")
        .map(|c: &str| c.to_string())
        .collect()
}

pub async fn validate_client(client_authorization: String, client: &Client) -> i32 {
    let client_credentials = decode_client_auth(client_authorization);
    let client_db_id = db::validate_client_credentials(
        &client,
        client_credentials[0].to_string(),
        client_credentials[1].to_string(),
    )
    .await;

    client_db_id
}

// Returns the user id if valid, 0 if invalid
pub async fn validate_code(client: &Client, code: &String, pcke: &String) -> i32 {
    let user_id = db::validate_code(&client, code, pcke).await;
    user_id
}

// Introspect a token
pub async fn introspect_token(
    client_authorization: String,
    access_token: String,
    db_pool: deadpool_postgres::Pool,
) -> std::result::Result<impl Reply, Rejection> {
    let client: Client = db_pool.get().await.expect("Error connecting to database");
    let client_db_id = validate_client(client_authorization, &client).await;
    if client_db_id == 0 {
        return Err(warp::reject::custom(AuthorizationError(
            "Client credentials invalid".to_string(),
        )));
    }
    let result = db::validate_access_token(&client, access_token, client_db_id);

    match result.await {
        None => {
            return Err(warp::reject::custom(NotFoundError(
                "Unknown token".to_string(),
            )))
        }
        Some(x) => return Ok(json(&x)),
    }
}

pub async fn get_authorization(
    authorization_params: AuthorizationParams,
    db_pool: deadpool_postgres::Pool,
) -> std::result::Result<impl Reply, Rejection> {
    println!("Redirecting to login page");
    let url = format!(
        "http://localhost:8082/auth?client_id={}&response_type={}&redirect_uri={}&scope={}",
        authorization_params.client_id,
        authorization_params.response_type,
        authorization_params.redirect_uri,
        authorization_params.scope
    );
    Ok(warp::redirect(Uri::from_str(&url).unwrap()))
}

// Request an access token
pub async fn get_access_token(
    params: Option<TokenParams>,
    client_authorization: String,
    db_pool: deadpool_postgres::Pool,
    server_config: ServerConfig,
) -> std::result::Result<impl Reply, Rejection> {
    let client: Client = db_pool.get().await.expect("Error connecting to database");

    if client_authorization.is_empty() {
        print!("Empty client auth?");
        return Err(warp::reject::custom(AuthorizationError(
            "Client credentials invalid".to_string(),
        )));
    }


    match params {
        Some(obj) => match obj.grant_type.as_str() {
            "password" => {
                if obj.username.is_some() && obj.password.is_some() {
                    let client_db_id = validate_client(client_authorization, &client).await;
                    let validation = db::validate_password_credentials(
                        &client,
                        obj.username.unwrap(),
                        obj.password.unwrap(),
                    )
                    .await;
                    if validation > 0 && client_db_id > 0 {
                        let token = generate_token();
                        let res = db::insert_token(
                            &client,
                            token.clone(),
                            obj.scope,
                            Some(validation),
                            client_db_id,
                            server_config.name,
                            obj.device,
                        )
                        .await;
                        return Ok(json(&res));
                    } else {
                        return Err(warp::reject::custom(AuthorizationError(
                            "client or user not found".to_string(),
                        )));
                    }
                }
            }
            "client_credentials" => {
                let client_db_id = validate_client(client_authorization, &client).await;
                if client_db_id > 0 {
                    let token = generate_token();
                    let res = db::insert_token(
                        &client,
                        token.clone(),
                        obj.scope,
                        None,
                        client_db_id,
                        server_config.name,
                        obj.device,
                    )
                    .await;
                    return Ok(json(&res));
                } else {
                    return Err(warp::reject::custom(AuthorizationError(
                        "client id not found".to_string(),
                    )));
                }
            }
            "authorization_code" => { //TODO add device hier ook, als extra check?
                println!("test");
                println!("{}", client_authorization);
                let client_db_id = validate_client(client_authorization, &client).await; //TODO support voor PCKE ipv client_secret hier
                let code = obj.code.unwrap();
                let pcke = obj.pcke.unwrap(); //TODO remove unwraps for safer code (unwrap will panic)
                println!("Found client {}", client_db_id);
                let user_id = validate_code(&client, &code, &pcke).await;
                println!("Found user {}", user_id);
                if client_db_id > 0 && user_id > 0 {
                    let token = generate_token();
                    let res = db::insert_token(
                        &client,
                        token.clone(),
                        obj.scope,
                        Some(user_id),
                        client_db_id,
                        server_config.name,
                        obj.device,
                    )
                    .await;
                    db::delete_code(&client, &code).await;
                    return Ok(json(&res));
                } else {
                    return Err(warp::reject::custom(AuthorizationError(
                        "client id or user id not found".to_string(),
                    )));
                }
            }
            _ => {
                return Err(warp::reject::custom(AuthorizationError(
                    "Unsupported grant type".to_string(),
                )));
            }
        },
        None => {}
    }
    print!("end of the road..");

    return Err(warp::reject::not_found());
}

pub async fn invalidate_token(
    db_pool: deadpool_postgres::Pool,
) -> std::result::Result<impl Reply, Rejection> {
    Ok("")
}

pub async fn get_health() -> std::result::Result<impl Reply, Rejection> {
    Ok(warp::reply::json(&"UP"))
}

pub async fn create_user(
    db_pool: deadpool_postgres::Pool,
) -> std::result::Result<impl Reply, Rejection> {
    Ok("")
}
