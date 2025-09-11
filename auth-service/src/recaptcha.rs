use serde::Deserialize;

#[derive(Deserialize)]
pub struct RecaptchaResponse {
    pub success: bool,
    pub score: Option<f32>,
    pub action: Option<String>,
    pub challenge_ts: Option<String>,
    pub hostname: Option<String>,
    pub error_codes: Option<Vec<String>>,
}

pub async fn verify_recaptcha(token: &str, secret: &str) -> Result<RecaptchaResponse, reqwest::Error> {
    let client = reqwest::Client::new();
    let params = [
        ("secret", secret),
        ("response", token),
    ];
    let resp = client
        .post("https://www.google.com/recaptcha/api/siteverify")
        .form(&params)
        .send()
        .await?
        .json::<RecaptchaResponse>()
        .await?;
    Ok(resp)
}
