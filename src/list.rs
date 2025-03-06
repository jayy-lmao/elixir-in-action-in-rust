use std::collections::HashMap;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

use crate::entry::TodoEntry;

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TodoList {
    auto_id: u64,
    entries: HashMap<u64, TodoEntry>,
}
impl TodoList {
    pub fn new() -> Self {
        TodoList::default()
    }

    pub fn add_entry(&mut self, entry: TodoEntry) {
        self.entries.insert(self.auto_id, entry);
        self.auto_id += 1;
    }

    pub fn entries(&self, date: NaiveDate) -> Vec<&TodoEntry> {
        self.entries
            .values()
            .filter(|entry| entry.date == date)
            .collect()
    }
}
