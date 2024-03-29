use crate::models::{AccessToken, Introspection, User};
use chrono::{DateTime, Duration, Local};
use deadpool_postgres::Client;
use tokio_pg_mapper::FromTokioPostgresRow;
use sha2::{Sha256, Digest};
use uuid::Uuid;
// validate in 1 go?
pub async fn validate_access_token(
    client: &Client,
    access_token: String,
    client_db_id: Uuid,
) -> Option<Introspection> {
    let statement = client.prepare("select a.scope, a.expire_time, a.creation_time, c.username, c.id, b.client_id, b.display_name, a.token_type, a.issuer
                                   from access_tokens as a join clients as b on a.client_id = b.id left join users as c on a.user_id = c.id 
                                   where a.access_token = $1 and b.id = $2").await.unwrap();
    let response = client
        .query(&statement, &[&access_token, &client_db_id])
        .await
        .expect("Error executing query on access token/clients table");

    //TODO check if response is not empty

    if response.is_empty() {
        return None;
    }

    let expire_time: DateTime<Local> = response[0].get(1);
    let is_active = if expire_time < Local::now() {
        false
    } else {
        true
    };

    let creation_time: DateTime<Local> = response[0].get(2);

    return Some(Introspection {
        active: is_active,
        client_id: response[0].get(5),
        username: response[0].get(3),
        user_id: response[0].get(4),
        scope: response[0].get(0),
        token_type: response[0].get(7),
        issuer: response[0].get(8),
        exp: expire_time.timestamp(),
        iat: creation_time.timestamp(),
    })
}

pub async fn validate_password_credentials(
    client: &Client,
    username: String,
    password: String,
) -> Option<Uuid> {
    let statement = client
        .prepare("select id from users where username = $1 and password = $2")
        .await
        .unwrap();

    let user = client
        .query(&statement, &[&username, &password])
        .await
        .expect("Error executing query on users table");

    if user.len() == 1 {
        return Some(user.get(0).unwrap().get(0));
    } else {
        return None;
    }
}

pub async fn validate_client_credentials(
    client: &Client,
    client_id: String,
    secret: String,
) -> Option<Uuid> {
    let statement = client
        .prepare("select id from clients where client_id = $1 and client_secret = $2")
        .await
        .unwrap();

    let client_response = client
        .query(&statement, &[&client_id, &secret])
        .await
        .expect("Error executing query on clients table");

    if client_response.len() == 1 {
        return Some(client_response.get(0).unwrap().get(0));
    } else {
        return None;
    }
}

pub async fn validate_code(client: &Client, code: &String, pcke: &String) -> Option<Uuid> {
    let mut hasher = Sha256::new();
    hasher.update(pcke);
    let pcke_result = format!("{:X}", hasher.finalize()).to_lowercase();

    println!("Looking for code {}", code);
    println!("Looking for pcke {}", pcke_result);

    let statement = client
        .prepare("select * from authorization_codes where code = $1 and pcke_hash = $2")
        .await
        .unwrap();

    let code_response = client
        .query(&statement, &[&code, &pcke_result])
        .await
        .expect("Error executing query on authorization_codes table");

    if code_response.len() == 1 {
        // TODO check tijd op token
        return Some(code_response.get(0).unwrap().get(2));
    } else {
        return None;
    }
}

pub async fn delete_code(client: &Client, code: &String) {
    let statement = client
        .prepare("delete from authorization_codes where code = $1")
        .await
        .unwrap();

    let code_response = client
        .query(&statement, &[&code])
        .await
        .expect("Error deleting query on authorization_codes table");
}

pub async fn create_tables(client: &Client, script: &str) {
    let res = client.batch_execute(script).await;

    match res {
        Ok(_) => println!("Database created"),
        Err(msg) => println!("Error creating database, continuing startup. Message: ${0}", msg)
    };
}

pub async fn insert_token(
    client: &Client,
    generated_token: String,
    _scope: Option<String>,
    uid: Option<Uuid>,
    cid: Uuid,
    issuer: String,
    device: Option<String>,
) -> AccessToken {
    let statement = client.prepare("insert into access_tokens (access_token, expire_time, user_id, client_id, scope, creation_time, token_type, issuer, device) 
                                   values($1, $2, $3, $4, $5, NOW(), 'bearer', $6, $7) 
                                   on conflict on constraint unique_uid_cid do 
                                   update set access_token = $1, expire_time = $2, creation_time = NOW(), scope = $5, issuer = $6, device = $7").await.unwrap();
    let token_duration = Duration::days(30);
    let local: DateTime<chrono::Local> = Local::now() + token_duration;
    let device_str: String = match device {
        Some(x) => x,
        None => "unknown".to_string()
    };

    let _result = client
        .query(
            &statement,
            &[&generated_token, &local, &uid, &cid, &_scope, &issuer, &device_str],
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
