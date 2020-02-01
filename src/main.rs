#[macro_use]
extern crate diesel;

use actix_files as fs;
use actix_session::CookieSession;
use actix_identity::{ CookieIdentityPolicy, IdentityService };
use actix_web::{ web, App, HttpServer, middleware };

#[macro_use]
mod utils;
mod views;
mod models;
mod error_types;
#[cfg(test)]
mod test;

use crate::utils::utils::{ db_pool, blog_config };

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    let config = &blog_config().unwrap()["production"];
    let address = config["address"].as_str().ok_or(failure::err_msg("no address in the section")).unwrap();
    let port = config["port"].as_integer().ok_or(failure::err_msg("no port in the section")).unwrap();
    let workers = config["workers"].as_integer().ok_or(failure::err_msg("no workers in the section")).unwrap() as usize;
    let log_level = config["log"].as_str().ok_or(failure::err_msg("no specified log level in the section")).unwrap();
    
    std::env::set_var("RUST_LOG", format!("actix_server={},actix_web={}", log_level, log_level)); // log level
    env_logger::init(); // init a log
    
    let pool = db_pool().expect("failed to open db connection");
    
    let blog_server = HttpServer::new( move || 
        App::new().data(pool.clone())
            .wrap(middleware::Logger::default())
            .wrap(middleware::NormalizePath::default())
            .wrap(
                CookieSession::signed(&[0; 32])
                    .name("post_session")
                    .path("/")
                    .secure(false)
                    .max_age(60 * 60i64)
            )
            .wrap(
                IdentityService::new(
                    CookieIdentityPolicy::new(&[0;32])
                        .name("admin")
                        .path("/admin")
                        .max_age(60 * 60i64)
                        .secure(false)
                )
            )
            // css, js files loading
            .service(fs::Files::new("/static", "static/").show_files_listing())
            .service(
                web::scope("/admin")
                    .service(web::resource("/").route(web::get().to(views::auth::redirect_admin)))
                    .service(web::resource("/login/").route(web::get().to(views::auth::login))
                                                     .route(web::post().to(views::auth::handle_login))
                    )
                    .service(web::resource("/user_exist/").route(web::post().to(views::auth::user_exist)))
                    .service(web::resource("/dashboard/").route(web::post().to(views::auth::dashboard))
                                                         .route(web::get().to(views::auth::dashboard))
                    )
                    .service(web::resource("/all_posts/").route(web::get().to(views::auth::show_all_posts_by_author)))
                    .service(web::resource("/today_comments/").route(web::get().to(views::auth::today_comments)))
                    .service(web::resource("/all_guests_messages/").route(web::get().to(views::auth::all_guests_messages)))
                    .service(web::resource("/about_self/").route(web::get().to(views::auth::about_self)))
                    .service(web::resource("/logout/").route(web::get().to(views::auth::logout)))
                    .service(web::resource("/write_post/").route(web::get().to(views::auth::write_post))
                                                          .route(web::post().to(views::auth::submit_post))
                    )
                    .service(web::resource("/register/").route(web::get().to(views::auth::register))
                                                        .route(web::post().to(views::auth::handle_registration))
                    )
                    .service(web::resource("/email_exist/").route(web::post().to(views::auth::email_exist)))
                    .service(web::resource("/reset_password/").route(web::get().to(views::auth::reset_password))
                                                              .route(web::post().to(views::auth::save_changed_password))
                    )
                    .service(web::resource("/{title}/").route(web::get().to(views::auth::modify_post))
                                                       .route(web::post().to(views::auth::save_modified_post))
                    )
            )
            .service(
                web::scope("/")
                    .service(web::resource("").route(web::get().to(views::post::show_all_posts)))
                    .service(web::resource("/index/").route(web::get().to(views::post::show_all_posts)))
                    .service(web::resource("/about/").route(web::get().to(views::post::about)))
                    .service(web::resource("/contact/").route(web::get().to(views::post::contact)))
                    .service(web::resource("/not_found/").route(web::get().to(views::post::page_404)))
                    .service(web::resource("/all_posts/").route(web::get().to(views::post::all_posts)))
                    .service(web::resource("/add_comment/").route(web::post().to(views::post::add_comment)))
                    .service(web::resource("/user_likes/").route(web::post().to(views::post::user_likes)))
                    .service(web::resource("/add_contact/").route(web::post().to(views::post::add_contact)))
                    .service(web::resource("/search/").route(web::post().to(views::post::search)))
                    .service(web::resource("/page/{page_num}/").route(web::get().to(views::post::pagination)))
                    .service(web::resource("/article/{title}/").route(web::get().to(views::post::post_detail)))
                    .service(web::resource("/category/{year}/").route(web::get().to(views::post::show_posts_by_year)))
            )
    )
    .workers(workers);
    
    // compile http1 capability if feature http1 enabled while http2 ignored
    #[cfg(feature = "http1")]
    {
        blog_server.bind(format!("{}:{}", &address, &port))?.run().await
    }
    
    #[cfg(feature = "http2")]
    {
        use crate::utils::utils::load_ssl;
        blog_server.bind_ssl(format!("{}:{}", &address, &port), load_ssl()?)?.run().await
    }
}
