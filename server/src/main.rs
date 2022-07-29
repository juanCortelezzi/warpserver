use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use warp::Filter;

#[derive(Debug)]
pub struct Node {
    pub id: usize,
    pub duration_as_millis: u128,
}

impl Node {
    pub fn new(id_counter: IdCounter, duration_as_millis: u128) -> Self {
        Self {
            id: id_counter.fetch_add(1, Ordering::Relaxed),
            duration_as_millis,
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
    use std::time::Duration;

    use crate::{IdCounter, Node, Queue};

    pub async fn enqueue(
        time_in_queue: u64,
        q: Queue,
        id_counter: IdCounter,
    ) -> Result<impl warp::Reply, Infallible> {
        let duration = Duration::from_millis(time_in_queue);

        tokio::spawn(async move {
            let id = id_counter.fetch_add(1, Ordering::Relaxed);
            {
                q.lock().unwrap().push_back(Node {
                    id,
                    duration_as_millis: duration.as_millis(),
                });
            }

            tokio::time::sleep(duration).await;

            let mut q = q.lock().unwrap();
            let node = q.front().expect("inserted node to be in queue");

            assert!(
                node.id == id,
                "id should be the same inQueue={} lookingFor={}",
                node.id,
                id
            );

            q.pop_front();
        });

        Ok(format!(
            "added item with duration: {}",
            duration.as_millis()
        ))
    }

    pub async fn status(q: Queue) -> Result<impl warp::Reply, Infallible> {
        let queue = q.lock().unwrap();
        Ok(format!("items in queue: {:?}", queue))
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
