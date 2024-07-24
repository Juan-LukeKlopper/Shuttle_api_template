use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::serde::json::Json;
use rocket::State;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::claims::Claims;
use crate::models::user::User;


#[derive(Deserialize)]
pub struct LoginRequest {
    pub username: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct RegisterRequest {
    pub username: String,
    pub password: String,
}


#[derive(Serialize)]
pub struct LoginResponse {
    pub token: String,
}

#[derive(Serialize)]
pub struct PublicResponse {
    message: String,
}

#[post("/login", data = "<login>")]
pub async fn login(login: Json<LoginRequest>, pool: &State<PgPool>) -> Result<Json<LoginResponse>, Custom<String>> {
    let user = sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = $1")
        .bind(&login.username)
        .fetch_one(pool.inner())
        .await;

    match user {
        Ok(u) => {
            if u.password.as_ref() == Some(&login.password) {
                let claim = Claims::new(u.id);
                let response = LoginResponse {
                    token: claim.into_token()?,
                };
                Ok(Json(response))
            } else {
                Err(Custom(Status::Unauthorized, "Invalid username or password".to_string()))
            }
        }
        Err(_) => Err(Custom(Status::Unauthorized, "Invalid username or password".to_string())),
    }
}

#[post("/register", data = "<register>")]
pub async fn register(register: Json<RegisterRequest>, pool: &State<PgPool>) -> Result<Json<PublicResponse>, Custom<String>> {
    let result = sqlx::query("INSERT INTO users (username, password) VALUES ($1, $2)")
        .bind(&register.username)
        .bind(&register.password)
        .bind(Option::<String>::None)
        .execute(pool.inner())
        .await;

    match result {
        Ok(_) => Ok(Json(PublicResponse {
            message: "User registered successfully".to_string(),
        })),
        Err(_) => Err(Custom(Status::Conflict, "Username already taken".to_string())),
    }
}
