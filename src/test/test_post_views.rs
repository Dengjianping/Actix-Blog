/// ```rust, no_run

use actix_web::{ test, web, App, http::header, http };
use actix_files as fs;
use actix_session::CookieSession;
use actix_service::Service;
use bytes::Bytes;
use chrono::Utc;
use serde::{ Serialize, Deserialize };

use crate::views;
use crate::models::comment::CreateComment;
use crate::models::contact::CreateContact;
use crate::models::post::{ NewPost, PostOperation };
use crate::utils::utils::Status;
use super::{ generate_random_string, test_db_pool };


#[test]
fn test_index() {
    let mut app = test::init_service(App::new().data(test_db_pool().unwrap().clone())
        .service(fs::Files::new("/static", "static/").show_files_listing())
        .service(
            web::scope("/").service(web::resource("").route(web::get().to_async(views::post::show_all_posts)))
        )
    );
    
    let req = test::TestRequest::get().uri("/").to_request();
    let resp = test::block_on(app.call(req)).unwrap();
    assert_eq!(resp.status(), http::StatusCode::OK);
}

#[test]
#[ignore]
#[should_panic(expected = "this page is unimplemented.")]
fn test_page_404() {
    let mut app = test::init_service(App::new()
        .service(fs::Files::new("/static", "static/").show_files_listing())
        .service(
            web::scope("/").service(web::resource("/not_found/").route(web::get().to_async(views::post::page_404)))
        )
    );
    
    let req = test::TestRequest::get().uri("/not_found/").to_request();
    let resp = test::block_on(app.call(req)).unwrap();
    assert_eq!(resp.status(), http::StatusCode::OK);
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
    assert_eq!(resp.status(), http::StatusCode::OK);
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
    assert_eq!(resp.status(), http::StatusCode::OK);
}

#[test]
fn test_all_post() {
    let mut app = test::init_service(App::new().data(test_db_pool().unwrap().clone())
        .service(fs::Files::new("/static", "static/").show_files_listing())
        .service(
            web::scope("/").service(web::resource("/all_posts/").route(web::get().to_async(views::post::all_posts)))
        )
    );
    
    let req = test::TestRequest::get().uri("/all_posts/").to_request();
    let resp = test::block_on(app.call(req)).unwrap();
    assert_eq!(resp.status(), http::StatusCode::OK);
}

#[test]
fn test_all_pagination() {
    let mut app = test::init_service(App::new().data(test_db_pool().unwrap().clone())
        .service(fs::Files::new("/static", "static/").show_files_listing())
        .service(
            web::scope("/").service(web::resource("/page/{page_num}/").route(web::get().to_async(views::post::pagination)))
        )
    );
    
    let req = test::TestRequest::get().uri("/page/1/").to_request();
    let resp = test::block_on(app.call(req)).unwrap();
    assert_eq!(resp.status(), http::StatusCode::OK);
}

#[test]
fn test_all_pagination_failure() {
    let mut app = test::init_service(App::new().data(test_db_pool().unwrap().clone())
        .service(fs::Files::new("/static", "static/").show_files_listing())
        .service(
            web::scope("/").service(web::resource("/page/{page_num}/").route(web::get().to_async(views::post::pagination)))
        )
    );
    
    // insert 4 posts to database at least for testing this case
    // due to each page hasing 4 posts to show there.
    let db = web::Data::new(test_db_pool().unwrap().clone());
    (0..4).for_each(|_| {
        let new_post = NewPost {
            title: generate_random_string(10),
            slug: generate_random_string(5),
            body: generate_random_string(40),
            publish: Some(Utc::now().naive_utc()),
            created: Some(Utc::now().naive_utc()),
            updated: Some(Utc::now().naive_utc()),
            status: "publish".to_owned(),
            user_id: 1,
            likes: 0,
        };
        match PostOperation::insert_post(&new_post, &db) {
            Ok(lhs) => assert_eq!(lhs, Status::Success),
            _ => assert!(false),
        }
    });
    
    let req = test::TestRequest::get().uri("/page/100/").to_request();
    let resp = test::block_on(app.call(req)).unwrap();
    assert_eq!(resp.status(), http::StatusCode::OK);
}

