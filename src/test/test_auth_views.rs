use actix_web::{ test, web, App, http::header, http };
use actix_files as fs;
use actix_identity::{ CookieIdentityPolicy, IdentityService };
use actix_service::Service;
use bytes::Bytes;
use serde::{ Serialize, Deserialize };

use crate::views;
use crate::utils::utils::db_pool;
use super::generate_random_string;


#[test]
fn test_login() {
    let mut app = test::init_service(App::new()
        .service(fs::Files::new("/static", "static/").show_files_listing())
        .service(
            web::scope("/admin").service(web::resource("/login/").route(web::get().to_async(views::auth::login)))
        )
    );
    
    let req = test::TestRequest::get().uri("/admin/login/").to_request();
    let resp = test::block_on(app.call(req)).unwrap();
    assert_eq!(resp.status(), http::StatusCode::OK);
}

#[test]
fn test_handle_login() {
    let mut app = test::init_service(App::new().data(db_pool().unwrap().clone())
        .wrap(
            IdentityService::new(
                CookieIdentityPolicy::new(&[0;32])
                    .name("admin")
                    .path("/admin")
                    .max_age(60i64)
                    .secure(false)
            )
        )
        .service(fs::Files::new("/static", "static/").show_files_listing())
        .service(
            web::scope("/admin")
                .service(web::resource("/login/").route(web::post().to_async(views::auth::handle_login))
                )
        )
    );
    
    let req = test::TestRequest::post()
                .uri("/admin/login/")
                .header(header::CONTENT_LENGTH, "41")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .set_payload(Bytes::from_static(b"username=jdeng&password=welcome"))
                .to_request();
    
    let resp = test::block_on(app.call(req)).unwrap();
    assert_eq!(resp.status(), http::StatusCode::TEMPORARY_REDIRECT);
}

#[test]
fn test_handle_login_failure() {
    let mut app = test::init_service(App::new().data(db_pool().unwrap().clone())
        .wrap(
            IdentityService::new(
                CookieIdentityPolicy::new(&[0;32])
                    .name("admin")
                    .path("/admin")
                    .max_age(60i64)
                    .secure(false)
            )
        )
        .service(fs::Files::new("/static", "static/").show_files_listing())
        .service(
            web::scope("/admin")
                .service(web::resource("/login/").route(web::post().to_async(views::auth::handle_login))
                )
        )
    );
    
    let req = test::TestRequest::post()
                .uri("/admin/login/")
                .header(header::CONTENT_LENGTH, "41")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .set_payload(Bytes::from_static(b"username=jdeng&password=123456"))
                .to_request();
    
    let resp = test::block_on(app.call(req)).unwrap();
    assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);
}

#[test]
fn test_register() {
    let mut app = test::init_service(App::new()
        .service(fs::Files::new("/static", "static/").show_files_listing())
        .service(
            web::scope("/admin").service(web::resource("/register/").route(web::get().to_async(views::auth::register)))
        )
    );
    
    let req = test::TestRequest::get().uri("/admin/register/").to_request();
    let resp = test::block_on(app.call(req)).unwrap();
    assert_eq!(resp.status(), http::StatusCode::OK);
}

#[test]
fn test_handle_registration() {
    let mut app = test::init_service(App::new().data(db_pool().unwrap().clone())
        .wrap(
            IdentityService::new(
                CookieIdentityPolicy::new(&[0;32])
                    .name("admin")
                    .path("/admin")
                    .max_age(60i64)
                    .secure(false)
            )
        )
        .service(fs::Files::new("/static", "static/").show_files_listing())
        .service(
            web::scope("/admin")
                .service(web::resource("/register/").route(web::post().to_async(views::auth::handle_registration))
                )
        )
    );
    
    let new_user = format!("username={}&password={}&first_name={}&last_name={}&email={}", 
                        generate_random_string(10), generate_random_string(8), generate_random_string(6),
                        generate_random_string(6), generate_random_string(15));
    let req = test::TestRequest::post()
                .uri("/admin/register/")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .set_payload(Bytes::from(new_user.as_bytes()))
                .to_request();
    
    let resp = test::block_on(app.call(req)).unwrap();
    assert_eq!(resp.status(), http::StatusCode::TEMPORARY_REDIRECT);
}

