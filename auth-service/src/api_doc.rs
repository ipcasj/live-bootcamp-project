/// OpenAPI documentation for the auth-service API.
use utoipa::OpenApi;
use crate::routes::signup::{SignupRequestRest, SignupResponseRest};
use crate::routes::login::{TwoFactorAuthResponseRest, LoginResponseRest};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::signup::signup,
        crate::routes::signup::health
    ),
    components(schemas(SignupRequestRest, SignupResponseRest, TwoFactorAuthResponseRest, LoginResponseRest)),
    tags(
        (name = "auth", description = "Authentication endpoints"),
        (name = "health", description = "Health check endpoints")
    )
)]
pub struct ApiDoc;
