use std::collections::VecDeque;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc, Mutex,
};
use std::time::Instant;
use warp::Filter;

#[derive(Debug)]
pub struct Node {
    pub id: usize,
    pub pop_at: Instant,
}

impl Node {
    pub fn new(id_counter: IdCounter, pop_at: Instant) -> Self {
        Self {
            id: id_counter.fetch_add(1, Ordering::Relaxed),
            pop_at,
        }
    }
}

pub type Queue = Arc<Mutex<VecDeque<Node>>>;
pub type IdCounter = Arc<AtomicUsize>;

pub fn new_queue() -> Queue {
    Arc::new(Mutex::new(VecDeque::new()))
}

pub fn with_queue(
    q: Queue,
) -> impl Filter<Extract = (Queue,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || q.clone())
}

pub fn new_counter() -> IdCounter {
    Arc::new(AtomicUsize::new(0))
}

pub fn with_counter(
    id_counter: IdCounter,
) -> impl Filter<Extract = (IdCounter,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || id_counter.clone())
}

mod filters {
    use crate::{handlers, with_counter, with_queue, IdCounter, Queue};
    use warp::Filter;

    pub fn enqueue(
        q: Queue,
        id_counter: IdCounter,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("enqueue" / u64)
            .and(warp::post())
            .and(with_queue(q))
            .and(with_counter(id_counter))
            .and_then(handlers::enqueue)
    }

    pub fn status(
        q: Queue,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("status")
            .and(warp::get())
            .and(with_queue(q))
            .and_then(handlers::status)
    }

    pub fn root() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path::end().map(|| "Hello, World at root!")
    }

    pub fn set(
        q: Queue,
        id_counter: IdCounter,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        root().or(enqueue(q.clone(), id_counter)).or(status(q))
    }
}

mod handlers {
    use std::convert::Infallible;
    use std::sync::atomic::Ordering;
    use std::time::{Duration, Instant};

    use crate::{IdCounter, Node, Queue};

    pub async fn enqueue(
        time_in_queue: u64,
        q: Queue,
        id_counter: IdCounter,
    ) -> Result<impl warp::Reply, Infallible> {
        let mut q = q.lock().unwrap();

        let now = Instant::now();
        while let Some(node) = q.front() {
            if node.pop_at > now {
                break;
            }

            q.pop_front();
        }

        let duration = Duration::from_millis(time_in_queue);
        let pop_at = now + duration;
        let id = id_counter.fetch_add(1, Ordering::Relaxed);

        q.push_back(Node { id, pop_at });

        println!("added: {id}");

        Ok(format!(
            "added item with duration: {}",
            duration.as_millis()
        ))
    }

    pub async fn status(q: Queue) -> Result<impl warp::Reply, Infallible> {
        let mut queue = q.lock().unwrap();

        let now = Instant::now();
        while let Some(node) = queue.front() {
            if node.pop_at > now {
                break;
            }

            queue.pop_front();
        }

        Ok(format!("{:?}", queue.len()))
    }
}

#[tokio::main]
async fn main() {
    let q = new_queue();
    let id_counter = new_counter();
    let routes = filters::set(q, id_counter);
    println!("running on port 3000 baby!!! time to shine");
    warp::serve(routes).run(([127, 0, 0, 1], 3000)).await;
}
