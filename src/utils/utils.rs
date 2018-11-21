use diesel::prelude::*;
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use dotenv::dotenv;

use actix::prelude::*;
use actix_web::{Error, HttpResponse, FutureResponse, AsyncResponder};
use futures::future::{Future, result};

use tera;

static DATABASE_URL: &'static str = env!("DATABASE_URL");
type PgPool = Pool<ConnectionManager<PgConnection>>;


lazy_static! {
    pub static ref compiled_templates: tera::Tera = {
        let mut t = compile_templates!(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*"));
        t
    };
}


pub struct DBPool {
    pub conn: PgConnection,
}

pub struct DBState {
    pub db: Addr<DBPool>,
}


// create a db connection
pub fn establish_connection() -> PgConnection {
    dotenv().ok();

    let database_url = DATABASE_URL;
    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}


// create a db pool
pub fn db_pool() -> PgPool {
    let manager = ConnectionManager::<PgConnection>::new(DATABASE_URL);
    Pool::new(manager).expect("failed to create a db pool")
}


// redirect
pub fn redirect(url: &str) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::TemporaryRedirect().header("Location", url).finish())
}


// async redirect
pub fn async_redirect(url: &str) -> FutureResponse<HttpResponse, Error> {
    result(Ok(HttpResponse::TemporaryRedirect().header("Location", url).finish())).responder()
}