//! Password type for validated passwords.

use argon2::{self, password_hash::{PasswordHash, PasswordVerifier}, Argon2};
use color_eyre::eyre::{eyre, Result};
use secrecy::{ExposeSecret, Secret};

#[derive(Debug, Clone)]
pub struct Password(Secret<String>);

impl PartialEq for Password {
    fn eq(&self, other: &Self) -> bool {
        // We can use the expose_secret method to expose the secret in a
        // controlled manner when needed!
        self.0.expose_secret() == other.0.expose_secret()
    }
}

impl Eq for Password {}

impl std::hash::Hash for Password {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.expose_secret().hash(state);
    }
}

impl PartialOrd for Password {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.expose_secret().partial_cmp(other.0.expose_secret())
    }
}

impl Ord for Password {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.expose_secret().cmp(other.0.expose_secret())
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum PasswordParseError {
    #[error("Password must be at least 8 characters")] 
    TooShort,
}

impl Password {
    pub fn parse(s: Secret<String>) -> Result<Password> {
        if validate_password(&s) {
            Ok(Self(s))
        } else {
            Err(eyre!("Failed to parse string to a Password type"))
        }
    }

    /// Helper to parse from string slice  
    pub fn parse_from_str(s: &str) -> Result<Password> {
        Self::parse(Secret::new(s.to_string()))
    }

    /// Verify a plaintext password against a stored hash.
    pub fn verify_against_hash(&self, hash: &str) -> bool {
        let parsed_hash = PasswordHash::new(hash);
        if let Ok(hash) = parsed_hash {
            Argon2::default().verify_password(self.0.expose_secret().as_bytes(), &hash).is_ok()
        } else {
            false
        }
    }

    /// For test/migration: create from a pre-hashed string (should only be used for legacy/test).
    pub fn from_hash(hash: String) -> Self {
        Password(Secret::new(hash))
    }
}

fn validate_password(s: &Secret<String>) -> bool {
    s.expose_secret().len() >= 8
}

impl AsRef<Secret<String>> for Password {
    fn as_ref(&self) -> &Secret<String> {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::Password;

    use secrecy::Secret;

    #[test]
    fn empty_string_is_rejected() {
        let password = Secret::new("".to_string());
        assert!(Password::parse(password).is_err());
    }
    
    #[test]
    fn string_less_than_8_characters_is_rejected() {
        let password = Secret::new("1234567".to_string());
        assert!(Password::parse(password).is_err());
    }

    #[derive(Debug, Clone)]
    struct ValidPasswordFixture(pub Secret<String>);

    impl quickcheck::Arbitrary for ValidPasswordFixture {
        fn arbitrary<G: quickcheck::Gen>(_g: &mut G) -> Self {
            // Use a simple approach compatible with fake
            use fake::{Fake, Faker};
            let password: String = Faker.fake();
            // Ensure it meets password requirements
            let password = if password.len() < 8 {
                "password123".to_string()
            } else {
                password
            };
            Self(Secret::new(password))
        }
    }
    
    #[quickcheck_macros::quickcheck]
    fn valid_passwords_are_parsed_successfully(valid_password: ValidPasswordFixture) -> bool {
        Password::parse(valid_password.0).is_ok()
    }
}
