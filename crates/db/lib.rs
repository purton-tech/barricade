use std::str::FromStr;

pub use deadpool_postgres::{Pool, PoolError, Transaction};

pub use tokio_postgres::Error as TokioPostgresError;

pub fn create_pool(database_url: &str) -> deadpool_postgres::Pool {
    let config = tokio_postgres::Config::from_str(database_url).unwrap();
    let manager = deadpool_postgres::Manager::new(config, tokio_postgres::NoTls);

    deadpool_postgres::Pool::builder(manager).build().unwrap()
}
