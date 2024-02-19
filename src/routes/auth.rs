use std::collections::HashMap;

use aes::{
    cipher::{generic_array::GenericArray, BlockDecrypt, BlockEncrypt, KeyInit},
    Aes128,
};
use axum::{
    extract::{Query, State},
    response::{Html, IntoResponse, Redirect},
};
use base64::{engine::general_purpose, Engine};
use reqwest::Client;
use serde::Deserialize;
use tower_sessions::Session;
use serde_json::json;

use crate::{error::Error, state::AppState};

/// routes
pub async fn authorize(State(state): State<AppState<'_>>) -> Redirect {
    let uri = format!(
        "https://github.com/login/oauth/authorize?scope=user:email&client_id={}",
        state.client_id
    );
    Redirect::to(&uri)
}

pub async fn destroy(session: Session) -> impl IntoResponse {
    let user = session.get_value("rmap_username").await.unwrap().unwrap_or_default();
    tracing::info!("Logging out user: {}", user);

    let _ = session.flush().await;
    Redirect::to("/")
}

pub async fn callback(
    session: Session,
    State(state): State<AppState<'_>>,
    Query(user): Query<UserToken>,
) -> impl IntoResponse {
    let mut params: HashMap<&str, &str> = HashMap::new();
    params.insert("client_id", &state.client_id);
    params.insert("client_secret", &state.client_secret);
    params.insert("code", &user.code);

    let response = get_access_token(&params).await;
    match response {
        Ok(res) => {
            let json = json::parse(&res.text().await.unwrap()).unwrap();
            let token = json["access_token"].to_string().to_owned();

            let _ = session.insert("token", encrypt_token(&token, state.key).await).await.unwrap();

            Ok(Html::from(
                state
                    .hb
                    .render(
                        "redirect",
                        &json!({}),
                    )
                    .unwrap(),
            )
            .into_response())
        }
        Err(_) => Err(Error::Unauthorized),
    }
}

/// service
pub async fn get_access_token(
    params: &HashMap<&str, &str>,
) -> Result<reqwest::Response, reqwest::Error> {
    let response = Client::new()
        .post("https://github.com/login/oauth/access_token")
        .form(params)
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .header(reqwest::header::ACCEPT, "application/json")
        .send()
        .await;

    response
}

pub async fn verify_user(token: &str) -> Result<reqwest::Response, reqwest::Error> {
    let response = Client::new()
        .get("https://api.github.com/user")
        .bearer_auth(token)
        .header("X-GitHub-Api-Version", "2022-11-28")
        .header("User-Agent", "rtree")
        .send()
        .await;

    response
}

pub async fn encrypt_token(token: &str, key: [u8; 16]) -> String {

    let mut front = GenericArray::clone_from_slice(token[0..16].as_bytes());
    let mut back = GenericArray::clone_from_slice(token[24..40].as_bytes());

    let aes = Aes128::new(&GenericArray::from(key));
    aes.encrypt_block(&mut front);
    aes.encrypt_block(&mut back);

    let back = general_purpose::STANDARD.encode(&back);
    let mut cipher = general_purpose::STANDARD.encode(&front);

    cipher.push_str(&back);
    cipher.push_str(&token[16..24]);

    cipher
}

pub async fn decrypt_token(token: &str, key: [u8; 16]) -> String {

    let middle = &token[48..56];
    let front = &token[0..24];
    let back = &token[24..48];

    let front = general_purpose::STANDARD.decode(&front).unwrap();
    let back = general_purpose::STANDARD.decode(&back).unwrap();

    let mut front = GenericArray::clone_from_slice(&front);
    let mut back = GenericArray::clone_from_slice(&back);

    let aes = Aes128::new(&GenericArray::from(key));
    aes.decrypt_block(&mut front);
    aes.decrypt_block(&mut back);

    let mut plain = String::from_utf8((&front).to_vec()).unwrap();
    plain.push_str(middle);
    plain.push_str(&String::from_utf8(back.to_vec()).unwrap());

    plain
}

/// model
#[derive(Deserialize)]
pub struct UserToken {
    pub code: String,
}

#[derive(Deserialize)]
pub struct User {
    pub login: String,
    pub name: Option<String>,
    pub email: Option<String>,
    pub twitter_username: Option<String>,
}
