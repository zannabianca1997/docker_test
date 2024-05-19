use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "bindgen", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
/// A stored message
pub struct StoredMessage {
    /// The time this message was received by the server
    pub time: DateTime<Utc>,
    /// The user that posted this message
    pub user: String,
    /// The message posted
    pub content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "bindgen", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
/// A message
pub struct Message {
    /// The user that posted this message
    pub user: String,
    /// The message posted
    pub content: String,
}

#[derive(Debug, Clone, Serialize)]
#[cfg_attr(feature = "bindgen", derive(schemars::JsonSchema))]
#[serde(deny_unknown_fields)]
/// A locked state of the board, ready to be serialized
pub struct Board {
    /// Title of the server
    pub title: String,
    /// The time the board was locked
    pub time: DateTime<Utc>,
    /// The time the server was started
    pub started_at: DateTime<Utc>,
    /// The messages at that time
    pub messages: Vec<StoredMessage>,
}
