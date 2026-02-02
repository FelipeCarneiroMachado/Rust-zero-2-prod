use crate::domain::{SubscriberEmail, SubscriberName};

#[derive(serde::Deserialize, Debug)]
pub struct NewSubscriber {
    pub email: SubscriberEmail,
    pub name: SubscriberName,
}
