//! Email type for validated email addresses.
use validator::validate_email;

#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Email(String);

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum EmailParseError {
    #[error("Invalid email format")] 
    InvalidFormat,
}

impl Email {
    /// Attempts to parse and validate an email address.
    pub fn parse(s: &str) -> Result<Self, EmailParseError> {
        if validate_email(s) {
            Ok(Email(s.to_owned()))
        } else {
            Err(EmailParseError::InvalidFormat)
        }
    }
}

impl AsRef<str> for Email {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use fake::{faker::internet::en::SafeEmail, Fake};
    use quickcheck_macros::quickcheck;

    #[test]
    fn valid_email_is_accepted() {
        let email = "user@example.com";
        assert!(Email::parse(email).is_ok());
    }

    #[test]
    fn invalid_email_is_rejected() {
        let email = "not-an-email";
        assert!(Email::parse(email).is_err());
    }

    #[test]
    fn as_ref_returns_inner() {
        let email = Email::parse("user@example.com").unwrap();
        assert_eq!(email.as_ref(), "user@example.com");
    }

    #[quickcheck]
    fn property_based_valid_emails_are_accepted() {
        let email: String = SafeEmail().fake();
        Email::parse(&email).expect("Should accept valid fake email");
    }

    #[quickcheck]
    fn property_based_invalid_emails_are_rejected(s: String) {
        if !validate_email(&s) {
            assert!(Email::parse(&s).is_err());
        }
    }
}
