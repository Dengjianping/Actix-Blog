use actix_web::{ web, Error as HttpResponseErr, HttpResponse };
use actix_session::Session;
use chrono::NaiveDateTime;

use futures::{ future::result as FutResult, future::err as FutErr, Future };
use serde_derive::{ Deserialize, Serialize };
use tera;

use crate::utils::utils::{ PgPool, COMPILED_TEMPLATES };
use crate::models::post::{ PostStatus, Post, PostOperation };
use crate::models::comment::{ CreateComment, CommentOperation, NewComment };
use crate::models::contact::{ NewContact, CreateContact, ContactOperation };
use crate::error_types::ErrorKind;

const PAGE: usize = 4;


pub(crate) fn about() -> impl Future<Item=HttpResponse, Error=ErrorKind> {
    let template = COMPILED_TEMPLATES.render("about.html", tera::Context::new());
    
    match template {
        Ok(t) => FutResult(Ok(HttpResponse::Ok().content_type("text/html").body(t))),
        Err(e) => FutErr(ErrorKind::TemplateError(e.to_string()))
    }
}

pub(crate) fn contact() -> impl Future<Item=HttpResponse, Error=ErrorKind> {
    let template = COMPILED_TEMPLATES.render("contact.html", tera::Context::new());
    
    match template {
        Ok(t) => FutResult(Ok(HttpResponse::Ok().content_type("text/html").body(t))),
        Err(e) => FutErr(ErrorKind::TemplateError(e.to_string()))
    }
}

pub(crate) fn add_contact(
    contact: web::Json<CreateContact>, 
    db: web::Data<PgPool>
) -> impl Future<Item=HttpResponse, Error=HttpResponseErr> {
    let new_contact = NewContact::new(&contact);
    if ContactOperation::insert_contact(new_contact, &db).is_ok() {
        FutResult(Ok(HttpResponse::Ok().json(true)))
    } else {
        FutResult(Ok(HttpResponse::Ok().json(false)))
    }
}

pub(crate) fn show_all_posts(
    db: web::Data<PgPool>
) -> impl Future<Item=HttpResponse, Error=ErrorKind> {
    let status = PostStatus::Published;
    let all_posts = PostOperation::get_all_posts(status, &db);
    
    match all_posts {
        Ok(posts) => {
            let mut ctx = tera::Context::new();
            if posts.len() <= PAGE {
                ctx.insert("posts", &posts.get(0..));
            } else {
                ctx.insert("posts", &posts.get(0..PAGE));
            }
            
            // indicate next the page number
            ctx.insert("page_num", &2);
            
            let template = COMPILED_TEMPLATES.render("index.html", ctx);
            match template {
                Ok(t) => FutResult(Ok(HttpResponse::Ok().content_type("text/html").body(t))),
                Err(e) => FutErr(ErrorKind::TemplateError(e.to_string()))
            }
        }
        Err(e) => FutErr(ErrorKind::DbOperationError(e.to_string()))
    }
}

pub(crate) fn pagination(
    page_num: web::Path<usize>, 
    db: web::Data<PgPool>
) -> impl Future<Item=HttpResponse, Error=ErrorKind> {
    let status = PostStatus::Published;
    
    match PostOperation::get_all_posts(status, &db) {
        Ok(posts) => {
            let mut ctx = tera::Context::new();
            
            let created_time: Vec<Option<&NaiveDateTime>> = posts.iter().map(|post| post.publish.as_ref()).collect();
            ctx.insert("created_time", &created_time);
            
            let range = if (*page_num * (PAGE + 1)).lt(&posts.len()) {
                ctx.insert("page_num", &(*page_num + 1));
                ctx.insert("last_page", &false);
                (*page_num - 1) * PAGE..*page_num * (PAGE + 1)
            } else {
                ctx.insert("page_num", &*page_num);
                ctx.insert("last_page", &true);
                (*page_num - 1) * PAGE..posts.len()
            };
            ctx.insert("posts", &posts[range]);

            let template = COMPILED_TEMPLATES.render("index.html", ctx);
            match template {
                Ok(t) => FutResult(Ok(HttpResponse::Ok().content_type("text/html").body(t))),
                Err(e) => FutErr(ErrorKind::TemplateError(e.to_string()))
            }
        }
        Err(e) => FutErr(ErrorKind::DbOperationError(e.to_string()))
    }
}

