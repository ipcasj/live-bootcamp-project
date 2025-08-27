//! Password type for validated passwords.

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Password(String);

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PasswordParseError {
    #[error("Password must be at least 8 characters")] 
    TooShort,
}

impl Password {
    /// Attempts to parse and validate a password.
    pub fn parse(s: &str) -> Result<Self, PasswordParseError> {
        if s.len() >= 8 {
            Ok(Password(s.to_owned()))
        } else {
            Err(PasswordParseError::TooShort)
        }
    }
}

impl AsRef<str> for Password {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_password_is_accepted() {
        let pw = "password123";
        assert!(Password::parse(pw).is_ok());
    }

    #[test]
    fn short_password_is_rejected() {
        let pw = "short";
        assert!(Password::parse(pw).is_err());
    }

    #[test]
    fn as_ref_returns_inner() {
        let pw = Password::parse("password123").unwrap();
        assert_eq!(pw.as_ref(), "password123");
    }
}
