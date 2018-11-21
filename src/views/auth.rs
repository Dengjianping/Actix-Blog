use actix_web::{server, App, HttpRequest, HttpResponse, Form, Json, Result, AsyncResponder, HttpMessage,
                FromRequest, error, http, middleware, Error, Query, State, http::Method, fs, FutureResponse, client};
use actix_web::middleware::session::{RequestSession, SessionStorage, CookieSessionBackend};

use diesel;
use diesel::RunQueryDsl;
use diesel::QueryDsl;
use diesel::prelude::*;
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};

use futures::Future;
use futures::future::{result};

use bcrypt;
use chrono::prelude::*;

use tera;
use utils::utils::{DBPool, DBState, compiled_templates, redirect, async_redirect};
use models::user::{NewUser, User, LoginUser, CreateUser, UserFinds};
use models::post::{Post, NewPost, SubmitPost, PostFind, UpdatedPost};
use models::schema;

use actix_blog::builtin_decorator;

// after rust 1.26, it begin to support impl trait, otherwise
// I have to use signature like this Box<Fn(HttpRequest<S>) -> Result<HttpResponse, Error>>
pub fn login_required<F, S>(func: F) -> impl Fn(HttpRequest<S>) -> FutureResponse<HttpResponse, Error>
    where F: Fn(HttpRequest<S>) -> FutureResponse<HttpResponse, Error> {
    move |req: HttpRequest<S>| {
        if req.is_auth_expired() {
            println!("decorator authrication in if branch");
            func(req)
        } else {
            println!("decorator authrication in else branch");
            async_redirect("/admin/login/")
        }
    }
}

pub fn login_required_with_params<F, S, T>(func: F) -> impl Fn(HttpRequest<S>, Form<T>) -> FutureResponse<HttpResponse, Error>
    where F: Fn(HttpRequest<S>, Form<T>) -> FutureResponse<HttpResponse, Error> {
    move |req: HttpRequest<S>, form: Form<T>| {
        if req.is_auth_expired() {
            println!("decorator authrication in if branch");
            func(req, form)
        } else {
            println!("decorator authrication in else branch");
            async_redirect("/admin/login/")
        }
    }
}


pub trait Authority {
    fn is_auth_expired(&self) -> bool;
    // fn is_logined(&self) -> Option<bool>;
}

impl<S> Authority for HttpRequest<S> {
    fn is_auth_expired(&self) -> bool {
        match self.session().get::<String>("username") {
            Ok(option_user) => {
                match option_user {
                    Some(user) => true,
                    None => false,
                }
            }
            Err(_) => false
        }
    }
}


// user login
pub fn login(req: HttpRequest<DBState>) -> Result<HttpResponse, Error> {
    let s = compiled_templates
            .render("admin/login.html", &tera::Context::new())
            .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}


pub fn handle_login(req: HttpRequest<DBState>, user: Form<LoginUser>) -> FutureResponse<HttpResponse> {
    let u1 = UserFinds::CheckLoginUser(user.clone());
    req.state().db.send(u1).from_err()
        .and_then(move |res| match res {
            Ok(found) => {
                match bcrypt::verify(&user.password, &found.password) {
                    Ok(true) => {
                        req.session().set("uid", found.id);
                        req.session().set("username", &user.username);
                        redirect("/admin/dashboard/")
                    }
                    _ => {
                        redirect("/admin/login/")
                    }
                }
            },
            Err(e) => Ok(HttpResponse::InternalServerError().into())
        }).responder()
}


pub fn logout(req: HttpRequest<DBState>) -> Result<HttpResponse, Error> {
    req.session().clear();
    redirect("/admin/login/")
}


pub fn register(req: HttpRequest<DBState>) -> Result<HttpResponse, Error> {
    let s = compiled_templates
            .render("admin/register.html", &tera::Context::new())
            .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}


pub fn handle_registration(req: HttpRequest<DBState>, user: Form<CreateUser>) -> FutureResponse<HttpResponse> {
    let new_user = user.into(); // already implement trait Into that can convert Form<CreateUser> to NewUser
    let msg = UserFinds::InsertUser(new_user);

    req.state().db.send(msg).from_err()
        .and_then(move |res| match res {
            Ok(found) => {
                req.session().set("uid", found.id);
                req.session().set("username", &found.username);
                redirect("/admin/dashboard/")
            }
            Err(_) => Ok(HttpResponse::InternalServerError().into()),
        })
        .responder()
}


pub fn reset_password(req: HttpRequest<DBState>) -> Result<HttpResponse, Error> {
    if req.is_auth_expired() {
        let s = compiled_templates
                .render("admin/reset_password.html", &tera::Context::new())
                .map_err(|_| error::ErrorInternalServerError("Template error"))?;
        Ok(HttpResponse::Ok().content_type("text/html").body(s))
    } else {
        redirect("/admin/login/")
    }
}


#[derive(Debug, Serialize, Deserialize)]
pub struct ChangePassword{
    pub old_password: String, 
    pub new_password: String,
}
pub fn save_changed_password(req: HttpRequest<DBState>, reset_pwd: Form<ChangePassword>) -> Result<HttpResponse, Error> {
    unimplemented!();
}


