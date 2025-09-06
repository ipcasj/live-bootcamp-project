use dotenvy::dotenv;
use lazy_static::lazy_static;
use std::env as std_env;

lazy_static! {
    pub static ref JWT_SECRET: String = set_token();
    pub static ref REFRESH_TOKEN_SECRET: String = set_refresh_token();
}

fn set_token() -> String {
    dotenv().ok();
    let secret = std_env::var(env::JWT_SECRET_ENV_VAR).expect("JWT_SECRET must be set.");
    if secret.is_empty() {
        panic!("JWT_SECRET must not be empty.");
    }
    secret
}

fn set_refresh_token() -> String {
    dotenv().ok();
    let secret = std_env::var(env::REFRESH_TOKEN_SECRET_ENV_VAR).unwrap_or_else(|_| "refresh_secret_dev".to_string());
    if secret.is_empty() {
        panic!("REFRESH_TOKEN_SECRET must not be empty.");
    }
    secret
}

pub mod env {
    pub const JWT_SECRET_ENV_VAR: &str = "JWT_SECRET";
    pub const REFRESH_TOKEN_SECRET_ENV_VAR: &str = "REFRESH_TOKEN_SECRET";
}

pub const JWT_COOKIE_NAME: &str = "jwt";

pub const REFRESH_TOKEN_TTL_SECONDS: i64 = 60 * 60 * 24 * 7; // 7 days
