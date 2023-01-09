use chrono::{DateTime, Utc};

pub use get::*;
pub use post::*;
pub use search::*;
pub use suggest::*;

mod get;
mod post;
mod search;
mod suggest;

pub struct PageSummary {
    pub url: String,
    pub title: String,
    pub updated_at: String,
    pub summary: Option<String>,
}

pub struct PageInfo {
    pub title: String,
    pub updated_at: DateTime<Utc>,
    pub summary: Option<String>,
}
