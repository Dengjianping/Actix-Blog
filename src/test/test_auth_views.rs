use actix_web::{ test, web, App, http::header, http };
use actix_files as fs;
use actix_identity::{ CookieIdentityPolicy, IdentityService };
use actix_service::Service;
use bytes::Bytes;
use serde::{ Serialize, Deserialize };

use crate::views;
use super::{ generate_random_string, insert_posts, insert_new_user, test_db_pool, USERNAME_WITH_PWD };

#[actix_rt::test]
async fn test_login() {
    let mut app = test::init_service(App::new()
        .service(fs::Files::new("/static", "static/").show_files_listing())
        .service(
            web::scope("/admin").service(web::resource("/login/").route(web::get().to(views::auth::login)))
        )
    ).await;

    let req = test::TestRequest::get().uri("/admin/login/").to_request();
    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), http::StatusCode::OK);
}

#[actix_rt::test]
async fn test_handle_login() {
    // There is one user in database at least for testing.
    insert_new_user();

    let mut app = test::init_service(App::new().data(test_db_pool().unwrap().clone())
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
                .service(web::resource("/login/").route(web::post().to(views::auth::handle_login))
                )
        )
    ).await;

    let req = test::TestRequest::post()
                .uri("/admin/login/")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .set_payload(Bytes::from_static(USERNAME_WITH_PWD))
                .to_request();

    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), http::StatusCode::TEMPORARY_REDIRECT);
}

#[actix_rt::test]
async fn test_handle_login_failure() {
    let mut app = test::init_service(App::new().data(test_db_pool().unwrap().clone())
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
                .service(web::resource("/login/").route(web::post().to(views::auth::handle_login))
                )
        )
    ).await;

    let req = test::TestRequest::post()
                .uri("/admin/login/")
                .header(header::CONTENT_LENGTH, "41")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .set_payload(Bytes::from_static(b"username=actix&password=123456"))
                .to_request();

    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), http::StatusCode::UNAUTHORIZED);
}

#[actix_rt::test]
async fn test_register() {
    let mut app = test::init_service(App::new()
        .service(fs::Files::new("/static", "static/").show_files_listing())
        .service(
            web::scope("/admin").service(web::resource("/register/").route(web::get().to(views::auth::register)))
        )
    ).await;

    let req = test::TestRequest::get().uri("/admin/register/").to_request();
    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), http::StatusCode::OK);
}

#[actix_rt::test]
async fn test_handle_registration() {
    let mut app = test::init_service(App::new().data(test_db_pool().unwrap().clone())
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
                .service(web::resource("/register/").route(web::post().to(views::auth::handle_registration))
                )
        )
    ).await;

    let new_user = format!("username={}&password={}&first_name={}&last_name={}&email={}",
                        generate_random_string(10), generate_random_string(8), generate_random_string(6),
                        generate_random_string(6), generate_random_string(15));
    let req = test::TestRequest::post()
                .uri("/admin/register/")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .set_payload(Bytes::from(new_user.into_bytes()))
                .to_request();

    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), http::StatusCode::TEMPORARY_REDIRECT);
}

#[actix_rt::test]
async fn test_handle_registration_failure() {
    let mut app = test::init_service(App::new().data(test_db_pool().unwrap().clone())
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
                .service(web::resource("/register/").route(web::post().to(views::auth::handle_registration))
                )
        )
    ).await;

    // cannot register a user with a username that has been existed.
    let new_user = b"username=actix&password=welcome&first_name=jack&last_name=jones&email=jack.jones@actix.com";
    let req = test::TestRequest::post()
                .uri("/admin/register/")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .set_payload(Bytes::from_static(new_user))
                .to_request();

    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), http::StatusCode::INTERNAL_SERVER_ERROR);
}

#[actix_rt::test]
async fn test_user_exist() {
    // There is one user in database at least for testing.
    insert_new_user();

    let mut app = test::init_service(App::new().data(test_db_pool().unwrap().clone())
        .service(fs::Files::new("/static", "static/").show_files_listing())
        .service(
            web::scope("/admin").service(web::resource("/user_exist/").route(web::post().to(views::auth::user_exist)))
        )
    ).await;

    new_struct!(UserExist, pub, [Debug, Clone, Serialize, Deserialize], (username=>String));
    let user_exist = UserExist { username: "actix".to_owned() };
    let req = test::TestRequest::post().uri("/admin/user_exist/")
                                       .header(header::CONTENT_TYPE, "application/json")
                                       .set_json(&user_exist)
                                       .to_request();
    let result: bool = test::read_response_json(&mut app, req).await;
    assert_eq!(result, true);
}

