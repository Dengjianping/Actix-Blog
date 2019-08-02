use dotenv::dotenv;
use failure;
use diesel::{ r2d2::{ ConnectionManager, Pool }, pg::PgConnection };
use lazy_static::lazy_static;
use openssl::ssl::{ SslMethod, SslAcceptor, SslFiletype, SslAcceptorBuilder };
use std::{ fs::File, io::{ BufReader, prelude::* } };

pub(crate) type PgPool = Pool<ConnectionManager<PgConnection>>;

#[derive(Debug)]
pub(crate) enum Status {
    Failure,
    Success,
}

impl PartialEq for Status {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Status::Success, Status::Success) => true,
            _ => false,
        }
    }
}

lazy_static! {
    pub(crate) static ref COMPILED_TEMPLATES: tera::Tera = {
        tera::Tera::new(concat!(env!("CARGO_MANIFEST_DIR"), "/templates/**/*")).unwrap()
    };
}

pub(crate) fn db_pool() -> Result<PgPool, failure::Error> {
    dotenv().ok();
    let database_url = dotenv::var("DATABASE_URL")?;
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = Pool::new(manager)?;
    Ok(pool)
}

// enable http2/s
#[allow(dead_code)]
pub(crate) fn load_ssl() -> Result<SslAcceptorBuilder, failure::Error> {
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls())?;
    builder.set_private_key_file("ssl_keys/server.pem", SslFiletype::PEM)?;
    builder.set_certificate_chain_file("ssl_keys/crt.pem")?;
    Ok(builder)
}

// read project config file
pub(crate) fn blog_config() -> Result<toml::Value, failure::Error> {
    let config = File::open(concat!(env!("CARGO_MANIFEST_DIR"),"/actix_blog.toml"))?;
    let mut buff = BufReader::new(config);
    let mut contents = String::new();
    buff.read_to_string(&mut contents)?;
    
    let value = contents.parse::<toml::Value>()?;
    Ok(value)
}