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
}

#[derive(Serialize)]
struct ErrorResponse {
    message: String,
}

impl warp::reject::Reject for Error {}

pub async fn handle_rejection(err: Rejection) -> std::result::Result<impl Reply, Infallible> {
    let code;
    let message;

    if err.is_not_found() {
        code = StatusCode::NOT_FOUND;
        message = "Not Found";
    } else if let Some(e) = err.find::<Error>() {
        match e {
            Error::AuthorizationError(e) => {
                code = StatusCode::UNAUTHORIZED;
                message = e; 
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

    let json = warp::reply::json(&ErrorResponse { message: message.into(), });

    Ok(warp::reply::with_status(json, code))
}
