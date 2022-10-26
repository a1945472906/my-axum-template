// use super::super::utils::global;
use sqlx::{postgres::PgPoolOptions, Error, Pool, Postgres, Transaction};
use std::marker::PhantomData;
use std::time::Duration;
#[derive(Clone)]
pub struct DB<T> {
    pub pool: Pool<Postgres>,
    _marker: PhantomData<T>,
}

#[allow(unused)]
impl<T> DB<T> {
    pub async fn new(db_url: &'static str) -> Self {
        let pool = PgPoolOptions::new()
            .max_connections(32)
            .acquire_timeout(Duration::from_secs(5))
            .connect(db_url)
            .await
            .expect("can not connect to databsae");
        DB {
            pool,
            _marker: PhantomData,
        }
    }
    pub async fn transtion(&self) -> Result<Transaction<'static, Postgres>, Error> {
        self.pool.begin().await
    }
}
