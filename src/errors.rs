use serde::Serialize;
use std::convert::Infallible;
use thiserror::Error;
use warp::{http::StatusCode, Rejection, Reply};

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
    println!("{:?}", err);
    if let Some(_) = err.find::<warp::reject::MethodNotAllowed>() {
        return Err(warp::reject::custom(Error::GetRouteFailed(false)));
        // return Err(warp::reject::custom(AuthorizationError("Client credentials invalid".to_string())));
    } else if err.is_not_found() {
        return Ok(warp::reply::with_status(
            warp::reply::json(&ErrorResponse {
                message: "Not Found".into(),
            }),
            StatusCode::NOT_FOUND,
        ));
    }
    Ok(warp::reply::with_status(
        warp::reply::json(&""),
        StatusCode::OK,
    ))
    // Err(err)
}

pub async fn handle_rejection(err: Rejection) -> std::result::Result<impl Reply, Infallible> {
    let code;
    let message;

    println!("{:?}", err);

    // else if let Some(_) = err.find::<warp::reject::MethodNotAllowed>() {
    //         code = StatusCode::NOT_FOUND; // this should be a 405...?
    //         message = "Not Found";
    //     }
    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "Not Found";
    } else if let Some(e) = err.find::<Error>() {
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
    } else {
        eprintln!("unhandled error: {:?}", err);
        code = StatusCode::INTERNAL_SERVER_ERROR;
        message = "Internal Server Error";
    }

    let json = warp::reply::json(&ErrorResponse {
        message: message.into(),
    });

    Ok(warp::reply::with_status(json, code))
}
