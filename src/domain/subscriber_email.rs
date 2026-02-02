use std::fmt::Formatter;
use validator::ValidateEmail;

#[derive(Debug, serde::Deserialize, Clone)]
pub struct SubscriberEmail(String);

impl std::fmt::Display for crate::domain::SubscriberEmail {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl SubscriberEmail {
    pub fn parse(email: String) -> Result<Self, String> {
        if email.validate_email() {
            Ok(Self(email))
        } else {
            Err("Invalid email address".to_string())
        }
    }
}

impl AsRef<str> for SubscriberEmail {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::{assert_err, assert_ok};
    use fake::Fake;
    use fake::faker::internet::en::SafeEmail;
    use fake::rand::{SeedableRng, rngs::StdRng};
    use quickcheck::Gen;

    // Quickcheck type for testing emails
    #[derive(Debug, Clone)]
    struct ValidEmailFixture(pub String);

    impl quickcheck::Arbitrary for ValidEmailFixture {
        fn arbitrary(g: &mut Gen) -> Self {
            let mut rng = StdRng::seed_from_u64(u64::arbitrary(g));
            let email = SafeEmail().fake_with_rng(&mut rng);
            Self(email)
        }
    }

    #[quickcheck_macros::quickcheck]
    fn valid_emails_are_parsed_correctly(valid_email: ValidEmailFixture) -> bool {
        SubscriberEmail::parse(valid_email.0.clone()).is_ok()
    }

    #[test]
    fn empty_email_fails() {
        assert_err!(SubscriberEmail::parse("".to_string()));
    }

    #[test]
    fn email_missing_at_symbol_is_rejected() {
        let email = "ursuladomain.com".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }
    #[test]
    fn email_missing_subject_is_rejected() {
        let email = "@domain.com".to_string();
        assert_err!(SubscriberEmail::parse(email));
    }

    #[test]
    fn valid_email_succeeds() {
        let email = SafeEmail().fake();
        assert_ok!(SubscriberEmail::parse(email));
    }
}
