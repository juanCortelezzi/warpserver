use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use warp::Filter;

pub type Queue = Arc<Mutex<VecDeque<u128>>>;

pub fn new_queue() -> Queue {
    Arc::new(Mutex::new(VecDeque::new()))
}

pub fn with_queue(
    q: Queue,
) -> impl Filter<Extract = (Queue,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || q.clone())
}

mod filters {
    use crate::{handlers, Queue, with_queue };
    use warp::Filter;

    pub fn enqueue(
        q: Queue,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("enqueue" / u64)
            .and(warp::post())
            .and(with_queue(q))
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
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        root()
            .or(enqueue(q.clone()))
            .or(status(q))
    }
}

mod handlers {
    use std::convert::Infallible;
    use std::time::Duration;

    use crate::Queue;

    pub async fn enqueue(time_in_queue: u64, q: Queue) -> Result<impl warp::Reply, Infallible> {
        let duration = Duration::from_millis(time_in_queue);

        tokio::spawn(async move {
            tokio::time::sleep(duration).await;
            q.lock().unwrap().push_back(duration.as_millis());
        });

        Ok(format!("added item with duration: {}", duration.as_millis()))
    }

    pub async fn status(q: Queue) -> Result<impl warp::Reply, Infallible> {
       Ok(format!("items in queue: {}", q.lock().unwrap().len()))
    }
}

#[tokio::main]
async fn main() {
    let q = new_queue();
    let routes = filters::set(q);
    warp::serve(routes).run(([127, 0, 0, 1], 3000)).await;
}

