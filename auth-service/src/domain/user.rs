// The User struct should contain 3 fields. email, which is a String; 
// password, which is also a String; and requires_2fa, which is a boolean. 
//Note: You will also need to update the User struct to derive a few traits for the unit tests to pass.
use super::{Email, Password};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct User {
    pub email: Email,
    pub password: Password,
    pub requires_2fa: bool,
    pub two_fa_method: TwoFAMethod,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize, utoipa::ToSchema)]
pub enum TwoFAMethod {
    Email,
    AuthenticatorApp,
    SMS,
}

impl Default for TwoFAMethod {
    fn default() -> Self {
        TwoFAMethod::Email
    }
}

impl User {
    pub fn new(email: Email, password: Password, requires_2fa: bool) -> Self {
        User {
            email,
            password,
            requires_2fa,
            two_fa_method: TwoFAMethod::default(),
        }
    }

    pub fn with_2fa_method(email: Email, password: Password, requires_2fa: bool, two_fa_method: TwoFAMethod) -> Self {
        User {
            email,
            password,
            requires_2fa,
            two_fa_method,
        }
    }
}