#[test]
fn test_handle_registration_failure() {
    let mut app = test::init_service(App::new().data(db_pool().unwrap().clone())
        .wrap(
            IdentityService::new(
                CookieIdentityPolicy::new(&[0;32])
                    .name("admin")
                    .path("/admin")
                    .max_age(60i64)
                    .secure(false)
            )
        )
        .service(fs::Files::new("/static", "static/").show_files_listing())
        .service(
            web::scope("/admin")
                .service(web::resource("/register/").route(web::post().to_async(views::auth::handle_registration))
                )
        )
    );
    
    let new_user = b"username=jdeng&password=welcome&first_name=Jianping&last_name=Deng&email=djptux@gmail.com";
    let req = test::TestRequest::post()
                .uri("/admin/register/")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .set_payload(Bytes::from_static(new_user))
                .to_request();
    
    let resp = test::block_on(app.call(req)).unwrap();
    assert_eq!(resp.status(), http::StatusCode::INTERNAL_SERVER_ERROR);
}

#[test]
fn test_user_exist() {
    let mut app = test::init_service(App::new().data(db_pool().unwrap().clone())
        .service(fs::Files::new("/static", "static/").show_files_listing())
        .service(
            web::scope("/admin").service(web::resource("/user_exist/").route(web::post().to_async(views::auth::user_exist)))
        )
    );
    
    new_struct!(UserExist, pub, [Debug, Clone, Serialize, Deserialize], (username=>String));
    let user_exist = UserExist { username: "jdeng".to_owned() };
    let req = test::TestRequest::post().uri("/admin/user_exist/")
                                       .header(header::CONTENT_TYPE, "application/json")
                                       .set_json(&user_exist)
                                       .to_request();
    let result: bool = test::read_response_json(&mut app, req);
    assert_eq!(result, true);
}

#[test]
fn test_user_not_exist() {
    let mut app = test::init_service(App::new().data(db_pool().unwrap().clone())
        .service(fs::Files::new("/static", "static/").show_files_listing())
        .service(
            web::scope("/admin").service(web::resource("/user_exist/").route(web::post().to_async(views::auth::user_exist)))
        )
    );
    
    new_struct!(UserExist, pub, [Debug, Clone, Serialize, Deserialize], (username=>String));
    let user_exist = UserExist { username: "Bob".to_owned() };
    let req = test::TestRequest::post().uri("/admin/user_exist/")
                                       .header(header::CONTENT_TYPE, "application/json")
                                       .set_json(&user_exist)
                                       .to_request();
    let result: bool = test::read_response_json(&mut app, req);
    assert_eq!(result, false);
}

#[test]
fn test_email_exist() {
    let mut app = test::init_service(App::new().data(db_pool().unwrap().clone())
        .service(fs::Files::new("/static", "static/").show_files_listing())
        .service(
            web::scope("/admin").service(web::resource("/email_exist/").route(web::post().to_async(views::auth::email_exist)))
        )
    );
    
    new_struct!(EmailExist, pub, [Debug, Clone, Serialize, Deserialize], (email=>String));
    let email_exist = EmailExist { email: "djptux@gmail.com".to_owned() };
    let req = test::TestRequest::post().uri("/admin/email_exist/")
                                       .header(header::CONTENT_TYPE, "application/json")
                                       .set_json(&email_exist)
                                       .to_request();
    let result: bool = test::read_response_json(&mut app, req);
    assert_eq!(result, true);
}

#[test]
fn test_email_not_exist() {
    let mut app = test::init_service(App::new().data(db_pool().unwrap().clone())
        .service(fs::Files::new("/static", "static/").show_files_listing())
        .service(
            web::scope("/admin").service(web::resource("/email_exist/").route(web::post().to_async(views::auth::email_exist)))
        )
    );
    
    new_struct!(EmailExist, pub, [Debug, Clone, Serialize, Deserialize], (email=>String));
    let email_exist = EmailExist { email: "bob@gmail.com".to_owned() };
    let req = test::TestRequest::post().uri("/admin/email_exist/")
                                       .header(header::CONTENT_TYPE, "application/json")
                                       .set_json(&email_exist)
                                       .to_request();
    let result: bool = test::read_response_json(&mut app, req);
    assert_eq!(result, false);
}

