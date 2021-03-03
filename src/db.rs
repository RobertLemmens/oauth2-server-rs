use crate::models::{User, AccessToken};
use deadpool_postgres::Client;
use tokio_pg_mapper::FromTokioPostgresRow;
use std::io;

pub async fn get_users(client: &Client) -> Result<Vec<User>, io::Error> {
    let statement = client.prepare("select * from users").await.unwrap();

    // (query, parameterlist)
    let users = client.query(&statement, &[])
        .await
        .expect("Error executing query on users table")
        .iter()
        .map(|row| User::from_row_ref(row).unwrap())
        .collect::<Vec<User>>();

    Ok(users)
}

pub async fn validate_credentials(client: &Client, username: String, password: String) -> bool {
    let statement = client.prepare("select id from users where username = $1 and password = $2").await.unwrap(); 

    let user = client.query(&statement, &[&username, &password]).await.expect("Error executing query on users table");

    if user.len() == 0 {
        return false;
    } else {
        return true;
    }

}

pub async fn create_user() {

}

// pub async fn get_tokens() {
// }
