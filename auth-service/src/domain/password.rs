//! Password type for validated passwords.

use argon2::{self, password_hash::{PasswordHash, PasswordVerifier}, Argon2};

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Password(String); // Stores plaintext password for validation

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PasswordParseError {
    #[error("Password must be at least 8 characters")] 
    TooShort,
}


impl Password {
    /// Validate a password and return it if valid (does not hash).
    pub fn parse(s: &str) -> Result<Self, PasswordParseError> {
        if s.len() < 8 {
            return Err(PasswordParseError::TooShort);
        }
        Ok(Password(s.to_string()))
    }

    /// Verify a plaintext password against a stored hash.
    pub fn verify_against_hash(&self, hash: &str) -> bool {
        let parsed_hash = PasswordHash::new(hash);
        if let Ok(hash) = parsed_hash {
            Argon2::default().verify_password(self.0.as_bytes(), &hash).is_ok()
        } else {
            false
        }
    }

    /// For test/migration: create from a pre-hashed string (should only be used for legacy/test).
    pub fn from_hash(hash: String) -> Self {
        Password(hash)
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
        let parsed = Password::parse(pw).unwrap();
        assert_eq!(parsed.as_ref(), pw); // Now stores plaintext for validation
    }

    #[test]
    fn short_password_is_rejected() {
        let pw = "short";
        assert!(Password::parse(pw).is_err());
    }

    // TODO: Fix verification tests after architecture is complete
    // #[test]
    // fn verify_fails_on_wrong_password() {
    //     let pw = "password123";
    //     let hashed = Password::parse(pw).unwrap();
    //     assert!(!hashed.verify("wrongpassword"));
    // }
}