#[test]
fn test_dashboard_post() {
    let mut app = test::init_service(App::new().data(db_pool().unwrap().clone())
        .wrap(
            IdentityService::new(
                CookieIdentityPolicy::new(&[0;32])
                    .name("admin")
                    .path("/admin")
                    .max_age(60i64)
                    .secure(false)
            )
        )
        .service(fs::Files::new("/static", "static/").show_files_listing())
        .service(
            web::scope("/admin").service(web::resource("/login/").route(web::post().to_async(views::auth::handle_login)))
                                .service(web::resource("/dashboard/").route(web::post().to_async(views::auth::dashboard)))
        )
    );
    
    // before test getting dashboard, login is required due to setting identity.
    let req = test::TestRequest::post()
                .uri("/admin/login/")
                .header(header::CONTENT_LENGTH, "41")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .set_payload(Bytes::from_static(b"username=jdeng&password=welcome"))
                .to_request();
    let resp = test::block_on(app.call(req)).unwrap();
    assert_eq!(resp.status(), http::StatusCode::TEMPORARY_REDIRECT);
    
    // get identity
    let identity = resp.response().cookies().next().clone();
    assert!(identity.is_some());
    
    let req = test::TestRequest::post().uri("/admin/dashboard/").cookie(identity.unwrap()).to_request();
    let resp = test::block_on(app.call(req)).unwrap();
    assert_eq!(resp.status(), http::StatusCode::OK);
}

#[test]
fn test_dashboard_get() {
    let mut app = test::init_service(App::new().data(db_pool().unwrap().clone())
        .wrap(
            IdentityService::new(
                CookieIdentityPolicy::new(&[0;32])
                    .name("admin")
                    .path("/admin")
                    .max_age(60i64)
                    .secure(false)
            )
        )
        .service(fs::Files::new("/static", "static/").show_files_listing())
        .service(
            web::scope("/admin").service(web::resource("/login/").route(web::post().to_async(views::auth::handle_login)))
                                .service(web::resource("/dashboard/").route(web::get().to_async(views::auth::dashboard)))
        )
    );
    
    // before test getting dashboard, login is required due to setting identity.
    let req = test::TestRequest::post()
                .uri("/admin/login/")
                .header(header::CONTENT_LENGTH, "41")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .set_payload(Bytes::from_static(b"username=jdeng&password=welcome"))
                .to_request();
    let resp = test::block_on(app.call(req)).unwrap();
    assert_eq!(resp.status(), http::StatusCode::TEMPORARY_REDIRECT);
    
    // get identity
    let identity = resp.response().cookies().next().clone();
    assert!(identity.is_some());
    
    let req = test::TestRequest::get().uri("/admin/dashboard/").cookie(identity.unwrap()).to_request();
    let resp = test::block_on(app.call(req)).unwrap();
    assert_eq!(resp.status(), http::StatusCode::OK);
}

#[test]
fn test_logout() {
    let mut app = test::init_service(App::new().data(db_pool().unwrap().clone())
        .wrap(
            IdentityService::new(
                CookieIdentityPolicy::new(&[0;32])
                    .name("admin")
                    .path("/admin")
                    .max_age(60i64)
                    .secure(false)
            )
        )
        .service(fs::Files::new("/static", "static/").show_files_listing())
        .service(
            web::scope("/admin").service(web::resource("/login/").route(web::post().to_async(views::auth::handle_login)))
                                .service(web::resource("/logout/").route(web::get().to_async(views::auth::logout)))
        )
    );
    
    // before test getting dashboard, login is required due to setting identity.
    let req = test::TestRequest::post()
                .uri("/admin/login/")
                .header(header::CONTENT_LENGTH, "41")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .set_payload(Bytes::from_static(b"username=jdeng&password=welcome"))
                .to_request();
    let resp = test::block_on(app.call(req)).unwrap();
    assert_eq!(resp.status(), http::StatusCode::TEMPORARY_REDIRECT);
    
    // get identity
    let identity = resp.response().cookies().next().clone();
    assert!(identity.is_some());
    
    let req = test::TestRequest::get().uri("/admin/logout/").cookie(identity.unwrap()).to_request();
    let resp = test::block_on(app.call(req)).unwrap();
    assert_eq!(resp.status(), http::StatusCode::TEMPORARY_REDIRECT);
}

