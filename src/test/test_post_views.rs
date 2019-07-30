/// ```rust, no_run

use actix_web::{ test, web, App, http::header, http };
use actix_files as fs;
use actix_http::cookie::Cookie;
use actix_session::CookieSession;
use actix_service::Service;
use bytes::Bytes;
use log::{ info, debug };
use serde::{Serialize, Deserialize};

use crate::views;
use crate::utils::utils::db_pool;
use crate::models::comment::CreateComment;

use super::service_on;

#[test]
fn test_index_get() {
    let mut app = test::init_service(App::new().data(db_pool().unwrap().clone())
        .service(fs::Files::new("/static", "static/").show_files_listing())
        .service(
            web::scope("/").service(web::resource("").route(web::get().to_async(views::post::show_all_posts)))
        )
    );
    
    let req = test::TestRequest::get().uri("/").to_request();
    let resp = test::block_on(app.call(req)).unwrap();
    assert!(resp.status().is_success());
}

#[test]
fn test_index_post() {
    let mut app = test::init_service(App::new().data(db_pool().unwrap().clone())
        .service(fs::Files::new("/static", "static/").show_files_listing())
        .service(
            web::scope("/").service(web::resource("").route(web::get().to_async(views::post::show_all_posts)))
        )
    );
    
    let req = test::TestRequest::post().uri("/").to_request();
    let resp = test::block_on(app.call(req)).unwrap();
    assert!(resp.status().is_client_error());
}

#[test]
fn test_about() {
    let mut app = test::init_service(App::new()
        .service(fs::Files::new("/static", "static/").show_files_listing())
        .service(
            web::scope("/").service(web::resource("/about/").route(web::get().to_async(views::post::about)))
        )
    );
    
    let req = test::TestRequest::get().uri("/about/").to_request();
    let resp = test::block_on(app.call(req)).unwrap();
    assert!(resp.status().is_success());
}

#[test]
fn test_contact() {
    let mut app = test::init_service(App::new()
        .service(fs::Files::new("/static", "static/").show_files_listing())
        .service(
            web::scope("/").service(web::resource("/contact/").route(web::get().to_async(views::post::contact)))
        )
    );
    
    let req = test::TestRequest::get().uri("/contact/").to_request();
    let resp = test::block_on(app.call(req)).unwrap();
    assert!(resp.status().is_success());
}

#[test]
fn test_all_post() {
    let mut app = test::init_service(App::new().data(db_pool().unwrap().clone())
        .service(fs::Files::new("/static", "static/").show_files_listing())
        .service(
            web::scope("/").service(web::resource("/all_posts/").route(web::get().to_async(views::post::all_posts)))
        )
    );
    
    let req = test::TestRequest::get().uri("/all_posts/").to_request();
    let resp = test::block_on(app.call(req)).unwrap();
    assert!(resp.status().is_success());
}

#[test]
fn test_all_pagination() {
    let mut app = test::init_service(App::new().data(db_pool().unwrap().clone())
        .service(fs::Files::new("/static", "static/").show_files_listing())
        .service(
            web::scope("/").service(web::resource("/page/{page_num}/").route(web::get().to_async(views::post::pagination)))
        )
    );
    
    // let req = test::TestRequest::get().uri("/page/{page_num}/").param("page_num", "2").to_request();
    let req = test::TestRequest::get().uri("/page/1/").to_request();
    let resp = test::block_on(app.call(req)).unwrap();
    assert!(resp.status().is_success());
}

#[test]
// #[should_panic(expected = "page not found")]
fn test_all_pagination_failure() {
    let mut app = test::init_service(App::new().data(db_pool().unwrap().clone())
        .service(fs::Files::new("/static", "static/").show_files_listing())
        .service(
            web::scope("/").service(web::resource("/page/{page_num}/").route(web::get().to_async(views::post::pagination)))
        )
    );
    
    // let req = test::TestRequest::get().uri("/page/{page_num}/").param("page_num", "2").to_request();
    let req = test::TestRequest::get().uri("/page/100/").to_request();
    let resp = test::block_on(app.call(req)).unwrap();
    assert!(resp.status().is_success());
}

#[test]
fn test_add_comment() {
    let mut app = test::init_service(App::new().data(db_pool().unwrap().clone())
        .wrap(CookieSession::signed(&[0; 32]).secure(false))
        .service(fs::Files::new("/static", "static/").show_files_listing())
        .service(
            web::scope("/").service(web::resource("/add_comment/").route(web::post().to_async(views::post::add_comment)))
        )
    );
    
    let comment = CreateComment {
        comment: "good post!".to_owned(),
        username: "Bob".to_owned(),
        email: "djptux@gmail.com".to_owned(),
    };
    let cookie = Cookie::new("article_id", "1");
    
    let req = test::TestRequest::post().uri("/add_comment/")
                                       .header(header::CONTENT_TYPE, "application/json")
                                       .set_json(&comment)
                                       .cookie(cookie)
                                       .to_request();
    let result: bool = test::read_response_json(&mut app, req);
    assert_eq!(result, true);
    // let result = test::read_response(&mut app, req);
    // let resp = test::call_service(&mut app, req);
    // let result = test::read_body(resp);
    // let resp = test::block_on(app.call(req)).unwrap();
    // assert!(resp.status().is_success());
    // assert_eq!(resp.status().is_success(), true);
    // assert_eq!(resp.status(), http::StatusCode::OK);
    // assert_eq!(resp, Bytes::from_static(b"true"));
}

#[test]
fn test_post_detail() {
    let mut app = test::init_service(App::new().data(db_pool().unwrap().clone())
        .wrap(CookieSession::signed(&[0; 32]).secure(false))
        .service(fs::Files::new("/static", "static/").show_files_listing())
        .service(
            web::scope("/").service(web::resource("/article/{title}/").route(web::get().to_async(views::post::post_detail)))
        )
    );
    
    // let cookie = Cookie::new("article_id", "1");
    //let req = test::TestRequest::get().uri("/article/Cow%20in%20Rust/").to_request();
    let req = test::TestRequest::get().uri("/article/").param("title", "Cow in Rust").to_request();
    let resp = test::block_on(app.call(req)).unwrap();
    assert!(resp.status().is_success());
}

#[test]
fn test_search() {
    let mut app = test::init_service(App::new().data(db_pool().unwrap().clone())
        .service(fs::Files::new("/static", "static/").show_files_listing())
        .service(
            web::scope("/").service(web::resource("/search/").route(web::post().to_async(views::post::search)))
        )
    );
    
    new_struct!(Search, pub, [Debug, Clone, Serialize, Deserialize], (key_word=>String));
    let search = Search { key_word: "python".to_owned() };
    let req = test::TestRequest::post().uri("/search/").set_json(&search).to_request();
    let resp = test::block_on(app.call(req)).unwrap();
    assert!(resp.status().is_success());
}