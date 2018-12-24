use actix_web::{ HttpRequest, HttpResponse, Form, middleware::session::{ RequestSession },
                 AsyncResponder, HttpMessage, error, Error as HttpResponseErr, FutureResponse };
use bcrypt;
use chrono::prelude::*;
use futures::{ future::result as FutResult, Future };
use itertools::Itertools;
use serde_derive::{ self, Deserialize, Serialize };
use std::collections::HashMap;
use tera;

use actix_blog::builtin_decorator; // I don't know why proc-macro cannot be import by crate:: in rust 2018
use crate::models::{ user::{ LoginUser, CreateUser, UserFinds, PasswordChange },
                     comment::{ Comment, NewComment, CommentHandle, CreateComment },
                     post::{ Post, NewPost, SubmitPost, PostFind, UpdatedPost },
                     contact::{ Contact, ContactHandle }};
use crate::utils::utils::{ DBState, compiled_templates, redirect, async_redirect };


// after rust 1.26, it begin to support impl trait, otherwise
// I have to use signature like this Box<Fn(HttpRequest<S>) -> Result<HttpResponse, HttpResponseErr>>
pub fn login_required<F, S>(func: F) -> impl Fn(HttpRequest<S>) -> FutureResponse<HttpResponse, HttpResponseErr>
    where F: Fn(HttpRequest<S>) -> FutureResponse<HttpResponse, HttpResponseErr> {
    move |req: HttpRequest<S>| {
        if req.is_auth_expired() {
            func(req)
        } else {
            async_redirect("/admin/login/")
        }
    }
}

pub fn login_required_with_params<F, S, T>(func: F) -> impl Fn(HttpRequest<S>, Form<T>) -> FutureResponse<HttpResponse, HttpResponseErr>
    where F: Fn(HttpRequest<S>, Form<T>) -> FutureResponse<HttpResponse, HttpResponseErr> {
    move |req: HttpRequest<S>, form: Form<T>| {
        if req.is_auth_expired() {
            func(req, form)
        } else {
            async_redirect("/admin/login/")
        }
    }
}


pub trait Authority {
    fn is_auth_expired(&self) -> bool;
}

impl<S> Authority for HttpRequest<S> {
    fn is_auth_expired(&self) -> bool {
        match self.session().get::<String>("username") {
            Ok(option_user) => {
                match option_user {
                    Some(_user) => true,
                    None => false,
                }
            }
            Err(_) => false
        }
    }
}


// user login
pub fn login(req: HttpRequest<DBState>) -> FutureResponse<HttpResponse, HttpResponseErr> {
    let template = compiled_templates
            .render("admin/login.html", &tera::Context::new())
            .map_err(|_| error::ErrorInternalServerError("Template error"));
    if let Ok(t) = template {
        FutResult(Ok(HttpResponse::Ok().content_type("text/html").body(t))).responder()
    } else {
        FutResult(Ok(HttpResponse::InternalServerError().into())).responder()
    }
}


