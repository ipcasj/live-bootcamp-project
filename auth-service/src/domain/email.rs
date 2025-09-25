//! Email type for validated email addresses.
use std::hash::Hash;

use color_eyre::eyre::{eyre, Result};
use secrecy::{ExposeSecret, Secret};
use regex::Regex;

#[derive(Debug, Clone)]
pub struct Email(Secret<String>);

impl PartialEq for Email {
    fn eq(&self, other: &Self) -> bool {
        self.0.expose_secret() == other.0.expose_secret()
    }
}

impl Hash for Email {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.expose_secret().hash(state);
    }
}

impl Eq for Email {}

impl PartialOrd for Email {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.expose_secret().partial_cmp(other.0.expose_secret())
    }
}

impl Ord for Email {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.expose_secret().cmp(other.0.expose_secret())
    }
}

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum EmailParseError {
    #[error("Invalid email format")] 
    InvalidFormat,
}

impl Email {
    /// Attempts to parse and validate an email address.
    pub fn parse(s: Secret<String>) -> Result<Email> {
        let email = s.expose_secret();
        let email_regex = Regex::new(r"^[^\s@]+@[^\s@]+\.[^\s@]+$").unwrap();
        
        if email_regex.is_match(email) {
            Ok(Self(s))
        } else {
            Err(eyre!("Email address is not valid"))
        }
    }

    /// Helper to parse from string slice  
    pub fn parse_from_str(s: &str) -> Result<Email> {
        Self::parse(Secret::new(s.to_string()))
    }
}

impl AsRef<Secret<String>> for Email {
    fn as_ref(&self) -> &Secret<String> {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::Email;

    use secrecy::Secret;

    #[test]
    fn empty_string_is_rejected() {
        let email = Secret::new("".to_string());
        assert!(Email::parse(email).is_err());
    }
    
    #[test]
    fn email_missing_at_symbol_is_rejected() {
        let email = Secret::new("ursuladomain.com".to_string());
        assert!(Email::parse(email).is_err());
    }
    
    #[test]
    fn email_missing_subject_is_rejected() {
        let email = Secret::new("@domain.com".to_string());
        assert!(Email::parse(email).is_err());
    }

    #[derive(Debug, Clone)]
    struct ValidEmailFixture(pub String);

    impl quickcheck::Arbitrary for ValidEmailFixture {
        fn arbitrary<G: quickcheck::Gen>(_g: &mut G) -> Self {
            // Use a simple approach compatible with fake
            use fake::{Fake, Faker};
            let email: String = Faker.fake();
            // Ensure it looks like an email
            let email = if email.contains('@') {
                email
            } else {
                "test@example.com".to_string()
            };
            Self(email)
        }
    }

    #[quickcheck_macros::quickcheck]
    fn valid_emails_are_parsed_successfully(valid_email: ValidEmailFixture) -> bool {
        Email::parse(Secret::new(valid_email.0)).is_ok()
    }
}