#[test]
fn test_about_self() {
    let mut app = test::init_service(App::new().data(db_pool().unwrap().clone())
        .wrap(
            IdentityService::new(
                CookieIdentityPolicy::new(&[0;32])
                    .name("admin")
                    .path("/admin")
                    .max_age(60i64)
                    .secure(false)
            )
        )
        .service(fs::Files::new("/static", "static/").show_files_listing())
        .service(
            web::scope("/admin").service(web::resource("/login/").route(web::post().to_async(views::auth::handle_login)))
                                .service(web::resource("/about_self/").route(web::get().to_async(views::auth::about_self)))
        )
    );
    
    // before test getting dashboard, login is required due to setting identity.
    let req = test::TestRequest::post()
                .uri("/admin/login/")
                .header(header::CONTENT_LENGTH, "41")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .set_payload(Bytes::from_static(b"username=jdeng&password=welcome"))
                .to_request();
    let resp = test::block_on(app.call(req)).unwrap();
    assert_eq!(resp.status(), http::StatusCode::TEMPORARY_REDIRECT);
    
    // get identity
    let identity = resp.response().cookies().next().clone();
    assert!(identity.is_some());
    
    let req = test::TestRequest::get().uri("/admin/about_self/").cookie(identity.unwrap()).to_request();
    let resp = test::block_on(app.call(req)).unwrap();
    assert_eq!(resp.status(), http::StatusCode::OK);
}

#[test]
fn test_show_all_posts_by_author() {
    let mut app = test::init_service(App::new().data(db_pool().unwrap().clone())
        .wrap(
            IdentityService::new(
                CookieIdentityPolicy::new(&[0;32])
                    .name("admin")
                    .path("/admin")
                    .max_age(60i64)
                    .secure(false)
            )
        )
        .service(fs::Files::new("/static", "static/").show_files_listing())
        .service(
            web::scope("/admin").service(web::resource("/login/").route(web::post().to_async(views::auth::handle_login)))
                                .service(web::resource("/all_posts/").route(web::get().to_async(views::auth::show_all_posts_by_author)))
        )
    );
    
    // before test getting dashboard, login is required due to setting identity.
    let req = test::TestRequest::post()
                .uri("/admin/login/")
                .header(header::CONTENT_LENGTH, "41")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .set_payload(Bytes::from_static(b"username=jdeng&password=welcome"))
                .to_request();
    let resp = test::block_on(app.call(req)).unwrap();
    assert_eq!(resp.status(), http::StatusCode::TEMPORARY_REDIRECT);
    
    // get identity
    let identity = resp.response().cookies().next().clone();
    assert!(identity.is_some());
    
    let req = test::TestRequest::get().uri("/admin/all_posts/").cookie(identity.unwrap()).to_request();
    let resp = test::block_on(app.call(req)).unwrap();
    assert_eq!(resp.status(), http::StatusCode::OK);
}

#[test]
fn test_today_comments() {
    let mut app = test::init_service(App::new().data(db_pool().unwrap().clone())
        .wrap(
            IdentityService::new(
                CookieIdentityPolicy::new(&[0;32])
                    .name("admin")
                    .path("/admin")
                    .max_age(60i64)
                    .secure(false)
            )
        )
        .service(fs::Files::new("/static", "static/").show_files_listing())
        .service(
            web::scope("/admin").service(web::resource("/login/").route(web::post().to_async(views::auth::handle_login)))
                                .service(web::resource("/today_comments/").route(web::get().to_async(views::auth::today_comments)))
        )
    );
    
    // before test getting dashboard, login is required due to setting identity.
    let req = test::TestRequest::post()
                .uri("/admin/login/")
                .header(header::CONTENT_LENGTH, "41")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .set_payload(Bytes::from_static(b"username=jdeng&password=welcome"))
                .to_request();
    let resp = test::block_on(app.call(req)).unwrap();
    assert_eq!(resp.status(), http::StatusCode::TEMPORARY_REDIRECT);
    
    // get identity
    let identity = resp.response().cookies().next().clone();
    assert!(identity.is_some());
    
    let req = test::TestRequest::get().uri("/admin/today_comments/").cookie(identity.unwrap()).to_request();
    let resp = test::block_on(app.call(req)).unwrap();
    assert_eq!(resp.status(), http::StatusCode::OK);
}