pub fn dashboard(req: HttpRequest<DBState>) -> FutureResponse<HttpResponse> {
    if let Some(uid) = req.session().get::<i32>("uid").unwrap() {
        let found = UserFinds::ID(uid);
        req.state().db.send(found).from_err()
            .and_then(move |res| match res {
                Ok(u) => {
                    let s = compiled_templates
                        .render("admin/dashboard.html", &u)
                        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
                    Ok(HttpResponse::Ok().content_type("text/html").body(s))
                }
                Err(_) => {
                    println!("error happened here dashboard");
                    Ok(HttpResponse::InternalServerError().into())
                }
            })
            .responder()
    } else {
        result(redirect("/admin/login/")).responder()
    }
}


#[derive(Debug, Serialize, Deserialize, Default)]
pub struct AJAX {
    pub number: i32,
}
pub fn add_num(req: HttpRequest<DBState>) -> Box<Future<Item=HttpResponse, Error=Error>> {
    req.json()
        .from_err()  // convert all errors into `Error`
        .and_then(|res: AJAX| {
            println!("model: {:?}", res);
            Ok(HttpResponse::Ok().finish())
        })
        .responder()
}


pub fn get_num(req: HttpRequest<DBState>) -> Box<Future<Item=HttpResponse, Error=Error>> {
    // let s = serde_json::to_string(&res).unwrap(); // serialize
    req.json()
        .from_err()  // convert all errors into `Error`
        .and_then(move |res: AJAX| {
            println!("model: {:?}", &res);
            Ok(HttpResponse::Ok().json(AJAX{number: 1111}))  // <- send response
        })
        .responder()
}

pub fn write_post(req: HttpRequest<DBState>) -> Result<HttpResponse, Error> {
    let s = compiled_templates
            .render("admin/writepost.html", &tera::Context::new())
            .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}

#[builtin_decorator(login_required_with_params)]
pub fn submit_post(req: HttpRequest<DBState>, _post: Form<SubmitPost>) -> FutureResponse<HttpResponse, Error> {
    use models::schema::posts::dsl::*; // posts imported
    use models::schema::users::dsl::*; // users imported

    if let Some(uid) = req.session().get::<i32>("uid").unwrap() {
        let found = UserFinds::ID(uid);
        let new_post = NewPost::new(&_post.title, &_post.slug, &_post.body, &_post.status, uid);
        req.state().db.send(new_post).from_err()
            .and_then(move |res| match res {
                Ok(u) => {
                    async_redirect("/admin/dashboard/").responder()
                }
                Err(_) => result(Ok(HttpResponse::InternalServerError().into())).responder()
            })
            .responder()
    } else {
        async_redirect("/admin/login/")
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ShowPost {
    pub posts: Vec<Post>,
}
#[builtin_decorator(login_required)]
pub fn show_all_posts_by_author(req: HttpRequest<DBState>) -> FutureResponse<HttpResponse, Error> {
    match req.is_auth_expired() {
        true => {
            match req.session().get::<String>("username") {
                Ok(author) => {
                    match author {
                        Some(_author) => {
                            let post_find = PostFind::AllPostByAuthor(_author);
                            req.state().db.send(post_find).from_err()
                                .and_then(move |res| match res {
                                    Ok(posts) => {
                                        let created_time: Vec<_> = posts.iter().map(move |post| {
                                            post.publish//.unwrap()
                                        }).collect();
                                        let mut ctx = tera::Context::new();
                                        ctx.insert("posts", &posts);
                                        ctx.insert("created_time", &created_time);

                                        let c = ShowPost{posts: posts};
                                        let s = compiled_templates
                                            .render("admin/allpost.html", &ctx)
                                            .map_err(|_| error::ErrorInternalServerError("Template error")).unwrap();
                                        result(Ok(HttpResponse::Ok().content_type("text/html").body(s)))
                                    }
                                    Err(_) => result(Ok(HttpResponse::InternalServerError().into()))
                                })
                                .responder()
                        }
                        None => async_redirect("/admin/login/")
                    }
                }
                Err(e) => async_redirect("/admin/login/")
            }
        }
        false => {
            async_redirect("/admin/login/")
        }
    } 
}


#[builtin_decorator(login_required)]
pub fn modify_post(req: HttpRequest<DBState>) -> FutureResponse<HttpResponse, Error> {
    let old_title = req.match_info().get("title").unwrap(); // get title back for database querying
    let post_find = PostFind::Title(old_title.to_string());

    req.state().db.send(post_find).from_err()
        .and_then(move |res| match res {
            Ok(posts) => {
                let p = &posts.first();
                let s = compiled_templates
                    .render("admin/modifypost.html", p)
                    .map_err(|_| error::ErrorInternalServerError("Template error")).unwrap();
                result(Ok(HttpResponse::Ok().content_type("text/html").body(s)))
            }
            Err(_) => result(Ok(HttpResponse::InternalServerError().into()))//.responder()
        })
        .responder()
}


#[builtin_decorator(login_required_with_params)]
pub fn save_modified_post(req: HttpRequest<DBState>, _post: Form<SubmitPost>) -> Box<Future<Item = HttpResponse, Error = Error>> {
    let old_title = req.match_info().get("title").unwrap();
    let updated_post = UpdatedPost{
        title: _post.title.to_string(), body: _post.body.to_string(),
        slug: _post.slug.to_string(), status: _post.status.to_string(),
        updated: Utc::now().naive_utc(),
    };

    let post_find = PostFind::UpdatePost(true, old_title.to_string(), updated_post);
    req.state().db.send(post_find).from_err()
        .and_then(move |res| match res {
            Ok(_) => {
                result(redirect("/admin/dashboard/"))
            }
            Err(_) => result(Ok(HttpResponse::InternalServerError().into()))//.responder()
        })
        .responder()
}