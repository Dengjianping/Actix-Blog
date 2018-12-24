#![allow(unused, unknown_lints, proc_macro_derive_resolution_fallback)]
//#![warn(unused_imports, unused)]
// if there is not proc_macro_derive_resolution_fallback imported, an error will occur
// like names from parent modules are not accessible without an explicit import
// note: for more information, see issue #50504 <https://github.com/rust-lang/rust/issues/50504>
// issue: https://github.com/diesel-rs/diesel/issues/1785

// proc-macro still needs extern crate in 2018 edition,
// in diesel, like 'table_name' and 'belongs_to'
#[macro_use] extern crate diesel;


// because a macro named 'new_struct' is in utils, so it need be place this line above others module
// or they will not see that macro.
#[macro_use] pub mod utils;
pub mod models;
pub mod views;

use actix_web::{ server, App, middleware, http::{ self, NormalizePath }, fs,
                 middleware::session::{ SessionStorage, CookieSessionBackend }};
use chrono::prelude::*;
use crate::utils::utils::{ DBPool, DBState, establish_connection, blog_config, load_ssl };
use std::{ env, fs::File, path::Path };


fn main() {
    let config = blog_config().unwrap();
    
    let address = config["address"].as_str().unwrap();
    let port = config["port"].as_integer().unwrap();
    let mut workers = config["workers"].as_integer().unwrap() as usize;
    let log_level = config["log"].as_str().unwrap();

    env::set_var("RUST_LOG", format!("actix_web={}", log_level)); // log level
    env_logger::init(); // init a log
    let sys = actix::System::new("actix-blog"); // start a system
    
    workers = num_cpus::get();
    
    let addr = actix::SyncArbiter::start(workers, move || DBPool{ conn: establish_connection() });
    
    let blog_server = server::new(move || {
        vec![
            App::with_state(DBState{ db: addr.clone() })
                // enable logger
                .middleware(middleware::Logger::default())
                // .middleware(middleware::csrf::CsrfFilter::new())
                .middleware(SessionStorage::new( // session setup
                    CookieSessionBackend::signed(&[0; 32])
                    .secure(false) // cannot be set as true
                    //.max_age(Duration::from_secs(60 * 30)) // do not support std::time::Duration right now
                    .max_age(chrono::Duration::minutes(30)) // session will expire after half an hour
                ))
                .handler("/static", fs::StaticFiles::new("static").unwrap()) // serve static files
                .scope("/admin", |scope| {
                    // admin path
                    scope.default_resource(|r| r.h(NormalizePath::default())) // normalize the path
                    .resource("/login/", |r| {
                        r.method(http::Method::GET).with(views::auth::login);
                        r.method(http::Method::POST).with(views::auth::handle_login);
                        // r.method(http::Method::POST).with(views::auth::login);
                    })
                    .resource("/", |r| {
                        r.method(http::Method::GET).with(views::auth::redirect_admin);
                    })
                    .resource("/user_exist/", |r| {
                        r.method(http::Method::POST).with(views::auth::user_exist);
                    })
                    .resource("/email_exist/", |r| {
                        r.method(http::Method::POST).with(views::auth::email_exist);
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
                        r.method(http::Method::POST).with(views::auth::save_changed_password);
                    })
                    .resource("/dashboard/", |r| {
                        r.method(http::Method::GET).with(views::auth::dashboard);
                        r.method(http::Method::POST).with(views::auth::dashboard);
                    })
                    .resource("/about_self/", |r| {
                        r.method(http::Method::GET).with(views::auth::about_self);
                    })
                    .resource("/today_comments/", |r| {
                        r.method(http::Method::GET).with(views::auth::today_comments);
                    })
                    .resource("/all_guests_messages/", |r| {
                        r.method(http::Method::GET).with(views::auth::all_guests_messages);
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
                    scope.default_resource(|r| r.h(NormalizePath::default())) // normalize the path
                    .resource("/index/", |r| {
                        r.method(http::Method::GET).with(views::post::show_all_posts);
                    })
                    .resource("/", |r| {
                        r.method(http::Method::GET).with(views::post::redirect_index);
                    })
                    .resource("/article/{title}/", |r| {
                        r.method(http::Method::GET).with(views::post::post_detail);
                    })
                    .resource("/page/{page_num}/", |r| {
                        r.method(http::Method::GET).with(views::post::pagination);
                    })
                    .resource("/all_posts/", |r| {
                        r.method(http::Method::GET).with(views::post::all_posts);
                    })
                    .resource("/about/", |r| {
                        r.method(http::Method::GET).with(views::post::about);
                    })
                    .resource("/contact/", |r| {
                        r.method(http::Method::GET).with(views::post::contact);
                    })
                    .resource("/add_contact/", |r| {
                        r.method(http::Method::POST).with(views::post::add_contact);
                    })
                    .resource("/user_likes/", |r| {
                        r.method(http::Method::POST).with(views::post::user_likes);
                    })
                    .resource("/not_found/", |r| {
                        r.method(http::Method::POST).with(views::post::page_404);
                    })
                    .resource("/search/", |r| {
                        r.method(http::Method::POST).with(views::post::search);
                    })
                    .resource("/add_comment/", |r| {
                        r.method(http::Method::POST).with(views::post::add_comment);
                    })
                })
        ]
    });
    
    if cfg!(feature = "http2") {
        blog_server.bind_ssl(format!("{}:{}", &address, &port), load_ssl()).unwrap().start();
    } else {
        blog_server.bind(format!("{}:{}", &address, &port)).unwrap().start();
    }
    println!("Started http server: {}", format!("{}:{}", &address, &port));
    let _ = sys.run();
}