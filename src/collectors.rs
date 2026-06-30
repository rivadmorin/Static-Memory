use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Event {
    pub timestamp: DateTime<Utc>,
    pub event_type: String,
    pub content: String,
    pub context: Option<String>,
}

pub trait Collector: Send {
    fn name(&self) -> &str;
}