#[test]
fn test_all_guests_messages() {
    let mut app = test::init_service(App::new().data(db_pool().unwrap().clone())
        .wrap(
            IdentityService::new(
                CookieIdentityPolicy::new(&[0;32])
                    .name("admin")
                    .path("/admin")
                    .max_age(60i64)
                    .secure(false)
            )
        )
        .service(fs::Files::new("/static", "static/").show_files_listing())
        .service(
            web::scope("/admin").service(web::resource("/login/").route(web::post().to_async(views::auth::handle_login)))
                                .service(web::resource("/all_guests_messages/").route(web::get().to_async(views::auth::all_guests_messages)))
        )
    );
    
    // before test getting dashboard, login is required due to setting identity.
    let req = test::TestRequest::post()
                .uri("/admin/login/")
                .header(header::CONTENT_LENGTH, "41")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .set_payload(Bytes::from_static(b"username=jdeng&password=welcome"))
                .to_request();
    let resp = test::block_on(app.call(req)).unwrap();
    assert_eq!(resp.status(), http::StatusCode::TEMPORARY_REDIRECT);
    
    // get identity
    let identity = resp.response().cookies().next().clone();
    assert!(identity.is_some());
    
    let req = test::TestRequest::get().uri("/admin/all_guests_messages/").cookie(identity.unwrap()).to_request();
    let resp = test::block_on(app.call(req)).unwrap();
    assert_eq!(resp.status(), http::StatusCode::OK);
}

#[test]
fn test_write_post() {
    let mut app = test::init_service(App::new().data(db_pool().unwrap().clone())
        .wrap(
            IdentityService::new(
                CookieIdentityPolicy::new(&[0;32])
                    .name("admin")
                    .path("/admin")
                    .max_age(60i64)
                    .secure(false)
            )
        )
        .service(fs::Files::new("/static", "static/").show_files_listing())
        .service(
            web::scope("/admin").service(web::resource("/login/").route(web::post().to_async(views::auth::handle_login)))
                                .service(web::resource("/write_post/").route(web::get().to_async(views::auth::write_post)))
        )
    );
    
    // before test getting dashboard, login is required due to setting identity.
    let req = test::TestRequest::post()
                .uri("/admin/login/")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .set_payload(Bytes::from_static(b"username=jdeng&password=welcome"))
                .to_request();
    let resp = test::block_on(app.call(req)).unwrap();
    assert_eq!(resp.status(), http::StatusCode::TEMPORARY_REDIRECT);
    
    // get identity
    let identity = resp.response().cookies().next().clone();
    assert!(identity.is_some());
    
    let req = test::TestRequest::get().uri("/admin/write_post/").cookie(identity.unwrap()).to_request();
    let resp = test::block_on(app.call(req)).unwrap();
    assert_eq!(resp.status(), http::StatusCode::OK);
}

#[test]
fn test_submit_post() {
    let mut app = test::init_service(App::new().data(db_pool().unwrap().clone())
        .wrap(
            IdentityService::new(
                CookieIdentityPolicy::new(&[0;32])
                    .name("admin")
                    .path("/admin")
                    .max_age(60i64)
                    .secure(false)
            )
        )
        .service(fs::Files::new("/static", "static/").show_files_listing())
        .service(
            web::scope("/admin").service(web::resource("/login/").route(web::post().to_async(views::auth::handle_login)))
                                .service(web::resource("/write_post/").route(web::post().to_async(views::auth::submit_post)))
        )
    );
    
    // before test getting dashboard, login is required due to setting identity.
    let req = test::TestRequest::post()
                .uri("/admin/login/")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .set_payload(Bytes::from_static(b"username=jdeng&password=welcome"))
                .to_request();
    let resp = test::block_on(app.call(req)).unwrap();
    assert_eq!(resp.status(), http::StatusCode::TEMPORARY_REDIRECT);
    
    // get identity
    let identity = resp.response().cookies().next().clone();
    assert!(identity.is_some());
    
    let post = format!("title={}&slug={}&body={}&status=publish", 
                        generate_random_string(4), generate_random_string(4), generate_random_string(40));
    let req = test::TestRequest::post()
                .uri("/admin/write_post/")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .set_payload(Bytes::from(post.as_bytes()))
                .cookie(identity.unwrap())
                .to_request();
    let resp = test::block_on(app.call(req)).unwrap();
    //let resp = test::block_on(app.call(req)).unwrap();
    assert_eq!(resp.status(), http::StatusCode::TEMPORARY_REDIRECT);
}

