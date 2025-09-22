use crate::domain::{Email, SubscriberName};

pub struct NewSubscriber {
    pub email: Email,
    pub name: SubscriberName,
}
