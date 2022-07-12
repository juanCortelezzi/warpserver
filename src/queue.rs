use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use warp::Filter;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Response {
    pub duration: i64,
    pub current_time: DateTime<Utc>,
    pub delete_time: DateTime<Utc>,
}

impl Response {
    pub fn new(duration: chrono::Duration) -> Self {
        let now = chrono::Utc::now();
        Self {
            duration: duration.num_milliseconds(),
            delete_time: now + duration,
            current_time: now,
        }
    }
}

pub type Queue = Arc<Mutex<VecDeque<Response>>>;

pub fn new() -> Queue {
    Arc::new(Mutex::new(VecDeque::new()))
}

pub fn clean(q: Queue) {
    let mut q = q.lock().unwrap();

    let now = chrono::Utc::now();

    while q.front().is_some() && q.front().unwrap().delete_time < now {
        q.pop_front();
    }
}

pub fn with_queue(
    q: Queue,
) -> impl Filter<Extract = (Queue,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || q.clone())
}
