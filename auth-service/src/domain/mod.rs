pub mod password;
pub use password::Password;
pub mod email;
pub use email::Email;
/// Domain module: contains core types, traits, and errors for the auth-service.
pub mod data_stores;
pub use data_stores::{UserStore, UserStoreError};
pub mod error;
pub use error::AuthAPIError;
pub use user::User;
pub mod user;
