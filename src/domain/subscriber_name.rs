use std::fmt::Formatter;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Debug, serde::Deserialize)]
pub struct SubscriberName(String);

const NAME_MAX_LENGTH: usize = 255;

impl std::fmt::Display for SubscriberName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl SubscriberName {
    pub fn parse(name: String) -> Result<Self, String> {
        // Checks if it's empty
        let is_empty = name.trim().is_empty();

        // Checks length
        let is_too_long = name.graphemes(true).count() > NAME_MAX_LENGTH;

        // Checks for forbiddden chars
        let forbidden_chars = ['/', '(', ')', '"', '<', '>', '\\', '{', '}'];
        let contains_forbidden_chars = name.chars().any(|char| forbidden_chars.contains(&char));

        if is_empty || is_too_long || contains_forbidden_chars {
            let err_msg = format!(
                "Invalid name.{}{}{}",
                if is_empty { " Empty name." } else { "" },
                if is_too_long { " Name too long." } else { "" },
                if contains_forbidden_chars {
                    " Forbidden characters present."
                } else {
                    ""
                },
            );
            Err(err_msg)
        } else {
            Ok(Self(name))
        }
    }
}

impl AsRef<str> for SubscriberName {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use claims::{assert_err, assert_ok};

    #[test]
    fn name_empty_fails() {
        assert_err!(SubscriberName::parse("".to_string()));
    }

    #[test]
    fn name_too_long_fails() {
        assert_err!(SubscriberName::parse("a".repeat(NAME_MAX_LENGTH + 1)));
    }

    #[test]
    fn name_forbidden_chars_fails() {
        assert_err!(SubscriberName::parse("/".to_string()));
    }

    #[test]
    fn name_valid_succeeds() {
        assert_ok!(SubscriberName::parse("John".to_string()));
    }
}
