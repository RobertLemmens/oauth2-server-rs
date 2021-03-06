use crate::models::{AccessToken, Introspection, User};
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

// validate in 1 go? 
pub async fn validate_access_token(client: &Client, access_token: String, client_id: String) -> Introspection {
    let statement = client.prepare("select a.scope, a.expire_time, a.creation_time, c.username, b.client_id, b.display_name, a.token_type from access_tokens as a join clients as b on a.client_id = b.id join users as c on a.user_id = c.id where a.access_token = $1 and b.client_id = $2").await.unwrap(); 
    let response = client.query(&statement, &[&access_token, &client_id]).await.expect("Error executing query on access token/clients table");

    Introspection { 
        active: true, 
        client_id: response[0].get(4), 
        username: response[0].get(3), 
        scope: response[0].get(0), 
        token_type: response[0].get(6), 
        exp: 1, 
        iat: 1 
    }
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

    let client_response = client.query(&statement, &[&client_id, &secret]).await.expect("Error executing query on clients table");

    if client_response.len() == 1 {
        return client_response.get(0).unwrap().get(0);
    } else {
        return 0;
    }

}

pub async fn insert_token(client: &Client, generated_token: String, uid: i32, cid: i32) -> AccessToken {
    let statement = client.prepare("insert into access_tokens (access_token, expire_time, user_id, client_id, scope, creation_time, token_type) values($1, $2, $3, $4, $5, NOW(), 'bearer') on conflict on constraint unique_uid_cid do update set access_token = $1, expire_time = $2").await.unwrap();
    let local: DateTime<Local> = Local::now();

    let all_scope: String = "read+write".to_string();

    let result = client.query(&statement, &[&generated_token, &local, &uid, &cid, &all_scope]).await.expect("Error creating access token");

    AccessToken {
        access_token: String::from(generated_token),
        token_type: "bearer".to_string(), 
        expires_in: local.to_string(),
        scope: all_scope
    }

}

pub async fn register_user() {

}