pub fn handle_login(req: HttpRequest<DBState>, user: Form<LoginUser>) -> FutureResponse<HttpResponse, HttpResponseErr> {
    let u1 = UserFinds::CheckLoginUser(user.clone());
    req.state().db.send(u1).from_err()
        .and_then(move |res| match res {
            Ok(mut found) => {
                match found.pop() {
                    Some(f) => {
                        match bcrypt::verify(&user.password, &f.password) {
                            Ok(true) => {
                                req.session().set("uid", f.id);
                                if let Err(e) = req.session().set("username", f.username) {
                                    println!("cannot set session {:?}", e);
                                }
                                async_redirect("/admin/dashboard/").responder()
                            }
                            _ => {
                                FutResult(Ok(HttpResponse::Ok().content_type("text/html")
                                    .body("<h1 style='text-align: center;'>Wrong Password.</h1> 
                                           <h2 style='text-align: center;'><a href='.'>Go back</a></h2>"))).responder()
                            }
                        }
                    }
                    None => FutResult(Ok(HttpResponse::Ok().content_type("text/html")
                                .body("<h1 style='text-align: center;'>User doesn't exist.</h1>
                                       <h2 style='text-align: center;'><a href='.'>Go back</a></h2>"))).responder()
                }
            },
            Err(e) => FutResult(Ok(HttpResponse::InternalServerError().into())).responder()
        }).responder()
}


new_struct!(UserExist, pub, [Debug, Clone, Serialize, Deserialize], (username=>String));
pub fn user_exist(req: HttpRequest<DBState>) -> FutureResponse<HttpResponse, HttpResponseErr> {
    req.json()
        .from_err()  // convert all errors into `Error`
        .and_then(move |res: UserExist| {
            let login_user = UserFinds::UserName(res.username.clone());
            let fut_result = req.state().db.send(login_user).wait().unwrap(); // this future block here until result returns
            match fut_result {
                Ok(mut u) => {
                    if let Some(_) = u.pop() { // someone popup
                        Ok(HttpResponse::Ok().finish())
                    } else { // none
                        Ok(HttpResponse::Ok().json(true))
                    }
                },
                // Err(e) => Ok(HttpResponse::Ok().json(true))
                Err(e) => Ok(HttpResponse::InternalServerError().into())//.responder()
            }
        }).responder()
}


new_struct!(EmailExist, pub, [Debug, Clone, Serialize, Deserialize], (email=>String));
pub fn email_exist(req: HttpRequest<DBState>) -> FutureResponse<HttpResponse, HttpResponseErr> {
    req.json()
        .from_err()  // convert all errors into `Error`
        .and_then(move |res: EmailExist| {
            let rg_email = UserFinds::Email(res.email.clone());
            let fut_result = req.state().db.send(rg_email).wait().unwrap(); // this future block here until result returns
            match fut_result {
                Ok(mut u) => {
                    if let Some(_) = u.pop() { // someone popup
                        Ok(HttpResponse::Ok().finish())
                    } else { // none
                        Ok(HttpResponse::Ok().json(true))
                    }
                },
                Err(e) => Ok(HttpResponse::InternalServerError().into())//.responder()
            }
        }).responder()
}


pub fn logout(req: HttpRequest<DBState>) -> FutureResponse<HttpResponse, HttpResponseErr> {
    req.session().clear();
    async_redirect("/admin/login/").responder()
}


pub fn register(req: HttpRequest<DBState>) -> FutureResponse<HttpResponse, HttpResponseErr> {
    let template = compiled_templates
            .render("admin/register.html", &tera::Context::new())
            .map_err(|_| error::ErrorInternalServerError("Template error"));
    if let Ok(t) = template {
        FutResult(Ok(HttpResponse::Ok().content_type("text/html").body(t))).responder()
    } else {
        FutResult(Ok(HttpResponse::InternalServerError().into())).responder()
    }
}


pub fn handle_registration(req: HttpRequest<DBState>, user: Form<CreateUser>) -> FutureResponse<HttpResponse, HttpResponseErr> {
    let new_user = user.into(); // already implement trait Into that can convert Form<CreateUser> to NewUser
    let msg = UserFinds::InsertUser(new_user);

    req.state().db.send(msg).from_err()
        .and_then(move |res| match res {
            Ok(mut found) => {
                match found.pop() {
                    Some(f) => {
                        req.session().set("uid", f.id);
                        req.session().set("username", &f.username);
                        async_redirect("/admin/dashboard/").responder()
                    }
                    None => async_redirect("/admin/login/").responder()
                }
            }
            Err(_) => FutResult(Ok(HttpResponse::InternalServerError().into())).responder(),
        })
        .responder()
}


// #[builtin_decorator(login_required)]
pub fn reset_password(req: HttpRequest<DBState>) -> FutureResponse<HttpResponse, HttpResponseErr> {
    let template = compiled_templates
            .render("admin/reset_password.html", &tera::Context::new())
            .map_err(|_| error::ErrorInternalServerError("Template error"));
    if let Ok(t) = template {
        FutResult(Ok(HttpResponse::Ok().content_type("text/html").body(t))).responder()
    } else {
        FutResult(Ok(HttpResponse::InternalServerError().into())).responder()
    }
}


#[builtin_decorator(login_required_with_params)]
pub fn save_changed_password(req: HttpRequest<DBState>, reset_pwd: Form<PasswordChange>) -> FutureResponse<HttpResponse, HttpResponseErr> {
    let mut new_pwd = reset_pwd.clone();
    if new_pwd.old_password.ne(&new_pwd.new_password) {
        let hashed_pwd = bcrypt::hash(&new_pwd.new_password, bcrypt::DEFAULT_COST).expect("Failed to hash the password");
        new_pwd.new_password = hashed_pwd.clone();
        let logined_user = req.session().get::<String>("username").unwrap().unwrap();
        req.state().db.send(UserFinds::UpdatePassword(new_pwd, logined_user)).from_err()
            .and_then(move |res| match res {
                Ok(_) => {
                    req.session().clear(); // let user login with new password
                    FutResult(Ok(HttpResponse::Ok().content_type("text/html")
                        .body("<h1 style='text-align: center;'>Password reset successfully.</h1> 
                               <h2 style='text-align: center;'><a href='/admin/login/'>Go back to login again.</a></h2>"))).responder()
                }
                Err(_) => FutResult(Ok(HttpResponse::Ok().content_type("text/html")
                            .body("<h1 style='text-align: center;'>Failed to reset password.</h1> 
                                   <h2 style='text-align: center;'><a href='.'>Go back to to reset again</a></h2>"))).responder()
        }).responder()
    } else {
        FutResult(Ok(HttpResponse::Ok().content_type("text/html")
            .body("<h1 style='text-align: center;'>Please do not input new password as the same one.</h1> 
                   <h2 style='text-align: center;'><a href='.'>Go back to to reset again</a></h2>"))).responder()
    }
}


#[builtin_decorator(login_required)]
pub fn dashboard(req: HttpRequest<DBState>) -> FutureResponse<HttpResponse, HttpResponseErr> {
    if let Some(user) = req.session().get::<String>("username").unwrap() {
        let found = UserFinds::UserName(user.clone());
        req.state().db.send(found).from_err()
            .and_then(move |res| match res {
                Ok(u) => {
                    // both are futures, async way may improve the performance
                    let all_comments = req.state().db.send(CommentHandle::TodayComments).wait(); 
                    let all_contacts = req.state().db.send(ContactHandle::AllContacts).wait();
                    let mut comments_count = 0;
                    let mut messages_count = 0;
                    if let Ok(Ok(comments)) = all_comments {
                        comments_count = comments.len();
                    } else {
                        (); // do nothing
                    }

                    if let Ok(Ok(contacts)) = all_contacts {
                        messages_count = contacts.len();
                    } else {
                        (); // do nothing
                    }

                    let template_data = serde_json::json!({ "username": &user, "comments_count":  &comments_count, "messages_count": messages_count });
                    let s = compiled_templates
                        .render("admin/dashboard.html", &template_data)
                        .map_err(|_| error::ErrorInternalServerError("Template error"))?;
                    Ok(HttpResponse::Ok().content_type("text/html").body(s))
                }
                Err(_) => {
                    Ok(HttpResponse::InternalServerError().into())
                }
            })
            .responder()
    } else {
        FutResult(redirect("/admin/login/")).responder()
    }
}


pub fn write_post(req: HttpRequest<DBState>) -> FutureResponse<HttpResponse, HttpResponseErr> {
    let template = compiled_templates
            .render("admin/writepost.html", &tera::Context::new())
            .map_err(|_| error::ErrorInternalServerError("Template error"));
    if let Ok(t) = template {
        FutResult(Ok(HttpResponse::Ok().content_type("text/html").body(t))).responder()
    } else {
        FutResult(Ok(HttpResponse::InternalServerError().into())).responder()
    }
}

#[builtin_decorator(login_required_with_params)]
pub fn submit_post(req: HttpRequest<DBState>, _post: Form<SubmitPost>) -> FutureResponse<HttpResponse, HttpResponseErr> {
    if let Some(uid) = req.session().get::<i32>("uid").unwrap() {
        let found = UserFinds::ID(uid);
        let new_post = NewPost::new(&_post.title, &_post.slug, &_post.body, &_post.status, uid);
        req.state().db.send(new_post).from_err()
            .and_then(move |res| match res {
                Ok(u) => {
                    async_redirect("/admin/dashboard/").responder()
                }
                Err(_) => FutResult(Ok(HttpResponse::InternalServerError().into())).responder()
            })
            .responder()
    } else {
        async_redirect("/admin/login/")
    }
}


#[builtin_decorator(login_required)]
pub fn show_all_posts_by_author(req: HttpRequest<DBState>) -> FutureResponse<HttpResponse, HttpResponseErr> {
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

                                let s = compiled_templates
                                    .render("admin/allpost.html", &ctx)
                                    .map_err(|_| error::ErrorInternalServerError("Template error")).unwrap();
                                FutResult(Ok(HttpResponse::Ok().content_type("text/html").body(s)))
                            }
                            Err(_) => FutResult(Ok(HttpResponse::InternalServerError().into()))
                        })
                        .responder()
                }
                None => async_redirect("/admin/login/")
            }
        }
        Err(e) => async_redirect("/admin/login/")
    }
}


#[builtin_decorator(login_required)]
pub fn modify_post(req: HttpRequest<DBState>) -> FutureResponse<HttpResponse, HttpResponseErr> {
    let old_title = req.match_info().get("title").unwrap(); // get title back for database querying
    let old_title = old_title.replace("%20", " ");
    let post_find = PostFind::Title(old_title.to_string());

    req.state().db.send(post_find).from_err()
        .and_then(move |res| match res {
            Ok(posts) => {
                let p = &posts.first();
                let s = compiled_templates
                    .render("admin/modifypost.html", p)
                    .map_err(|_| error::ErrorInternalServerError("Template error")).unwrap();
                FutResult(Ok(HttpResponse::Ok().content_type("text/html").body(s)))
            }
            Err(_) => FutResult(Ok(HttpResponse::InternalServerError().into()))//.responder()
        })
        .responder()
}


#[builtin_decorator(login_required_with_params)]
pub fn save_modified_post(req: HttpRequest<DBState>, _post: Form<SubmitPost>) -> FutureResponse<HttpResponse, HttpResponseErr> {
    let old_title = req.match_info().get("title").unwrap();
    let updated_post = UpdatedPost{
        title: _post.title.to_string(), body: _post.body.to_string(),
        slug: _post.slug.to_string(), status: _post.status.to_string(),
        updated: Some(Utc::now().naive_utc()),
    };

    let post_find = PostFind::UpdatePost(true, old_title.to_string(), updated_post);
    req.state().db.send(post_find).from_err()
        .and_then(move |res| match res {
            Ok(_) => {
                FutResult(redirect("/admin/dashboard/"))
            }
            Err(_) => FutResult(Ok(HttpResponse::InternalServerError().into()))//.responder()
        })
        .responder()
}


pub fn redirect_admin(req: HttpRequest<DBState>) -> FutureResponse<HttpResponse, HttpResponseErr> {
    async_redirect("/admin/login/").responder()
}


#[builtin_decorator(login_required)]
pub fn today_comments(req: HttpRequest<DBState>) -> FutureResponse<HttpResponse, HttpResponseErr> {
    let _ = CommentHandle::TodayComments;
    req.state().db.send(CommentHandle::TodayComments).from_err()
        .and_then(move |res| match res {
            Ok(comments) => {
                let mut ctx = tera::Context::new();
                let mut maps: HashMap<String, Vec<&Comment>> = HashMap::new();
                
                // unique trait from crate itertools
                let ids: Vec<_> = comments.iter().map(move |comment| comment.post_id).unique().collect();
                if ids.len().ne(&0) {
                    let related_posts = req.state().db.send(PostFind::AllPostByComment(ids)).wait();
                    if let Ok(Ok(posts)) = related_posts {
                        let _: Vec<_> = posts.iter().map(|post| 
                                            maps.insert(post.title.clone(), comments.iter().filter(|comment| comment.post_id.eq(&post.id)).collect()))
                                        .collect();
                    } else { (); }
                }
                ctx.insert("comments", &maps);
                let template = compiled_templates.render("admin/today_comments.html", &ctx)
                    .map_err(|_| error::ErrorInternalServerError("Template error"));
                if let Ok(t) = template {
                    FutResult(Ok(HttpResponse::Ok().content_type("text/html").body(t)))
                } else {
                    FutResult(Ok(HttpResponse::InternalServerError().into()))
                }
            }
            Err(_) => FutResult(Ok(HttpResponse::Ok().content_type("text/html")
                        .body("<h1>There's error happened to load today's comments.")))
    }).responder()
    
}


#[builtin_decorator(login_required)]
pub fn all_guests_messages(req: HttpRequest<DBState>) -> FutureResponse<HttpResponse, HttpResponseErr> {
    let mut ctx = tera::Context::new();
    let all_contacts = req.state().db.send(ContactHandle::AllContacts).wait();
    if let Ok(Ok(contacts)) = all_contacts {
        ctx.insert("contacts", &contacts);
    } else { (); }

    let template = compiled_templates
            .render("admin/guest_messages.html", &ctx)
            .map_err(|_| error::ErrorInternalServerError("Template error"));
    if let Ok(t) = template {
        FutResult(Ok(HttpResponse::Ok().content_type("text/html").body(t))).responder()
    } else {
        FutResult(Ok(HttpResponse::InternalServerError().into())).responder()
    }
}


#[builtin_decorator(login_required)]
pub fn about_self(req: HttpRequest<DBState>) -> FutureResponse<HttpResponse, HttpResponseErr> {
    let yourself = req.session().get::<String>("username");
    if let Ok(Some(you)) = yourself {
        req.state().db.send(UserFinds::UserName(you)).from_err()
            .and_then(move |res| match res {
                Ok(mut u) => {
                    let mut ctx = tera::Context::new();
                    if let Some(y) = u.pop() {
                        ctx.insert("yourself", &y);
                    } else { (); }
                    let template = compiled_templates
                            .render("admin/self_info.html", &ctx)
                            .map_err(|_| error::ErrorInternalServerError("Template error"));

                    if let Ok(t) = template {
                        FutResult(Ok(HttpResponse::Ok().content_type("text/html").body(t)))
                    } else {
                        FutResult(Ok(HttpResponse::InternalServerError().into()))
                    }
                }
                Err(_) => FutResult(Ok(HttpResponse::InternalServerError().into()))
        }).responder()
    } else { 
        FutResult(Ok(HttpResponse::Ok().content_type("text/html")
            .body("<h1 style='text-align: center;'>Your session may expired.</h1> 
                   <h2 style='text-align: center;'><a href='/admin/login/'>Go back to login again.</a></h2>"))).responder()
    }
}


#[builtin_decorator(login_required)]
pub fn change_self(req: HttpRequest<DBState>) -> FutureResponse<HttpResponse, HttpResponseErr> {
    unimplemented!();
}