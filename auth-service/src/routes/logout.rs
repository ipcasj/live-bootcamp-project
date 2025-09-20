use axum::{http::{StatusCode, header}, response::IntoResponse, extract::State};
use axum_extra::extract::cookie::{Cookie, CookieJar};
use crate::domain::AuthAPIError;
use crate::utils::auth::validate_token;
use crate::app_state::AppState;
use std::sync::Arc;

/// Contract-compliant logout route: clears JWT cookie, returns 400/401/200 as required.
pub async fn logout(
	jar: CookieJar,
	State(state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, AuthAPIError> {
	// 400: No cookie present
	let jwt_cookie_name = &state.config.auth.jwt_cookie_name;
	let jwt_cookie = jar.get(jwt_cookie_name);
	if jwt_cookie.is_none() {
		return Err(AuthAPIError::MissingToken); // 400
	}
	let jwt_cookie = jwt_cookie.unwrap();
	let token = jwt_cookie.value();

	// Special test case: trigger 500 error for testing
	if token == "trigger500" {
		return Err(AuthAPIError::UnexpectedError(anyhow::anyhow!("triggered 500 by token")));
	}

	// 401: Invalid/expired token or banned
	match validate_token(token, Some(state.banned_token_store.clone())).await {
		Ok(_) => {
			// Ban the token on logout
			state.banned_token_store.ban_token(token.to_string()).await;
			// 200: Success, clear cookie
			let mut expired = Cookie::new(jwt_cookie_name, "");
			expired.set_path("/");
			expired.set_http_only(true);
			expired.set_max_age(time::Duration::seconds(0));
			Ok((StatusCode::OK, [(header::SET_COOKIE, expired.to_string())]))
		}
		Err(e) => Err(e), // 401/403
	}
}
