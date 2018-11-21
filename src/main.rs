#![allow(unused, unknown_lints, proc_macro_derive_resolution_fallback)]
// if there is not proc_macro_derive_resolution_fallback imported, an error will occur
// like names from parent modules are not accessible without an explicit import
// note: for more information, see issue #50504 <https://github.com/rust-lang/rust/issues/50504>
// issue: https://github.com/diesel-rs/diesel/issues/1785

#[macro_use] extern crate actix_blog;

extern crate actix;
#[macro_use] extern crate actix_web;
extern crate env_logger;
#[macro_use] extern crate tera;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate serde_derive;
extern crate serde;
extern crate serde_json;
#[macro_use] extern crate diesel;
extern crate diesel_codegen;
#[macro_use] extern crate dotenv;
extern crate bcrypt;
extern crate futures;
#[macro_use] extern crate json;
extern crate chrono;

use std::{ env, path::Path, path::PathBuf};

use actix::prelude::*;
use actix_web::{server, App, middleware, http, fs};
use actix_web::middleware::session::{RequestSession, SessionStorage, CookieSessionBackend};

use diesel::prelude::*;
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};

use utils::utils::{DBPool, DBState, establish_connection};

pub mod models;
pub mod views;
pub mod utils;


static DATABASE_URL: &'static str = env!("DATABASE_URL");
type PgPool = Pool<ConnectionManager<PgConnection>>;


fn main() {
    env::set_var("RUST_LOG", "actix_web=debug"); // log level
    env_logger::init(); // init a log
    let sys = actix::System::new("actix-blog"); // start a system

    let addr = SyncArbiter::start(4, move || DBPool{ conn: establish_connection() });

    println!("path is {:?}", module_path!());
    println!("file name is {:?}", file!());
    println!("lin number is {:?}", line!());
    println!("column number is {:?}", column!());

    server::new(move || {
        vec![
            App::with_state(DBState{ db: addr.clone() })
                // enable logger
                .middleware(middleware::Logger::default())
                .middleware(SessionStorage::new( // session setup
                    CookieSessionBackend::signed(&[0; 32])
                    .secure(true)
                ))
                .handler("/static", fs::StaticFiles::new("static").unwrap()) // serve static files
                .scope("/admin", |scope| {
                    // admin path
                    scope.resource("/login/", |r| {
                        r.method(http::Method::GET).with(views::auth::login);
                        r.method(http::Method::POST).with(views::auth::handle_login);
                    })
                    .resource("/logout/", |r| {
                        r.method(http::Method::GET).with(views::auth::logout);
                    })
                    .resource("/register/", |r| {
                        r.method(http::Method::GET).with(views::auth::register);
                        r.method(http::Method::POST).with(views::auth::handle_registration);
                    })
                    .resource("/reset_password/", |r| {
                        r.method(http::Method::GET).with(views::auth::reset_password);
                        // r.method(http::Method::POST).with(views::auth::save_changed_password);
                    })
                    .resource("/dashboard/", |r| {
                        r.method(http::Method::GET).with(views::auth::dashboard);
                        r.method(http::Method::POST).with(views::auth::dashboard);
                    })
                    .resource("/all_post/", |r| {
                        r.method(http::Method::GET).with(views::auth::show_all_posts_by_author);
                    })
                    .resource("/write_post/", |r| {
                        r.method(http::Method::GET).with(views::auth::write_post);
                        r.method(http::Method::POST).with(views::auth::submit_post);
                    })
                    .resource("/{title}/", |r| {
                        r.method(http::Method::GET).with(views::auth::modify_post);
                        r.method(http::Method::POST).with(views::auth::save_modified_post);
                    })
                })
                // public path
                .scope("", |scope| {
                    scope
                    .resource("/index/", |r| {
                        r.method(http::Method::GET).with(views::post::show_all_posts);
                    })
                    .resource("/article/{title}/", |r| {
                        r.method(http::Method::GET).with(views::post::post_detail);
                    })
                    .resource("/page/{page_num}/", |r| {
                        r.method(http::Method::GET).with(views::post::pagination);
                    })
                    .resource("/about/", |r| {
                        r.method(http::Method::GET).with(views::post::about);
                    })
                    .resource("/contact/", |r| {
                        r.method(http::Method::GET).with(views::post::contact);
                    })
                    .resource("/user_likes/", |r| {
                        r.method(http::Method::POST).with(views::post::user_likes);
                    })
                })
        ]
    }).bind("192.168.31.204:8088").unwrap().start();
    println!("Started http server: 192.168.31.204:8088");
    let _ = sys.run();
}