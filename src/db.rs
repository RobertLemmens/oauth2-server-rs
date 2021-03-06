use crate::models::{AccessToken, Introspection, User};
use chrono::{DateTime, Duration, Local};
use deadpool_postgres::Client;
use std::io;
use tokio_pg_mapper::FromTokioPostgresRow;

pub async fn get_users(client: &Client) -> Result<Vec<User>, io::Error> {
    let statement = client.prepare("select * from users").await.unwrap();

    // (query, parameterlist)
    let users = client
        .query(&statement, &[])
        .await
        .expect("Error executing query on users table")
        .iter()
        .map(|row| User::from_row_ref(row).unwrap())
        .collect::<Vec<User>>();

    Ok(users)
}

// validate in 1 go?
pub async fn validate_access_token(
    client: &Client,
    access_token: String,
    client_db_id: i32,
) -> Introspection {
    let statement = client.prepare("select a.scope, a.expire_time, a.creation_time, c.username, b.client_id, b.display_name, a.token_type, a.issuer 
                                   from access_tokens as a join clients as b on a.client_id = b.id left join users as c on a.user_id = c.id 
                                   where a.access_token = $1 and b.id = $2").await.unwrap();
    let response = client
        .query(&statement, &[&access_token, &client_db_id])
        .await
        .expect("Error executing query on access token/clients table");

    let expire_time: DateTime<Local> = response[0].get(1);
    let is_active = if expire_time < Local::now() {
        false
    } else {
        true
    };

    let creation_time: DateTime<Local> = response[0].get(2);

    Introspection {
        active: is_active,
        client_id: response[0].get(4),
        username: response[0].get(3),
        scope: response[0].get(0),
        token_type: response[0].get(6),
        issuer: response[0].get(7),
        exp: expire_time.timestamp(),
        iat: creation_time.timestamp(),
    }
}

pub async fn validate_password_credentials(
    client: &Client,
    username: String,
    password: String,
) -> i32 {
    let statement = client
        .prepare("select id from users where username = $1 and password = $2")
        .await
        .unwrap();

    let user = client
        .query(&statement, &[&username, &password])
        .await
        .expect("Error executing query on users table");

    if user.len() == 1 {
        return user.get(0).unwrap().get(0);
    } else {
        return 0;
    }
}

pub async fn validate_client_credentials(
    client: &Client,
    client_id: String,
    secret: String,
) -> i32 {
    let statement = client
        .prepare("select id from clients where client_id = $1 and client_secret = $2")
        .await
        .unwrap();

    let client_response = client
        .query(&statement, &[&client_id, &secret])
        .await
        .expect("Error executing query on clients table");

    if client_response.len() == 1 {
        return client_response.get(0).unwrap().get(0);
    } else {
        return 0;
    }
}

pub async fn insert_token(
    client: &Client,
    generated_token: String,
    _scope: Option<String>,
    uid: Option<i32>,
    cid: i32,
    issuer: String,
) -> AccessToken {
    let statement = client.prepare("insert into access_tokens (access_token, expire_time, user_id, client_id, scope, creation_time, token_type, issuer) 
                                   values($1, $2, $3, $4, $5, NOW(), 'bearer', $6) 
                                   on conflict on constraint unique_uid_cid do 
                                   update set access_token = $1, expire_time = $2, creation_time = NOW(), scope = $5, issuer = $6").await.unwrap();
    let token_duration = Duration::days(30);
    let local: DateTime<chrono::Local> = Local::now() + token_duration;

    let _result = client
        .query(
            &statement,
            &[&generated_token, &local, &uid, &cid, &_scope, &issuer],
        )
        .await
        .expect("Error creating access token");

    AccessToken {
        access_token: String::from(generated_token),
        token_type: "bearer".to_string(),
        expires_in: token_duration.num_seconds(),
        scope: _scope,
    }
}

pub async fn register_user() {}