#[actix_rt::test]
async fn test_user_not_exist() {
    let mut app = test::init_service(App::new().data(test_db_pool().unwrap().clone())
        .service(fs::Files::new("/static", "static/").show_files_listing())
        .service(
            web::scope("/admin").service(web::resource("/user_exist/").route(web::post().to(views::auth::user_exist)))
        )
    ).await;

    new_struct!(UserExist, pub, [Debug, Clone, Serialize, Deserialize], (username=>String));
    let user_exist = UserExist { username: "Bob".to_owned() };
    let req = test::TestRequest::post().uri("/admin/user_exist/")
                                       .header(header::CONTENT_TYPE, "application/json")
                                       .set_json(&user_exist)
                                       .to_request();
    let result: bool = test::read_response_json(&mut app, req).await;
    assert_eq!(result, false);
}

#[actix_rt::test]
async fn test_email_exist() {
    // There is one user in database at least for testing.
    insert_new_user();

    let mut app = test::init_service(App::new().data(test_db_pool().unwrap().clone())
        .service(fs::Files::new("/static", "static/").show_files_listing())
        .service(
            web::scope("/admin").service(web::resource("/email_exist/").route(web::post().to(views::auth::email_exist)))
        )
    ).await;

    new_struct!(EmailExist, pub, [Debug, Clone, Serialize, Deserialize], (email=>String));
    let email_exist = EmailExist { email: "jim.bob@actix.com".to_owned() };
    let req = test::TestRequest::post().uri("/admin/email_exist/")
                                       .header(header::CONTENT_TYPE, "application/json")
                                       .set_json(&email_exist)
                                       .to_request();
    let result: bool = test::read_response_json(&mut app, req).await;
    assert_eq!(result, true);
}

#[actix_rt::test]
async fn test_email_not_exist() {
    let mut app = test::init_service(App::new().data(test_db_pool().unwrap().clone())
        .service(fs::Files::new("/static", "static/").show_files_listing())
        .service(
            web::scope("/admin").service(web::resource("/email_exist/").route(web::post().to(views::auth::email_exist)))
        )
    ).await;

    new_struct!(EmailExist, pub, [Debug, Clone, Serialize, Deserialize], (email=>String));
    let email_exist = EmailExist { email: "bob@gmail.com".to_owned() };
    let req = test::TestRequest::post().uri("/admin/email_exist/")
                                       .header(header::CONTENT_TYPE, "application/json")
                                       .set_json(&email_exist)
                                       .to_request();
    let result: bool = test::read_response_json(&mut app, req).await;
    assert_eq!(result, false);
}

#[actix_rt::test]
async fn test_dashboard_post() {
    // There is one user in database at least for testing.
    insert_new_user();

    let mut app = test::init_service(App::new().data(test_db_pool().unwrap().clone())
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
            web::scope("/admin").service(web::resource("/login/").route(web::post().to(views::auth::handle_login)))
                                .service(web::resource("/dashboard/").route(web::post().to(views::auth::dashboard)))
        )
    ).await;

    // before test getting dashboard, login is required due to setting identity.
    let req = test::TestRequest::post()
                .uri("/admin/login/")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .set_payload(Bytes::from_static(USERNAME_WITH_PWD))
                .to_request();
    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), http::StatusCode::TEMPORARY_REDIRECT);

    // get identity
    let identity = resp.response().cookies().next().clone();
    assert!(identity.is_some());

    let req = test::TestRequest::post().uri("/admin/dashboard/").cookie(identity.unwrap()).to_request();
    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), http::StatusCode::OK);
}

