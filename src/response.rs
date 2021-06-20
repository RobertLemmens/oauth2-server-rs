use warp::{Rejection};

pub type Response = std::result::Result<warp::reply::Json, Rejection>;
