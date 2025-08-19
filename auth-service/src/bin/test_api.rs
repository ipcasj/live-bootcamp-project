use reqwest::Client;
use serde_json::json;

#[tokio::main]
async fn main() {
    let client = Client::new();
    let api_url = "http://localhost:3000";

    // Test /signup
    let signup_resp = client.post(format!("{}/signup", api_url))
        .json(&json!({"email": "test@example.com", "password": "password"}))
        .send().await;
    print_status("/signup", &signup_resp);

    // Test /login
    let login_resp = client.post(format!("{}/login", api_url))
        .json(&json!({"email": "test@example.com", "password": "password"}))
        .send().await;
    print_status("/login", &login_resp);

    // Test /logout
    let logout_resp = client.post(format!("{}/logout", api_url))
        .json(&json!({"jwt": "dummy"}))
        .send().await;
    print_status("/logout", &logout_resp);

    // Test /verify-2fa
    let verify_2fa_resp = client.post(format!("{}/verify-2fa", api_url))
        .json(&json!({"email": "test@example.com"}))
        .send().await;
    print_status("/verify-2fa", &verify_2fa_resp);

    // Test /verify-token
    let verify_token_resp = client.post(format!("{}/verify-token", api_url))
        .json(&json!({"token": "dummy"}))
        .send().await;
    print_status("/verify-token", &verify_token_resp);
}

fn print_status(endpoint: &str, resp: &Result<reqwest::Response, reqwest::Error>) {
    match resp {
        Ok(r) => println!("{}: HTTP {}", endpoint, r.status()),
        Err(e) => println!("{}: Error: {}", endpoint, e),
    }
}