#[actix_rt::test]
async fn test_dashboard_get() {
    // There is one user in database at least for testing.
    insert_new_user();

    let mut app = test::init_service(App::new().data(test_db_pool().unwrap().clone())
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
            web::scope("/admin").service(web::resource("/login/").route(web::post().to(views::auth::handle_login)))
                                .service(web::resource("/dashboard/").route(web::get().to(views::auth::dashboard)))
        )
    ).await;

    // before test getting dashboard, login is required due to setting identity.
    let req = test::TestRequest::post()
                .uri("/admin/login/")
                .header(header::CONTENT_LENGTH, "41")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .set_payload(Bytes::from_static(USERNAME_WITH_PWD))
                .to_request();
    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), http::StatusCode::TEMPORARY_REDIRECT);

    // get identity
    let identity = resp.response().cookies().next().clone();
    assert!(identity.is_some());

    let req = test::TestRequest::get().uri("/admin/dashboard/").cookie(identity.unwrap()).to_request();
    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), http::StatusCode::OK);
}

#[actix_rt::test]
async fn test_logout() {
    // There is one user in database at least for testing.
    insert_new_user();

    let mut app = test::init_service(App::new().data(test_db_pool().unwrap().clone())
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
            web::scope("/admin").service(web::resource("/login/").route(web::post().to(views::auth::handle_login)))
                                .service(web::resource("/logout/").route(web::get().to(views::auth::logout)))
        )
    ).await;

    // before test getting dashboard, login is required due to setting identity.
    let req = test::TestRequest::post()
                .uri("/admin/login/")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .set_payload(Bytes::from_static(USERNAME_WITH_PWD))
                .to_request();
    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), http::StatusCode::TEMPORARY_REDIRECT);

    // get identity
    let identity = resp.response().cookies().next().clone();
    assert!(identity.is_some());

    let req = test::TestRequest::get().uri("/admin/logout/").cookie(identity.unwrap()).to_request();
    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), http::StatusCode::TEMPORARY_REDIRECT);
}

#[actix_rt::test]
async fn test_about_self() {
    // There is one user in database at least for testing.
    insert_new_user();

    let mut app = test::init_service(App::new().data(test_db_pool().unwrap().clone())
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
            web::scope("/admin").service(web::resource("/login/").route(web::post().to(views::auth::handle_login)))
                                .service(web::resource("/about_self/").route(web::get().to(views::auth::about_self)))
        )
    ).await;

    // before test getting dashboard, login is required due to setting identity.
    let req = test::TestRequest::post()
                .uri("/admin/login/")
                .header(header::CONTENT_LENGTH, "41")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .set_payload(Bytes::from_static(USERNAME_WITH_PWD))
                .to_request();
    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), http::StatusCode::TEMPORARY_REDIRECT);

    // get identity
    let identity = resp.response().cookies().next().clone();
    assert!(identity.is_some());

    let req = test::TestRequest::get().uri("/admin/about_self/").cookie(identity.unwrap()).to_request();
    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), http::StatusCode::OK);
}

#[actix_rt::test]
async fn test_show_all_posts_by_author() {
    // There is one user in database at least for testing.
    insert_new_user();

    let mut app = test::init_service(App::new().data(test_db_pool().unwrap().clone())
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
            web::scope("/admin").service(web::resource("/login/").route(web::post().to(views::auth::handle_login)))
                                .service(web::resource("/all_posts/").route(web::get().to(views::auth::show_all_posts_by_author)))
        )
    ).await;

    // before test getting dashboard, login is required due to setting identity.
    let req = test::TestRequest::post()
                .uri("/admin/login/")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .set_payload(Bytes::from_static(USERNAME_WITH_PWD))
                .to_request();
    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), http::StatusCode::TEMPORARY_REDIRECT);

    // get identity
    let identity = resp.response().cookies().next().clone();
    assert!(identity.is_some());

    let req = test::TestRequest::get().uri("/admin/all_posts/").cookie(identity.unwrap()).to_request();
    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), http::StatusCode::OK);
}

#[actix_rt::test]
async fn test_today_comments() {
    // There is one user in database at least for testing.
    insert_new_user();

    let mut app = test::init_service(App::new().data(test_db_pool().unwrap().clone())
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
            web::scope("/admin").service(web::resource("/login/").route(web::post().to(views::auth::handle_login)))
                                .service(web::resource("/today_comments/").route(web::get().to(views::auth::today_comments)))
        )
    ).await;

    // before test getting dashboard, login is required due to setting identity.
    let req = test::TestRequest::post()
                .uri("/admin/login/")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .set_payload(Bytes::from_static(USERNAME_WITH_PWD))
                .to_request();
    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), http::StatusCode::TEMPORARY_REDIRECT);

    // get identity
    let identity = resp.response().cookies().next().clone();
    assert!(identity.is_some());

    let req = test::TestRequest::get().uri("/admin/today_comments/").cookie(identity.unwrap()).to_request();
    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), http::StatusCode::OK);
}

