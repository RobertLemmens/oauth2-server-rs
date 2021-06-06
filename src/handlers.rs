use deadpool_postgres::Client;
use rand::distributions::Alphanumeric;
use rand::Rng;
use warp::{reply::json, Rejection, Reply};
use std::str;
use crate::models::{ServerConfig, TokenParams};
use crate::db;
use crate::errors::Error::*;

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

// Introspect a token
pub async fn introspect_token(
    client_authorization: String,
    access_token: String,
    db_pool: deadpool_postgres::Pool,
) -> std::result::Result<impl Reply, Rejection> {
    let client: Client = db_pool.get().await.expect("Error connecting to database");
    let client_db_id = validate_client(client_authorization, &client).await;
    if client_db_id == 0 {
        return Err(warp::reject::custom(AuthorizationError("Client credentials invalid".to_string())));
    }
    let result = db::validate_access_token(&client, access_token, client_db_id);


    match result.await {
        None => return Err(warp::reject::custom(NotFoundError("Unknown token".to_string()))),
        Some(x) => return Ok(json(&x)),
    }
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
        return Err(warp::reject::custom(AuthorizationError("Client credentials invalid".to_string())));
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
                        )
                        .await;
                        return Ok(json(&res));
                    } else {
                        return Err(warp::reject::custom(AuthorizationError("client or user not found".to_string())));
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
                    )
                    .await;
                    return Ok(json(&res));
                } else {
                    return Err(warp::reject::custom(AuthorizationError("client id not found".to_string())));
                }
            }
            _ => {
                return Err(warp::reject::custom(AuthorizationError("Unsupported grant type".to_string())));
            }
        },
        None => {}
    }

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