#[test]
fn test_add_contact() {
    let mut app = test::init_service(App::new().data(test_db_pool().unwrap().clone())
        .service(fs::Files::new("/static", "static/").show_files_listing())
        .service(
            web::scope("/").service(web::resource("/add_contact/").route(web::post().to_async(views::post::add_contact)))
        )
    );
    
    let new_contact = CreateContact {
        tourist_name: "jamie".to_owned(),
        email: "example.bob@actix.com".to_owned(),
        message: "I like you content".to_owned(),
    };
    let req = test::TestRequest::post().uri("/add_contact/")
                                       .header(header::CONTENT_TYPE, "application/json")
                                       .set_json(&new_contact)
                                       .to_request();
    let result: bool = test::read_response_json(&mut app, req);
    assert_eq!(result, true);
}

#[test]
fn test_add_comment() {
    let mut app = test::init_service(App::new().data(test_db_pool().unwrap().clone())
        .wrap(CookieSession::signed(&[0; 32]).name("post_session").secure(false))
        .service(fs::Files::new("/static", "static/").show_files_listing())
        .service(
            web::scope("/").service(web::resource("/article/{title}/").route(web::get().to_async(views::post::post_detail)))
                           .service(web::resource("/add_comment/").route(web::post().to_async(views::post::add_comment)))
        )
    );
    
    // set session
    let req = test::TestRequest::get().uri("/article/python/").to_request();
    let resp = test::block_on(app.call(req)).unwrap();
    
    let cookie = resp.response().cookies().find(|c| c.name() == "post_session").clone();
    assert!(cookie.is_some());
    
    let comment = CreateComment {
        comment: "good post!".to_owned(),
        username: "Bob".to_owned(),
        email: "djptux@gmail.com".to_owned(),
    };
    let req = test::TestRequest::post().uri("/add_comment/")
                                       .header(header::CONTENT_TYPE, "application/json")
                                       .set_json(&comment)
                                       .cookie(cookie.unwrap())
                                       .to_request();
    let result: bool = test::read_response_json(&mut app, req);
    assert_eq!(result, true);
}

#[test]
fn test_user_likes() {
    let mut app = test::init_service(App::new().data(test_db_pool().unwrap().clone())
        .wrap(CookieSession::signed(&[0; 32]).name("post_session").secure(false))
        .service(fs::Files::new("/static", "static/").show_files_listing())
        .service(
            web::scope("/").service(web::resource("/article/{title}/").route(web::get().to_async(views::post::post_detail)))
                           .service(web::resource("/user_likes/").route(web::post().to_async(views::post::user_likes)))
        )
    );
    
    // set session
    let req = test::TestRequest::get().uri("/article/python/").to_request();
    let resp = test::block_on(app.call(req)).unwrap();
    
    let cookie = resp.response().cookies().find(|c| c.name() == "post_session").clone();
    assert!(cookie.is_some());
    
    new_struct!(Like, pub, [Debug, Clone, Serialize, Deserialize], (likes_count=>i32));
    let likes = Like { likes_count: 6 };
    let req = test::TestRequest::post().uri("/user_likes/")
                                       .header(header::CONTENT_TYPE, "application/json")
                                       .set_json(&likes)
                                       .cookie(cookie.unwrap())
                                       .to_request();
    let result: bool = test::read_response_json(&mut app, req);
    assert_eq!(result, true);
}

#[test]
fn test_post_detail() {
    let mut app = test::init_service(App::new().data(test_db_pool().unwrap().clone())
        .wrap(CookieSession::signed(&[0; 32]).secure(false))
        .service(fs::Files::new("/static", "static/").show_files_listing())
        .service(
            web::scope("/").service(web::resource("/article/{title}/").route(web::get().to_async(views::post::post_detail)))
        )
    );
    
    let req = test::TestRequest::get().uri("/article/Cow%20in%20Rust/").to_request();
    let resp = test::block_on(app.call(req)).unwrap();
    assert_eq!(resp.status(), http::StatusCode::OK);
}

#[test]
fn test_search() {
    let mut app = test::init_service(App::new().data(test_db_pool().unwrap().clone())
        .service(fs::Files::new("/static", "static/").show_files_listing())
        .service(
            web::scope("/").service(web::resource("/search/").route(web::post().to_async(views::post::search)))
        )
    );
    
    new_struct!(Search, pub, [Debug, Clone, Serialize, Deserialize], (key_word=>String));
    let req = test::TestRequest::post()
                .uri("/search/")
                .header(header::CONTENT_LENGTH, "13")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .set_payload(Bytes::from_static(b"key_word=rust"))
                .to_request();
    
    /*
    let resp = test::call_service(&mut app, req);
    let result = test::read_body(resp);
    assert_eq!(result, Bytes::from_static(b"welcome!"));
    */
    
    let resp = test::block_on(app.call(req)).unwrap();
    assert_eq!(resp.status(), http::StatusCode::OK);
}