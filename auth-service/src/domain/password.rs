//! Password type for validated passwords.


use argon2::{self, password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString}, Argon2};
use rand::rngs::OsRng;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Password(String); // Always stores the hash

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PasswordParseError {
    #[error("Password must be at least 8 characters")] 
    TooShort,
}


impl Password {
    /// Hash and validate a password, returning a Password (hash) if valid.
    pub fn parse(s: &str) -> Result<Self, PasswordParseError> {
        if s.len() < 8 {
            return Err(PasswordParseError::TooShort);
        }
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let hash = argon2.hash_password(s.as_bytes(), &salt)
            .map_err(|_| PasswordParseError::TooShort)?
            .to_string();
        Ok(Password(hash))
    }

    /// Verify a plaintext password against the stored hash.
    pub fn verify(&self, password: &str) -> bool {
        let parsed_hash = PasswordHash::new(&self.0);
        if let Ok(hash) = parsed_hash {
            Argon2::default().verify_password(password.as_bytes(), &hash).is_ok()
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
    fn valid_password_is_accepted_and_hashes() {
        let pw = "password123";
        let hashed = Password::parse(pw).unwrap();
        assert_ne!(hashed.as_ref(), pw); // Should not store plaintext
        assert!(hashed.verify(pw));
    }

    #[test]
    fn short_password_is_rejected() {
        let pw = "short";
        assert!(Password::parse(pw).is_err());
    }

    #[test]
    fn verify_fails_on_wrong_password() {
        let pw = "password123";
        let hashed = Password::parse(pw).unwrap();
        assert!(!hashed.verify("wrongpassword"));
    }
}
