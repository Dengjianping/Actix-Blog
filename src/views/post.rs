use actix_web::{ HttpRequest, HttpResponse, dev::PathConfig, Form, middleware::session::{ RequestSession },
                 AsyncResponder, HttpMessage, error, Error as HttpResponseErr, FutureResponse };
use chrono::prelude::*;
use futures::{ Future, future::result as FutResult };
use regex::Regex;
use serde_derive::{ Deserialize, Serialize };
use tera;

#[allow(unused_imports)]
use crate::utils::utils::{ DBState, compiled_templates, redirect, async_redirect };
use crate::models::{ post::{ Post, PostFind },
                     comment::{ Comment, NewComment, CommentHandle, CreateComment },
                     contact::{ Contact, ContactHandle, CreateContact, NewContact }};

// #[macro_use]
// use utils::macros::*;


pub fn about(req: HttpRequest<DBState>) -> FutureResponse<HttpResponse, HttpResponseErr> {
    let template = compiled_templates
            .render("about.html", &tera::Context::new())
            .map_err(|_| error::ErrorInternalServerError("Template error"));
    if let Ok(t) = template {
        FutResult(Ok(HttpResponse::Ok().content_type("text/html").body(t))).responder()
    } else {
        FutResult(Ok(HttpResponse::InternalServerError().into())).responder()
    }
}


pub fn contact(req: HttpRequest<DBState>) -> FutureResponse<HttpResponse, HttpResponseErr> {
    let template = compiled_templates
            .render("contact.html", &tera::Context::new())
            .map_err(|_| error::ErrorInternalServerError("Template error"));
    if let Ok(t) = template {
        FutResult(Ok(HttpResponse::Ok().content_type("text/html").body(t))).responder()
    } else {
        FutResult(Ok(HttpResponse::InternalServerError().into())).responder()
    }
}


pub fn add_contact(req: HttpRequest<DBState>) -> FutureResponse<HttpResponse, HttpResponseErr> {
    req.json().from_err()  // convert all errors into `Error`
        .and_then(move |res: CreateContact| {
            let new_contact = NewContact::new(&res);
            if let Ok(Ok(_)) = req.state().db.send(ContactHandle::InsertContact(new_contact)).wait() {
                Ok(HttpResponse::Ok().json(true))
            } else {
                Ok(HttpResponse::Ok().json(false))
            }
    }).responder()
}


pub fn redirect_index(req: HttpRequest<DBState>) -> FutureResponse<HttpResponse, HttpResponseErr> {
    async_redirect("/index/").responder()
}


pub fn post_detail(req: HttpRequest<DBState>) -> FutureResponse<HttpResponse, HttpResponseErr> {
    // PathConfig::<String>default().disable_decoding();
    let t = req.match_info().get("title").unwrap(); // get title back for database querying
    let t = t.replace("%20", " "); // the parameter is decoded after actix-web 0.7.15.
    let post_find = PostFind::Title(t.to_string());

    req.state().db.send(post_find).from_err()
        .and_then(move |res| match res {
            Ok(posts) => {
                let mut p = posts.first().unwrap().clone(); // should be only one post in this vector
                let _ = req.session().set("article_id", &p.id);

                let comment_handler = CommentHandle::AllCommentByPost(p.id);
                let all_comments = req.state().db.send(comment_handler).wait();

                let mut ctx = tera::Context::new();
                ctx.insert("post", &p);
                match all_comments {
                    Ok(Ok(comments)) => ctx.insert("comments", &comments),
                    _ => (),
                }
                let s = compiled_templates
                    .render("post_detail.html", &ctx)
                    .map_err(|_| error::ErrorInternalServerError("Template error")).unwrap();
                FutResult(Ok(HttpResponse::Ok().content_type("text/html").body(s)))
            }
            Err(_) => FutResult(Ok(HttpResponse::InternalServerError().into()))
        })
        .responder()
}


pub fn show_all_posts(req: HttpRequest<DBState>) -> FutureResponse<HttpResponse, HttpResponseErr> {
    let post_find = PostFind::Status(String::from("publish"));

    const PAGE: u32 = 4; // each page show 4 posts
    req.state().db.send(post_find).from_err()
        .and_then(move |res| match res {
            Ok(posts) => {
                let mut ctx = tera::Context::new();
                if posts.len() < PAGE as usize {
                    ctx.insert("posts", &posts.get(0..));
                } else {
                    ctx.insert("posts", &posts.get(0..PAGE as usize));
                }

                ctx.insert("page_num", &1);

                let s = compiled_templates
                    .render("index.html", &ctx)
                    .map_err(|_| error::ErrorInternalServerError("Template error")).unwrap();
                FutResult(Ok(HttpResponse::Ok().content_type("text/html").body(s)))
            }
            Err(_) => FutResult(Ok(HttpResponse::InternalServerError().into()))
        })
        .responder()
}


