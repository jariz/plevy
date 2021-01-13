use crate::db::Db;
use crate::handlers;
use std::convert::Infallible;
use warp::Filter;

pub async fn serve(db: Db) {
    let routes = all(db);
    warp::serve(routes).run(([127, 0, 0, 1], 3000)).await;
}

pub fn all(db: Db) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    get(db.clone()).or(list(db.clone())).or(add(db.clone()))
}

fn list(db: Db) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("entries")
        .and(warp::get())
        .and(with_db(db))
        .and_then(handlers::list)
}

fn add(db: Db) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path("entries")
        .and(warp::post())
        .and(with_db(db))
        .and(warp::body::json())
        .and_then(handlers::add)
}

fn get(db: Db) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::path!("entries" / u64)
        .and(warp::get())
        .and(with_db(db))
        .and_then(handlers::get)
}

fn with_db(db: Db) -> impl Filter<Extract = (Db,), Error = Infallible> + Clone {
    warp::any().map(move || db.clone())
}
