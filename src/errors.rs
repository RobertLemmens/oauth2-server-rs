use serde::Serialize;
use std::convert::Infallible;
use warp::{http::StatusCode, Rejection, Reply};
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Error getting connection from the database pool: {0}")]
    DBPoolError(deadpool_postgres::PoolError),
    #[error("Error executing query: {0}")]
    DBQueryError(#[from] tokio_postgres::Error),
    #[error("Error initializing database: {0}")]
    DBInitError(tokio_postgres::Error),
    #[error("Error authorizing: {0}")]
    AuthorizationError(String),
    #[error("Not found: {0}")]
    NotFoundError(String),
    #[error("Get request not allowed {0}")]
    GetRouteFailed(bool),
}

#[derive(Serialize)]
struct ErrorResponse {
    message: String,
}

impl warp::reject::Reject for Error {}

pub async fn handle_get_notallowed(err: Rejection) -> std::result::Result<impl Reply, Rejection> {
    if let Some(_) = err.find::<warp::reject::MethodNotAllowed>() {
        println!("get throwed not allowed");
        return Err(warp::reject::custom(Error::GetRouteFailed(false)));
        // return Err(warp::reject::custom(AuthorizationError("Client credentials invalid".to_string())));
    }
    println!("got err, but returning ok");
    Ok("")
    // Err(err)
}

pub async fn handle_rejection(err: Rejection) -> std::result::Result<impl Reply, Infallible> {
    let code;
    let message;
    println!("Big rejection handler");

    // TODO catch method not allowed and other common http errors
    if err.is_not_found() {
        println!("The error is not fuond..");
        code = StatusCode::NOT_FOUND;
        message = "Not Found";
    } else if let Some(_) = err.find::<warp::reject::MethodNotAllowed>() {
        println!("The error is not method not allowed..");
        code = StatusCode::NOT_FOUND;
        message = "Not Found";
    }
    else if let Some(e) = err.find::<Error>() {
        println!("${0}", e);
        match e {
            Error::AuthorizationError(e) => {
                code = StatusCode::UNAUTHORIZED;
                message = e; 
            }
            Error::NotFoundError(e) => {
                code = StatusCode::NOT_FOUND;
                message = e;
            }
            Error::GetRouteFailed(e) => {
                code = StatusCode::METHOD_NOT_ALLOWED;
                message = "Method not allowed";
            }
            _ => {
                code = StatusCode::UNAUTHORIZED;
                message = "Unauthorized"; 
            }
        }
    }
    else {
        eprintln!("unhandled error: {:?}", err);
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "Internal Server Error";
    }

    let json = warp::reply::json(&ErrorResponse { message: message.into(), });

    Ok(warp::reply::with_status(json, code))
}
