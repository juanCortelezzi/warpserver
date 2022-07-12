mod queue;

#[tokio::main]
async fn main() {
    let q = queue::new();
    let routes = filters::set(q);
    warp::serve(routes).run(([127, 0, 0, 1], 3000)).await;
}

mod filters {
    use crate::queue::Queue;
    use crate::{handlers, queue};
    use warp::Filter;

    pub fn enqueue(
        q: Queue,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path!("enqueue" / i64)
            .and(warp::post())
            .and(queue::with_queue(q))
            .and_then(handlers::enqueue)
    }

    pub fn status(
        q: Queue,
    ) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
        warp::path("status")
            .and(warp::get())
            .and(queue::with_queue(q))
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
    use chrono::{DateTime, Utc};
    use warp::hyper::StatusCode;

    use crate::queue::{self, Response};
    use std::convert::Infallible;

    pub async fn enqueue(duration: i64, q: queue::Queue) -> Result<impl warp::Reply, Infallible> {
        queue::clean(q.clone());

        let duration = chrono::Duration::milliseconds(duration);
        let res = Response::new(duration);

        q.lock().unwrap().push_back(res);

        Ok(StatusCode::OK)
    }

    pub async fn status(q: queue::Queue) -> Result<impl warp::Reply, Infallible> {

        let q = q.lock().unwrap();

        let res: Vec<(i64, DateTime<Utc>)> = q.iter().map(|i| (i.duration, i.delete_time)).collect();

        Ok(warp::reply::json(&res))
    }
}