#[test]
fn test_submit_post_failure() {
    let mut app = test::init_service(App::new().data(db_pool().unwrap().clone())
        .wrap(
            IdentityService::new(
                CookieIdentityPolicy::new(&[0;32])
                    .name("admin")
                    .path("/admin")
                    .max_age(60i64)
                    .secure(false)
            )
        )
        .service(fs::Files::new("/static", "static/").show_files_listing())
        .service(
            web::scope("/admin").service(web::resource("/login/").route(web::post().to_async(views::auth::handle_login)))
                                .service(web::resource("/submit_post/").route(web::post().to_async(views::auth::submit_post)))
        )
    );
    
    // before test getting dashboard, login is required due to setting identity.
    let req = test::TestRequest::post()
                .uri("/admin/login/")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .set_payload(Bytes::from_static(b"username=jdeng&password=welcome"))
                .to_request();
    let resp = test::block_on(app.call(req)).unwrap();
    assert_eq!(resp.status(), http::StatusCode::TEMPORARY_REDIRECT);
    
    // get identity
    let identity = resp.response().cookies().next().clone();
    assert!(identity.is_some());
    
    // this post has existed, insert the same post will cause failure
    let post = b"title=python&slug=python&body=this is python&status=publish";
    let req = test::TestRequest::post()
                .uri("/admin/submit_post/")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .set_payload(Bytes::from_static(post))
                .cookie(identity.unwrap())
                .to_request();
    let resp = test::block_on(app.call(req)).unwrap();
    //let resp = test::block_on(app.call(req)).unwrap();
    assert_eq!(resp.status(), http::StatusCode::INTERNAL_SERVER_ERROR);
}

#[test]
fn test_reset_password() {
    let mut app = test::init_service(App::new().data(db_pool().unwrap().clone())
        .wrap(
            IdentityService::new(
                CookieIdentityPolicy::new(&[0;32])
                    .name("admin")
                    .path("/admin")
                    .max_age(60i64)
                    .secure(false)
            )
        )
        .service(fs::Files::new("/static", "static/").show_files_listing())
        .service(
            web::scope("/admin").service(web::resource("/login/").route(web::post().to_async(views::auth::handle_login)))
                                .service(web::resource("/reset_password/").route(web::get().to_async(views::auth::reset_password)))
        )
    );
    
    // before test getting dashboard, login is required due to setting identity.
    let req = test::TestRequest::post()
                .uri("/admin/login/")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .set_payload(Bytes::from_static(b"username=jdeng&password=welcome"))
                .to_request();
    let resp = test::block_on(app.call(req)).unwrap();
    assert_eq!(resp.status(), http::StatusCode::TEMPORARY_REDIRECT);
    
    // get identity
    let identity = resp.response().cookies().next().clone();
    assert!(identity.is_some());
    
    let req = test::TestRequest::get()
                .uri("/admin/reset_password/")
                .cookie(identity.unwrap())
                .to_request();
    let resp = test::block_on(app.call(req)).unwrap();
    assert_eq!(resp.status(), http::StatusCode::OK);
}

