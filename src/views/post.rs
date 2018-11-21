use std::collections::HashMap;
use std::time::SystemTime;

use actix_web::{server, App, HttpRequest, HttpResponse, Form, Json, Result, AsyncResponder, HttpMessage,
                FromRequest, error, http, middleware, Error, Query, State, http::Method, fs, FutureResponse, client};
use actix_web::middleware::session::{RequestSession, SessionStorage, CookieSessionBackend};

use diesel::prelude::*;
use diesel::pg::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};

use futures::Future;
use futures::future::{result};

use bcrypt;
use chrono::prelude::*;


use tera;
use utils::utils::{DBPool, DBState, compiled_templates, redirect, async_redirect};
use models::user::{NewUser, User, LoginUser, CreateUser};
use models::post::{Post, NewPost, SubmitPost, PostFind, UpdatedPost};
use models::schema;

#[macro_use]
use utils::macros;


pub fn about(req: HttpRequest<DBState>) -> Result<HttpResponse, Error> {
    let s = compiled_templates
            .render("about.html", &tera::Context::new())
            .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}


pub fn contact(req: HttpRequest<DBState>) -> Result<HttpResponse, Error> {
    let s = compiled_templates
            .render("contact.html", &tera::Context::new())
            .map_err(|_| error::ErrorInternalServerError("Template error"))?;
    Ok(HttpResponse::Ok().content_type("text/html").body(s))
}


pub fn post_detail(req: HttpRequest<DBState>) -> FutureResponse<HttpResponse, Error> {
    let t = req.match_info().get("title").unwrap(); // get title back for database querying
    let post_find = PostFind::Title(t.to_string());

    req.state().db.send(post_find).from_err()
        .and_then(move |res| match res {
            Ok(posts) => {
                let mut p = posts.first().unwrap().clone(); // should be only one post in this vector
                let s = compiled_templates
                    .render("post_detail.html", &p)
                    .map_err(|_| error::ErrorInternalServerError("Template error")).unwrap();
                result(Ok(HttpResponse::Ok().content_type("text/html").body(s)))
            }
            Err(_) => result(Ok(HttpResponse::InternalServerError().into()))//.responder()
        })
        .responder()
}


#[derive(Serialize, Deserialize, Debug)]
pub struct ShowPost {
    pub posts: Vec<Post>,
}
pub fn show_all_posts(req: HttpRequest<DBState>) -> FutureResponse<HttpResponse, Error> {
    let post_find = PostFind::Status(String::from("publish"));

    const PAGE: u32 = 4; // each page show 4 posts
    req.state().db.send(post_find).from_err()
        .and_then(move |res| match res {
            Ok(mut posts) => {
                let created_time: Vec<NaiveDateTime> = posts.iter().map(move |post| {
                    let chrono_time = post.publish;
                    chrono_time
                }).collect();

                let mut ctx = tera::Context::new();
                if posts.len() < PAGE as usize {
                    ctx.insert("posts", &posts.get(0..));
                } else {
                    ctx.insert("posts", &posts.get(0..PAGE as usize));

                }
                ctx.insert("created_time", &created_time);
                ctx.insert("page_num", &1);

                let c = ShowPost{posts: posts};
                let s = compiled_templates
                    .render("index.html", &ctx)
                    .map_err(|_| error::ErrorInternalServerError("Template error")).unwrap();
                result(Ok(HttpResponse::Ok().content_type("text/html").body(s)))
            }
            Err(_) => result(Ok(HttpResponse::InternalServerError().into()))
        })
        .responder()
}


pub fn pagination(req: HttpRequest<DBState>) -> FutureResponse<HttpResponse, Error> {
    let post_find = PostFind::Status(String::from("publish"));

    const PAGE: u32 = 4; // each page show 4 posts

    req.state().db.send(post_find).from_err()
        .and_then(move |res| match res {
            Ok(mut posts) => {
                let page_num: u32 = req.match_info().get("page_num").unwrap().parse::<u32>().unwrap();

                let posts_count: u32 = posts.len() as u32;
                let posts_gcd: u32 = posts_count % PAGE;

                let page_iter = move |len: u32, gcd: u32| {
                    static mut TIME: u32 = 0;
                    let mut t: u32 = 0;
                    unsafe {
                        TIME += 1;
                        t = TIME;
                    }
                    let end: u32 = if len <= PAGE * (t + 1) { len } else { PAGE * (t + 1) };
                    let start: u32 = if end < len { PAGE * t } else { end - gcd };
                    Some((start as usize..end as usize)) 
                    // return a std::ops::Range
                    // https://doc.rust-lang.org/std/ops/struct.Range.html
                };

                let i = page_iter(posts_count, posts_gcd).unwrap();
               
                let created_time: Vec<_> = posts.iter().map(move |post| {
                    post.publish//.unwrap()
                }).collect();
                let mut ctx = tera::Context::new();

                let length: usize = i.end - i.start; // I have to caculate it here before i has been moved
                ctx.insert("posts", &posts[i]);
                ctx.insert("created_time", &created_time);
                
                if length == posts_gcd as usize {
                    ctx.insert("page_num", &page_num);
                    ctx.insert("last_page", &true);
                } else {
                    ctx.insert("page_num", &(page_num + 1));
                }

                let s = compiled_templates
                    .render("index.html", &ctx)
                    .map_err(|_| error::ErrorInternalServerError("Template error")).unwrap();
                result(Ok(HttpResponse::Ok().content_type("text/html").body(s)))
            }
            Err(_) => result(Ok(HttpResponse::InternalServerError().into()))
        })
        .responder()
}

// new_struct!(Like, pub, [Debug, Clone, Serialize, Deserialize], (like_count=>u32));
#[derive(Debug, Clone, Serialize, Deserialize)]
struct Like { likes_count: u32, }
pub fn user_likes(req: HttpRequest<DBState>) -> Box<Future<Item=HttpResponse, Error=Error>> {
    req.json()
        .from_err()  // convert all errors into `Error`
        .and_then(move |res: Like| {
            println!("ajax event: {:?}", res); // now I don't save the likes count to database
            Ok(HttpResponse::Ok().finish())
        })
        .responder()
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Search { pub key_word: String, }
pub fn search(req: HttpRequest<DBState>, search: Form<Search>) -> FutureResponse<HttpResponse, Error> {
    unimplemented!();
}