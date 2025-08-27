/// OpenAPI documentation for the auth-service API.
use utoipa::OpenApi;
use crate::routes::signup::{SignupRequest, SignupResponse};

#[derive(OpenApi)]
#[openapi(
    paths(
        crate::routes::signup::signup,
        crate::routes::signup::health
    ),
    components(schemas(SignupRequest, SignupResponse)),
    tags(
        (name = "auth", description = "Authentication endpoints"),
        (name = "health", description = "Health check endpoints")
    )
)]
pub struct ApiDoc;