#[test]
fn test_save_changed_password_failure() {
    let mut app = test::init_service(App::new().data(db_pool().unwrap().clone())
        .wrap(
            IdentityService::new(
                CookieIdentityPolicy::new(&[0;32])
                    .name("admin")
                    .path("/admin")
                    .max_age(60i64)
                    .secure(false)
            )
        )
        .service(fs::Files::new("/static", "static/").show_files_listing())
        .service(
            web::scope("/admin").service(web::resource("/login/").route(web::post().to_async(views::auth::handle_login)))
                                .service(web::resource("/reset_password/").route(web::post().to_async(views::auth::save_changed_password)))
        )
    );
    
    // before test getting dashboard, login is required due to setting identity.
    let req = test::TestRequest::post()
                .uri("/admin/login/")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .set_payload(Bytes::from_static(b"username=jdeng&password=welcome"))
                .to_request();
    let resp = test::block_on(app.call(req)).unwrap();
    assert_eq!(resp.status(), http::StatusCode::TEMPORARY_REDIRECT);
    
    // get identity
    let identity = resp.response().cookies().next().clone();
    assert!(identity.is_some());
    
    let new_password = b"old_password=welcome&new_password=welcome";
    let req = test::TestRequest::post()
                .uri("/admin/reset_password/")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .set_payload(Bytes::from_static(new_password))
                .cookie(identity.unwrap())
                .to_request();
    let resp = test::block_on(app.call(req)).unwrap();
    assert_eq!(resp.status(), http::StatusCode::FORBIDDEN);
}

#[test]
fn test_modify_post() {
    let mut app = test::init_service(App::new().data(db_pool().unwrap().clone())
        .wrap(
            IdentityService::new(
                CookieIdentityPolicy::new(&[0;32])
                    .name("admin")
                    .path("/admin")
                    .max_age(60i64)
                    .secure(false)
            )
        )
        .service(fs::Files::new("/static", "static/").show_files_listing())
        .service(
            web::scope("/admin").service(web::resource("/login/").route(web::post().to_async(views::auth::handle_login)))
                                .service(web::resource("/article/{title}/").route(web::get().to_async(views::auth::modify_post)))
        )
    );
    
    // before test getting dashboard, login is required due to setting identity.
    let req = test::TestRequest::post()
                .uri("/admin/login/")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .set_payload(Bytes::from_static(b"username=jdeng&password=welcome"))
                .to_request();
    let resp = test::block_on(app.call(req)).unwrap();
    assert_eq!(resp.status(), http::StatusCode::TEMPORARY_REDIRECT);
    
    // get identity
    let identity = resp.response().cookies().next().clone();
    assert!(identity.is_some());
    
    let req = test::TestRequest::get()
                .uri("/admin/article/Cow%20in%20Rust/")
                .cookie(identity.unwrap())
                .to_request();
    let resp = test::block_on(app.call(req)).unwrap();
    assert_eq!(resp.status(), http::StatusCode::OK);
}

#[test]
fn test_save_modified_post() {
    let mut app = test::init_service(App::new().data(db_pool().unwrap().clone())
        .wrap(
            IdentityService::new(
                CookieIdentityPolicy::new(&[0;32])
                    .name("admin")
                    .path("/admin")
                    .max_age(60i64)
                    .secure(false)
            )
        )
        .service(fs::Files::new("/static", "static/").show_files_listing())
        .service(
            web::scope("/admin").service(web::resource("/login/").route(web::post().to_async(views::auth::handle_login)))
                                .service(web::resource("/{title}/").route(web::post().to_async(views::auth::save_modified_post)).route(web::get().to_async(views::auth::modify_post)))
        )
    );
    
    // before test getting dashboard, login is required due to setting identity.
    let req = test::TestRequest::post()
                .uri("/admin/login/")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .set_payload(Bytes::from_static(b"username=jdeng&password=welcome"))
                .to_request();
    let resp = test::block_on(app.call(req)).unwrap();
    assert_eq!(resp.status(), http::StatusCode::TEMPORARY_REDIRECT);
    
    // get identity
    let identity = resp.response().cookies().next().clone();
    assert!(identity.is_some());
    
    let modified_post = format!("title=python&slug=python&body={}&status=publish", generate_random_string(40));
    let req = test::TestRequest::post()
                .uri("/admin/python/")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .set_payload(Bytes::from(modified_post.as_bytes()))
                .cookie(identity.unwrap())
                .to_request();
    let resp = test::block_on(app.call(req)).unwrap();
    assert_eq!(resp.status(), http::StatusCode::TEMPORARY_REDIRECT);
}