new_struct!(Like, pub, [Debug, Clone, Serialize, Deserialize], (likes_count=>i32));
pub fn user_likes(
    like: web::Json<Like>, 
    session: Session, 
    db: web::Data<PgPool>
) -> impl Future<Item=HttpResponse, Error=HttpResponseErr> {
    let article_id = session.get::<i32>("article_id");
    
    if let Ok(Some(post_id)) = article_id {
        let _ = PostOperation::update_likes((like.likes_count, post_id), &db);
        FutResult(Ok(HttpResponse::Ok().json(true)))
    } else {
        FutResult(Ok(HttpResponse::Ok().json(false)))
    }
}

new_struct!(Search, pub, [Debug, Clone, Serialize, Deserialize], (key_word=>String));
pub(crate) fn search(
    key_word: web::Form<Search>, 
    db: web::Data<PgPool>
) -> impl Future<Item=HttpResponse, Error=ErrorKind> {
    let status = PostStatus::Published;
    let all_posts = PostOperation::get_all_posts(status, &db);
    
    let ctx = match (all_posts, regex::Regex::new(&format!("(?i){}", &key_word.key_word))) {
        (Ok(posts), Ok(re)) => {
            let matched_posts: Vec<&Post> = posts.iter().filter_map(|post| if re.is_match(&post.body) { Some(post) } else { None }).collect();
            let mut ctx = tera::Context::new();
            ctx.insert("posts", &matched_posts);
            ctx
        }
        _ => {
            let empty_posts: Vec<&Post> = Vec::new();
            let mut ctx = tera::Context::new();
            ctx.insert("posts", &empty_posts);
            ctx
        }
    };
    
    let template = COMPILED_TEMPLATES.render("search.html", ctx);
    match template {
        Ok(t) => FutResult(Ok(HttpResponse::Ok().content_type("text/html").body(t))),
        Err(e) => FutErr(ErrorKind::TemplateError(e.to_string()))
    }
}

pub(crate) fn post_detail(
    title: web::Path<String>,
    session: Session, 
    db: web::Data<PgPool>
) -> impl Future<Item=HttpResponse, Error=ErrorKind> {
    let post_found = PostOperation::get_post_by_title(&title, &db);
    
    match post_found {
        Ok(Some(post)) => {
            let mut ctx = tera::Context::new();
            ctx.insert("post", &post);
            
            let _ = session.set("article_id", &post.id);
            
            let related_comments = CommentOperation::get_comments_by_post(post.id, &db);
            let _ = related_comments.map(|comments| ctx.insert("comments", &comments));
            
            let template = COMPILED_TEMPLATES.render("post_detail.html", ctx);
            match template {
                Ok(t) => FutResult(Ok(HttpResponse::Ok().content_type("text/html").body(t))),
                Err(e) => FutErr(ErrorKind::TemplateError(e.to_string()))
            }
        }
        Ok(None) => FutResult(Ok(HttpResponse::Ok().content_type("text/html").body("this post couldn't be found in database"))),
        Err(e) => FutErr(ErrorKind::DbOperationError(e.to_string()))
    }
}

pub(crate) fn all_posts(db: web::Data<PgPool>) -> impl Future<Item=HttpResponse, Error=ErrorKind> {
    let status = PostStatus::Published;
    let all_posts = PostOperation::get_all_posts(status, &db);

    match all_posts {
        Ok(posts) => {
            let mut ctx = tera::Context::new();
            ctx.insert("posts", &posts);
            
            let template = COMPILED_TEMPLATES.render("all_posts.html", ctx);
            match template {
                Ok(t) => FutResult(Ok(HttpResponse::Ok().content_type("text/html").body(t))),
                Err(e) => FutErr(ErrorKind::TemplateError(e.to_string()))
            }
        }
        Err(e) => FutErr(ErrorKind::DbOperationError(e.to_string()))
    }
}

pub(crate) fn page_404() -> impl Future<Item=HttpResponse, Error=ErrorKind> {
    let template = COMPILED_TEMPLATES.render("404.html", tera::Context::new());
            
    match template {
        Ok(t) => FutResult(Ok(HttpResponse::NotFound().content_type("text/html").body(t))),
        Err(e) => FutErr(ErrorKind::TemplateError(e.to_string()))
    }
}

pub(crate) fn add_comment(
    comment: web::Json<CreateComment>, 
    session: Session, 
    db: web::Data<PgPool>
) -> impl Future<Item=HttpResponse, Error=HttpResponseErr> {
    let article_id = session.get::<i32>("article_id");

    if let Ok(Some(id)) = article_id {
        let new_comment = NewComment::new(&comment, id);
        let _ = CommentOperation::insert_comment(new_comment, &db);
        
        FutResult(Ok(HttpResponse::Ok().json(true)))
    } else {
        FutResult(Ok(HttpResponse::InternalServerError().into()))
    }
}