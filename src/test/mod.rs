pub(self) mod test_auth_views;
pub(self) mod test_post_views;

use std::iter;
use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;

use failure;
use diesel::{ r2d2::{ ConnectionManager, Pool }, pg::PgConnection };
use crate::utils::utils::PgPool;

pub(self) fn generate_random_string(length: usize) -> String {
    iter::repeat(()).map(|()| thread_rng().sample(Alphanumeric))
                    .take(length)
                    .collect::<String>()
}

pub(self) fn test_db_pool() -> Result<PgPool, failure::Error> {
    dotenv::dotenv().ok();
    let database_url = dotenv::var("TEST_DATABASE_URL")?;
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = Pool::new(manager)?;
    Ok(pool)
}