#[actix_rt::test]
async fn test_all_guests_messages() {
    // There is one user in database at least for testing.
    insert_new_user();

    let mut app = test::init_service(App::new().data(test_db_pool().unwrap().clone())
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
            web::scope("/admin").service(web::resource("/login/").route(web::post().to(views::auth::handle_login)))
                                .service(web::resource("/all_guests_messages/").route(web::get().to(views::auth::all_guests_messages)))
        )
    ).await;

    // before test getting dashboard, login is required due to setting identity.
    let req = test::TestRequest::post()
                .uri("/admin/login/")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .set_payload(Bytes::from_static(USERNAME_WITH_PWD))
                .to_request();
    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), http::StatusCode::TEMPORARY_REDIRECT);

    // get identity
    let identity = resp.response().cookies().next().clone();
    assert!(identity.is_some());

    let req = test::TestRequest::get().uri("/admin/all_guests_messages/").cookie(identity.unwrap()).to_request();
    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), http::StatusCode::OK);
}

#[actix_rt::test]
async fn test_write_post() {
    // There is one user in database at least for testing.
    insert_new_user();

    let mut app = test::init_service(App::new().data(test_db_pool().unwrap().clone())
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
            web::scope("/admin").service(web::resource("/login/").route(web::post().to(views::auth::handle_login)))
                                .service(web::resource("/write_post/").route(web::get().to(views::auth::write_post)))
        )
    ).await;

    // before test getting dashboard, login is required due to setting identity.
    let req = test::TestRequest::post()
                .uri("/admin/login/")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .set_payload(Bytes::from_static(USERNAME_WITH_PWD))
                .to_request();
    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), http::StatusCode::TEMPORARY_REDIRECT);

    // get identity
    let identity = resp.response().cookies().next().clone();
    assert!(identity.is_some());

    let req = test::TestRequest::get().uri("/admin/write_post/").cookie(identity.unwrap()).to_request();
    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), http::StatusCode::OK);
}

#[actix_rt::test]
async fn test_submit_post() {
    // There is one user in database at least for testing.
    insert_new_user();

    let mut app = test::init_service(App::new().data(test_db_pool().unwrap().clone())
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
            web::scope("/admin").service(web::resource("/login/").route(web::post().to(views::auth::handle_login)))
                                .service(web::resource("/write_post/").route(web::post().to(views::auth::submit_post)))
        )
    ).await;

    // before test getting dashboard, login is required due to setting identity.
    let req = test::TestRequest::post()
                .uri("/admin/login/")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .set_payload(Bytes::from_static(USERNAME_WITH_PWD))
                .to_request();
    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), http::StatusCode::TEMPORARY_REDIRECT);

    // get identity
    let identity = resp.response().cookies().next().clone();
    assert!(identity.is_some());

    let post = format!("title={}&slug={}&body={}&status=publish",
                        generate_random_string(4), generate_random_string(4), generate_random_string(40));
    let req = test::TestRequest::post()
                .uri("/admin/write_post/")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .set_payload(Bytes::from(post.into_bytes()))
                .cookie(identity.unwrap())
                .to_request();
    let resp = app.call(req).await.unwrap();
    //let resp = test::block_on(app.call(req)).unwrap();
    assert_eq!(resp.status(), http::StatusCode::TEMPORARY_REDIRECT);
}

#[actix_rt::test]
async fn test_submit_post_failure() {
    // There is one user in database at least for testing.
    insert_new_user();

    let mut app = test::init_service(App::new().data(test_db_pool().unwrap().clone())
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
            web::scope("/admin").service(web::resource("/login/").route(web::post().to(views::auth::handle_login)))
                                .service(web::resource("/submit_post/").route(web::post().to(views::auth::submit_post)))
        )
    ).await;

    // before test getting dashboard, login is required due to setting identity.
    let req = test::TestRequest::post()
                .uri("/admin/login/")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .set_payload(Bytes::from_static(USERNAME_WITH_PWD))
                .to_request();
    let resp = app.call(req).await.unwrap();
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
    let resp = app.call(req).await.unwrap();
    //let resp = test::block_on(app.call(req)).unwrap();
    assert_eq!(resp.status(), http::StatusCode::INTERNAL_SERVER_ERROR);
}

