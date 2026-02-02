use crate::domain::SubscriberEmail;
use reqwest::Client;
use secrecy::{ExposeSecret, SecretBox};

pub struct EmailClient {
    pub sender: SubscriberEmail,
    pub http_client: Client,
    pub base_url: reqwest::Url,
    auth_token: SecretBox<String>,
}

impl EmailClient {
    pub fn new(
        base_url: reqwest::Url,
        sender: SubscriberEmail,
        auth_token: SecretBox<String>,
        timeout: std::time::Duration,
    ) -> Self {
        Self {
            sender,
            http_client: Client::builder().timeout(timeout).build().unwrap(),
            base_url,
            auth_token,
        }
    }

    pub async fn send_email(
        &self,
        recipient: SubscriberEmail,
        subject: &str,
        html_content: &str,
        raw_content: &str,
    ) -> Result<(), reqwest::Error> {
        let url = self.base_url.join("email").unwrap();

        let json_body = SendEmailRequest {
            from: self.sender.as_ref(),
            to: recipient.as_ref(),
            subject: subject,
            text_body: raw_content,
            html_body: html_content,
        };

        let req_builder = self
            .http_client
            .post(url)
            .header("X-Postmark-Server-Token", self.auth_token.expose_secret())
            .json(&json_body)
            .send()
            .await?
            .error_for_status()?;
        Ok(())
    }
}

// Structure of the request JSON body for the postmark API
#[derive(serde::Serialize)]
#[serde(rename_all = "PascalCase")]
struct SendEmailRequest<'a> {
    pub from: &'a str,
    pub to: &'a str,
    pub subject: &'a str,
    pub text_body: &'a str,
    pub html_body: &'a str,
}

#[cfg(test)]
mod tests {
    use crate::domain::SubscriberEmail;
    use crate::email_client::EmailClient;
    use claims::{assert_err, assert_ok};
    use fake::faker::internet::en::SafeEmail;
    use fake::faker::lorem::en::Sentence;
    use fake::{Fake, Faker};
    use secrecy::SecretBox;
    use wiremock::matchers::{any, header, header_exists, method, path};
    use wiremock::{Mock, MockServer, Request, ResponseTemplate};

    struct SendEmailBodyMatcher;

    impl wiremock::Match for SendEmailBodyMatcher {
        fn matches(&self, request: &Request) -> bool {
            let result: Result<serde_json::Value, _> = serde_json::from_slice(&request.body);
            if let Ok(body) = result {
                body.get("To").is_some()
                    && body.get("From").is_some()
                    && body.get("Subject").is_some()
                    && body.get("TextBody").is_some()
                    && body.get("HtmlBody").is_some()
            } else {
                false
            }
        }
    }

    struct EmailClientFixture {
        mock_server: MockServer,
        email_client: EmailClient,
    }

    impl EmailClientFixture {
        async fn new() -> Self {
            let mock_server = MockServer::start().await;
            let sender_email = SubscriberEmail::parse(SafeEmail().fake()).unwrap();
            let email_client = EmailClient::new(
                reqwest::Url::parse(&mock_server.uri()).expect("Failed to parse mock server url"),
                sender_email,
                SecretBox::new(Faker.fake()),
                std::time::Duration::from_millis(500),
            );
            Self {
                mock_server,
                email_client,
            }
        }

        fn random_recipient(&self) -> SubscriberEmail {
            SubscriberEmail::parse(SafeEmail().fake()).unwrap()
        }

        fn random_subject(&self) -> String {
            Sentence(1..2).fake()
        }

        fn random_content(&self) -> String {
            Sentence(1..10).fake()
        }

        async fn mount_status(&self, status: u16) {
            Mock::given(any())
                .respond_with(ResponseTemplate::new(status))
                .expect(1)
                .mount(&self.mock_server)
                .await;
        }

        async fn mount_delayed_status(&self, status: u16, delay: std::time::Duration) {
            Mock::given(any())
                .respond_with(ResponseTemplate::new(status).set_delay(delay))
                .expect(1)
                .mount(&self.mock_server)
                .await;
        }
    }

    #[tokio::test]
    async fn send_email_sends_expected_request() {
        // Arrange
        let fixture = EmailClientFixture::new().await;

        Mock::given(header_exists("X-Postmark-Server-Token"))
            .and(header("Content-Type", "application/json"))
            .and(path("/email"))
            .and(method("POST"))
            .and(SendEmailBodyMatcher)
            .respond_with(ResponseTemplate::new(200))
            .expect(1)
            .mount(&fixture.mock_server)
            .await;

        let subscriber_email = fixture.random_recipient();
        let subject = fixture.random_subject();
        let content = fixture.random_content();

        // Act
        let _ = fixture
            .email_client
            .send_email(subscriber_email, &subject, &content, &content)
            .await;

        // Assert
        // Mock expectations are checked on drop
    }

    #[tokio::test]
    async fn send_email_succeeds_if_server_200() {
        // Arrange
        let fixture = EmailClientFixture::new().await;
        fixture.mount_status(200).await;

        let subscriber_email = fixture.random_recipient();
        let subject = fixture.random_subject();
        let content = fixture.random_content();

        // Act
        let result = fixture
            .email_client
            .send_email(subscriber_email, &subject, &content, &content)
            .await;

        // Assert
        assert_ok!(result);
    }

    #[tokio::test]
    async fn send_email_fails_if_server_500() {
        // Arrange
        let fixture = EmailClientFixture::new().await;
        fixture.mount_status(500).await;

        let subscriber_email = fixture.random_recipient();
        let subject = fixture.random_subject();
        let content = fixture.random_content();

        // Act
        let result = fixture
            .email_client
            .send_email(subscriber_email, &subject, &content, &content)
            .await;

        // Assert
        assert_err!(result);
    }

    #[tokio::test]
    async fn send_email_fails_if_server_timeout() {
        // Arrange
        let fixture = EmailClientFixture::new().await;
        fixture
            .mount_delayed_status(200, std::time::Duration::from_secs(180))
            .await;

        let subscriber_email = fixture.random_recipient();
        let subject = fixture.random_subject();
        let content = fixture.random_content();

        // Act
        let result = fixture
            .email_client
            .send_email(subscriber_email, &subject, &content, &content)
            .await;

        // Assert
        assert_err!(result);
    }
}