pub fn pagination(req: HttpRequest<DBState>) -> FutureResponse<HttpResponse, HttpResponseErr> {
    let post_find = PostFind::Status(String::from("publish"));
    const PAGE: u32 = 4; // each page show 4 posts

    req.state().db.send(post_find).from_err()
        .and_then(move |res| match res {
            Ok(posts) => {
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
                FutResult(Ok(HttpResponse::Ok().content_type("text/html").body(s)))
            }
            Err(_) => FutResult(Ok(HttpResponse::InternalServerError().into()))
        })
        .responder()
}


new_struct!(Like, pub, [Debug, Clone, Serialize, Deserialize], (likes_count=>i32));
pub fn user_likes(req: HttpRequest<DBState>) -> FutureResponse<HttpResponse, HttpResponseErr> {
    let article_id = req.session().get::<i32>("article_id");
    if let Ok(Some(post_id)) = article_id {
        req.json().from_err()  // convert all errors into `Error`
            .and_then(move |res: Like| {
                let handle = PostFind::UpdateLikes(res.likes_count, post_id);
                let _ = req.state().db.send(handle).wait();
                Ok(HttpResponse::Ok().json(true))
            }).responder()
    } else {
        FutResult(Ok(HttpResponse::Ok().json(false))).responder()
    }
}


new_struct!(Search, pub, [Debug, Clone, Serialize, Deserialize], (key_word=>String));
pub fn search(req: HttpRequest<DBState>, search_form: Form<Search>) -> FutureResponse<HttpResponse, HttpResponseErr> {
    let key_word = search_form.clone();
    let published_posts = PostFind::Status(String::from("publish"));
    let all_posts = req.state().db.send(published_posts).wait();
    let re = Regex::new(&format!("(?i){}", &key_word.key_word)).unwrap();
    if let Ok(Ok(posts)) = all_posts /*&& let Ok(re) = Regex::new(&key_word.key_word)*/ {
                let matched_posts: Vec<_> = posts.iter().filter_map(|post| if re.is_match(&post.body) { Some(post) } else { None }).collect();
                let mut ctx = tera::Context::new();
                ctx.insert("posts", &matched_posts);
                let s = compiled_templates
                        .render("search.html", &ctx)
                        .map_err(|_| error::ErrorInternalServerError("Template error")).unwrap();
            FutResult(Ok(HttpResponse::Ok().content_type("text/html").body(s))).responder()
        } else {
            FutResult(Ok(HttpResponse::InternalServerError().into())).responder()
    }
}


pub fn page_404(req: HttpRequest<DBState>) -> FutureResponse<HttpResponse, HttpResponseErr> {
    let template = compiled_templates
            .render("404.html", &tera::Context::new())
            .map_err(|_| error::ErrorInternalServerError("Template error"));
    if let Ok(t) = template {
        FutResult(Ok(HttpResponse::Ok().content_type("text/html").body(t))).responder()
    } else {
        FutResult(Ok(HttpResponse::InternalServerError().into())).responder()
    }
}


pub fn all_posts(req: HttpRequest<DBState>) -> FutureResponse<HttpResponse, HttpResponseErr> {
    let post_find = PostFind::Status(String::from("publish"));

    req.state().db.send(post_find).from_err()
        .and_then(move |res| match res {
            Ok(posts) => {
                let mut ctx = tera::Context::new();
                ctx.insert("posts", &posts);
                let s = compiled_templates
                    .render("all_posts.html", &ctx)
                    .map_err(|_| error::ErrorInternalServerError("Template error")).unwrap();
                FutResult(Ok(HttpResponse::Ok().content_type("text/html").body(s)))
            }
            Err(_) => FutResult(Ok(HttpResponse::InternalServerError().into()))
        })
        .responder()
}


pub fn add_comment(req: HttpRequest<DBState>) -> FutureResponse<HttpResponse, HttpResponseErr> {
    let article_id = req.session().get::<i32>("article_id");
    if let Ok(Some(id)) = article_id {
        req.json().from_err()  // convert all errors into `Error`
                .and_then(move |res: CreateComment| {
                    let _comment = NewComment::new(&res, id);
                    let insertting_comment = CommentHandle::InsertComment(_comment);
                    let _ = req.state().db.send(insertting_comment).wait();
                    
                    Ok(HttpResponse::Ok().json(true))
                }).responder()
    } else {
        FutResult(Ok(HttpResponse::InternalServerError().into())).responder()
    }
}