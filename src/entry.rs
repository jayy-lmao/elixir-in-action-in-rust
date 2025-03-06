use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct TodoEntry {
    pub date: NaiveDate,
    pub title: String,
}
