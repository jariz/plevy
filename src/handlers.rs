use crate::db::{Db, Entry};
use std::convert::Infallible;
use warp::{reject, Rejection};

pub async fn list(db: Db) -> Result<impl warp::Reply, Rejection> {
    match db.list() {
        Ok(list) => Ok(warp::reply::json::<Vec<Entry>>(
            &list
                .into_iter()
                .filter_map(Result::ok)
                .map(|(_, entry)| entry)
                .collect(),
        )),
        Err(_) => Err(reject::not_found()), // TODO: bad
    }
}

pub async fn add(db: Db, entry: Entry) -> Result<impl warp::Reply, Rejection> {
    match db.add(entry) {
        Ok(ino) => Ok(warp::reply::json::<u64>(&ino)),
        Err(_) => Err(reject::not_found()), // TODO: bad
    }
}

pub async fn get(ino: u64, db: Db) -> Result<impl warp::Reply, Rejection> {
    match db.get(ino) {
        Ok(entry) => Ok(warp::reply::json::<Option<Entry>>(&entry)),
        Err(_) => Err(reject::not_found()), // TODO: bad
    }
}