#[actix_rt::test]
async fn test_reset_password() {
    // There is one user in database at least for testing.
    insert_new_user();

    let mut app = test::init_service(App::new().data(test_db_pool().unwrap().clone())
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
            web::scope("/admin").service(web::resource("/login/").route(web::post().to(views::auth::handle_login)))
                                .service(web::resource("/reset_password/").route(web::get().to(views::auth::reset_password)))
        )
    ).await;

    // before test getting dashboard, login is required due to setting identity.
    let req = test::TestRequest::post()
                .uri("/admin/login/")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .set_payload(Bytes::from_static(USERNAME_WITH_PWD))
                .to_request();
    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), http::StatusCode::TEMPORARY_REDIRECT);

    // get identity
    let identity = resp.response().cookies().next().clone();
    assert!(identity.is_some());

    let req = test::TestRequest::get()
                .uri("/admin/reset_password/")
                .cookie(identity.unwrap())
                .to_request();
    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), http::StatusCode::OK);
}

#[actix_rt::test]
async fn test_save_changed_password_failure() {
    // There is one user in database at least for testing.
    insert_new_user();

    let mut app = test::init_service(App::new().data(test_db_pool().unwrap().clone())
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
            web::scope("/admin").service(web::resource("/login/").route(web::post().to(views::auth::handle_login)))
                                .service(web::resource("/reset_password/").route(web::post().to(views::auth::save_changed_password)))
        )
    ).await;

    // before test getting dashboard, login is required due to setting identity.
    let req = test::TestRequest::post()
                .uri("/admin/login/")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .set_payload(Bytes::from_static(USERNAME_WITH_PWD))
                .to_request();
    let resp = app.call(req).await.unwrap();
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
    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), http::StatusCode::FORBIDDEN);
}

#[actix_rt::test]
async fn test_modify_post() {
    // There is one user in database at least for testing.
    insert_new_user();
    insert_posts();

    let mut app = test::init_service(App::new().data(test_db_pool().unwrap().clone())
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
            web::scope("/admin").service(web::resource("/login/").route(web::post().to(views::auth::handle_login)))
                                .service(web::resource("/{title}/").route(web::get().to(views::auth::modify_post)))
        )
    ).await;

    // before test getting dashboard, login is required due to setting identity.
    let req = test::TestRequest::post()
                .uri("/admin/login/")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .set_payload(Bytes::from_static(USERNAME_WITH_PWD))
                .to_request();
    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), http::StatusCode::TEMPORARY_REDIRECT);

    // get identity
    let identity = resp.response().cookies().next().clone();
    assert!(identity.is_some());

    let req = test::TestRequest::get()
                .uri("/admin/python/")
                .cookie(identity.unwrap())
                .to_request();
    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), http::StatusCode::OK);
}

#[actix_rt::test]
async fn test_save_modified_post() {
    // There is one user in database at least for testing.
    insert_new_user();
    insert_posts();

    let mut app = test::init_service(App::new().data(test_db_pool().unwrap().clone())
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
            web::scope("/admin").service(web::resource("/login/").route(web::post().to(views::auth::handle_login)))
                                .service(web::resource("/{title}/").route(web::post().to(views::auth::save_modified_post)).route(web::get().to(views::auth::modify_post)))
        )
    ).await;

    // before test getting dashboard, login is required due to setting identity.
    let req = test::TestRequest::post()
                .uri("/admin/login/")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .set_payload(Bytes::from_static(USERNAME_WITH_PWD))
                .to_request();
    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), http::StatusCode::TEMPORARY_REDIRECT);

    // get identity
    let identity = resp.response().cookies().next().clone();
    assert!(identity.is_some());

    let modified_post = format!("title=python&slug=python&body={}&status=publish", generate_random_string(40));
    let req = test::TestRequest::post()
                .uri("/admin/python/")
                .header(header::CONTENT_TYPE, "application/x-www-form-urlencoded")
                .set_payload(Bytes::from(modified_post.into_bytes()))
                .cookie(identity.unwrap())
                .to_request();
    let resp = app.call(req).await.unwrap();
    assert_eq!(resp.status(), http::StatusCode::TEMPORARY_REDIRECT);
}
