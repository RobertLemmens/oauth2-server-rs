use crate::models::{User, AccessToken};
use deadpool_postgres::Client;
use tokio_pg_mapper::FromTokioPostgresRow;
use std::io;
use chrono::{Local, DateTime};

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

pub async fn validate_password_credentials(client: &Client, username: String, password: String) -> i32 {
    let statement = client.prepare("select id from users where username = $1 and password = $2").await.unwrap(); 

    let user = client.query(&statement, &[&username, &password]).await.expect("Error executing query on users table");

    if user.len() == 1 {
        return user.get(0).unwrap().get(0);
    } else {
        return 0;
    }

}

pub async fn validate_client_credentials(client: &Client, client_id: String, secret: String) -> i32 {
    let statement = client.prepare("select id from clients where client_id = $1 and client_secret = $2").await.unwrap(); 

    let client_response = client.query(&statement, &[&client_id, &secret]).await.expect("Error executing query on users table");

    if client_response.len() == 1 {
        return client_response.get(0).unwrap().get(0);
    } else {
        return 0;
    }

}

pub async fn insert_token(client: &Client, generated_token: String, uid: i32, cid: i32) -> AccessToken {
    let statement = client.prepare("insert into access_tokens (access_token, expire_time, user_id, client_id) values($1, $2, $3, $4) on conflict on constraint unique_uid_cid do update set access_token = $1, expire_time = $2").await.unwrap();
    let local: DateTime<Local> = Local::now();

    let result = client.query(&statement, &[&generated_token, &local, &uid, &cid]).await.expect("Error creating access token");

    let token = AccessToken {
        access_token: String::from(generated_token),
        token_type: "bearer".to_string(), 
        expire_time: local.to_string()
    };

    return token;
}

pub async fn create_user() {

}

// pub async fn get_tokens() {
// }
