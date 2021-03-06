use actix_web::{ web, Error as HttpResponseErr, HttpResponse };
use actix_identity::Identity;
use chrono::{ NaiveDateTime, Utc };
use itertools::Itertools;
use serde_derive::{ Deserialize, Serialize };
use std::convert::TryFrom;
use std::collections::HashMap;

use crate::utils::utils::{ PgPool, COMPILED_TEMPLATES, Status };
use crate::models::user::{ LoginUser, CreateUser, NewUser, PasswordChange, UserOperation };
use crate::models::contact::ContactOperation;
use crate::models::comment::{ Comment, CommentOperation };
use crate::models::post::{ NewPost, PostOperation, SubmitPost, UpdatedPost };
use crate::error_types::ErrorKind;

use actix_blog::login_required;


pub(crate) fn redirect(url: &str) -> HttpResponse {
    HttpResponse::TemporaryRedirect().header("Location", url).finish()
}

pub(crate) async fn async_redirect(url: &str) -> Result<HttpResponse, HttpResponseErr> {
    Ok(HttpResponse::TemporaryRedirect().header("Location", url).finish())
}

pub(crate) async fn login() -> Result<HttpResponse, ErrorKind> {
    let template = COMPILED_TEMPLATES.render("admin/login.html", &tera::Context::new());
    match template {
        Ok(t) => Ok(HttpResponse::Ok().content_type("text/html").body(t)),
        Err(e) => Err(ErrorKind::TemplateError(e.to_string()))
    }
}

pub(crate) async fn handle_login(
    login_user: web::Form<LoginUser>, 
    db:         web::Data<PgPool>,
    identity:   Identity
) -> Result<HttpResponse, HttpResponseErr> {
    let user_found = UserOperation::get_user_by_name(&login_user.username, &db);
    
    if let Ok(Some(user)) = user_found {
        match bcrypt::verify(&login_user.password, &user.password) {
            Ok(true) => {
                identity.remember(user.username);
                Ok(redirect("/admin/dashboard/"))
            }
            _ => {
                Ok(HttpResponse::Unauthorized().content_type("text/html")
                    .body("<h1 style='text-align: center;'>Wrong Password.</h1> 
                           <h2 style='text-align: center;'><a href='.'>Go back</a></h2>"))
            }
        }
    } else {
        Ok(HttpResponse::Unauthorized().content_type("text/html")
            .body("<h1 style='text-align: center;'>User doesn't exist.</h1>
                   <h2 style='text-align: center;'><a href='.'>Go back</a></h2>"))
    }
}

new_struct!(UserExist, pub, [Debug, Clone, Serialize, Deserialize], (username=>String));
pub(crate) async fn user_exist(
    user_exist: web::Json<UserExist>,
    db: web::Data<PgPool>
) -> Result<HttpResponse, HttpResponseErr> {
    let existed_user = UserOperation::get_user_by_name(&user_exist.username, &db);
    
    match existed_user {
        Ok(Some(_user)) => Ok(HttpResponse::Ok().json(true)),
        _ => Ok(HttpResponse::Ok().json(false)),
        // Err(_e) => FutResult(Ok(HttpResponse::InternalServerError().into()))
    }
}

new_struct!(EmailExist, pub, [Debug, Clone, Serialize, Deserialize], (email=>String));
pub(crate) async fn email_exist(
    email: web::Json<EmailExist>, 
    db: web::Data<PgPool>
) -> Result<HttpResponse, HttpResponseErr> {
    let existed_user = UserOperation::get_user_by_email(&email.email, &db);
    
    match existed_user {
        Ok(Some(_user)) => Ok(HttpResponse::Ok().json(true)),
        Ok(None) => Ok(HttpResponse::Ok().json(false)),
        Err(_e) => Ok(HttpResponse::InternalServerError().into())
    }
}

#[login_required]
pub(crate) async fn dashboard(
    db: web::Data<PgPool>,
    identity: Identity
) -> Result<HttpResponse, ErrorKind> {
    if let Some(user) = identity.identity() {
        // async way may improve the performance
        let all_comments = CommentOperation::get_all_comments(&db); 
        let all_contacts = ContactOperation::get_all_contacts(&db);
        
        let (comments_count, messages_count) = match (all_comments, all_contacts) {
            (Ok(comments), Ok(messages)) => (comments.len(), messages.len()),
            _ => (0, 0)
        };
        
        let mut ctx = tera::Context::new();
        ctx.insert("username", &user);
        ctx.insert("comments_count", &comments_count);
        ctx.insert("messages_count", &messages_count);
        
        let template = COMPILED_TEMPLATES.render("admin/dashboard.html", &ctx);
        match template {
            Ok(t) => Ok(HttpResponse::Ok().content_type("text/html").body(t)),
            Err(e) => Err(ErrorKind::TemplateError(e.to_string()))
        }
    } else {
        Err(ErrorKind::IdentityExpiredError)
    }
}

pub(crate) async fn logout(identity: Identity) -> Result<HttpResponse, HttpResponseErr> {
    identity.forget();
    Ok(redirect("/admin/login/"))
}

pub(crate) async fn register() -> Result<HttpResponse, ErrorKind> {
    let template = COMPILED_TEMPLATES.render("admin/register.html", &tera::Context::new());
    
    match template {
        Ok(t) => Ok(HttpResponse::Ok().content_type("text/html").body(t)),
        Err(e) => Err(ErrorKind::TemplateError(e.to_string()))
    }
}

pub(crate) async fn handle_registration(
    new_user: web::Form<CreateUser>, 
    db: web::Data<PgPool>
) -> Result<HttpResponse, ErrorKind> {
    if let Ok(new_user) = NewUser::try_from(new_user) {
        let is_inserted = UserOperation::insert_user(&new_user, &db);
        match is_inserted {
            Ok(Status::Success) => Ok(redirect("/admin/login/")),
            Ok(Status::Failure) => Ok(HttpResponse::InternalServerError().into()),
            Err(e) => Err(ErrorKind::DbOperationError(e.to_string())),
        }
    } else {
        // failed to extract form data, maybe need a definition of error
        Ok(HttpResponse::InternalServerError().into())
    }
}

#[login_required]
pub(crate) async fn reset_password(identity: Identity) -> Result<HttpResponse, ErrorKind> {
    let template = COMPILED_TEMPLATES.render("admin/reset_password.html", &tera::Context::new());
    
    match template {
        Ok(t) => Ok(HttpResponse::Ok().content_type("text/html").body(t)),
        Err(e) => Err(ErrorKind::TemplateError(e.to_string()))
    }
}

#[login_required]
pub(crate) async fn save_changed_password(
    reset_pwd: web::Form<PasswordChange>, 
    db: web::Data<PgPool>,
    identity: Identity
) -> Result<HttpResponse, ErrorKind> {
    let user_name = identity.identity().unwrap();
    if reset_pwd.old_password.ne(&reset_pwd.new_password) {
        let hashed_new_pwd = bcrypt::hash(&reset_pwd.new_password, bcrypt::DEFAULT_COST);
        match hashed_new_pwd {
            Ok(hashed_pwd) => {
                let is_modified = UserOperation::modify_password(&hashed_pwd, &user_name, &db);
                if let Ok(Status::Success) = is_modified {
                    identity.forget(); // re-login
                    Ok(HttpResponse::Ok().content_type("text/html")
                        .body("<h1 style='text-align: center;'>Password reset successfully.</h1> 
                               <h2 style='text-align: center;'><a href='/admin/login/'>Go back to login again.</a></h2>"))
                } else {
                    Ok(HttpResponse::Ok().content_type("text/html")
                        .body("<h1 style='text-align: center;'>Failed to reset password.</h1> 
                               <h2 style='text-align: center;'><a href='.'>Go back to to reset again</a></h2>"))
                }
            }
            Err(e) => Err(ErrorKind::PasswordModificationError(e.to_string()))
        }
    } else {
        Ok(HttpResponse::Forbidden().content_type("text/html")
            .body("<h1 style='text-align: center;'>Do not use the same password.</h1> 
                   <h2 style='text-align: center;'><a href='.'>Go back to to reset again</a></h2>"))
    }
}

#[login_required]
pub(crate) async fn write_post(identity: Identity) -> Result<HttpResponse, ErrorKind> {
    let author = identity.identity().unwrap();
    let mut ctx = tera::Context::new();
    ctx.insert("username", &author);
    let template = COMPILED_TEMPLATES.render("admin/write_post.html", &ctx);
    
    match template {
        Ok(t) => Ok(HttpResponse::Ok().content_type("text/html").body(t)),
        Err(e) => Err(ErrorKind::TemplateError(e.to_string()))
    }
}

#[login_required]
pub(crate) async fn submit_post(
    new_post: web::Form<SubmitPost>, 
    db: web::Data<PgPool>,
    identity: Identity 
) -> Result<HttpResponse, ErrorKind> {
    let author = identity.identity().unwrap();
    match UserOperation::get_id_by_username(&author, &db) {
        Ok(uid) => {
            let new_post= NewPost::new(&*new_post, uid);
            match PostOperation::insert_post(&new_post, &db) {
                Ok(Status::Success) => Ok(redirect("/admin/dashboard/")),
                _ => Ok(HttpResponse::InternalServerError().into())
            }
        }
        Err(e) => Err(ErrorKind::DbOperationError(e.to_string()))
    }
}

#[login_required]
pub(crate) async fn show_all_posts_by_author(
    db: web::Data<PgPool>,
    identity: Identity
) -> Result<HttpResponse, ErrorKind> {
    let author = identity.identity().unwrap();
    let user_posts = PostOperation::get_posts_by_author(&author, &db).unwrap(); // remove unwrap
    let created_time: Vec<Option<&NaiveDateTime>> = user_posts.iter().map(|post| post.publish.as_ref()).collect();
    
    let mut ctx = tera::Context::new();
    ctx.insert("posts", &user_posts);
    ctx.insert("created_time", &created_time);
    ctx.insert("username", &author);
    
    let template = COMPILED_TEMPLATES.render("admin/all_posts.html", &ctx);
    match template {
        Ok(t) => Ok(HttpResponse::Ok().content_type("text/html").body(t)),
        Err(e) => Err(ErrorKind::TemplateError(e.to_string()))
    }
}

#[login_required]
pub(crate) async fn modify_post(
    title: web::Path<String>,
    db: web::Data<PgPool>,
    identity: Identity
) -> Result<HttpResponse, ErrorKind> {
    let user_name = identity.identity().unwrap();
    if let Ok(Some(post)) = PostOperation::get_post_by_title(&title, &db) {
        let mut ctx = tera::Context::from_serialize(post).unwrap();
        ctx.insert("username", &user_name);
        let template = COMPILED_TEMPLATES.render("admin/modify_post.html", &ctx);
        match template {
            Ok(t) => Ok(HttpResponse::Ok().content_type("text/html").body(t)),
            Err(e) => Err(ErrorKind::TemplateError(e.to_string()))
        }
    } else {
        Err(ErrorKind::IdentityExpiredError)
    }
}

#[login_required]
pub(crate) async fn save_modified_post(
    title: web::Path<String>,
    modified_post: web::Form<SubmitPost>, 
    db: web::Data<PgPool>,
    identity: Identity
) -> Result<HttpResponse, ErrorKind> {
    let updated_post = UpdatedPost {
        title: modified_post.title.to_string(), body: modified_post.body.to_string(),
        slug: modified_post.slug.to_string(), status: modified_post.status.to_string(),
        updated: Some(Utc::now().naive_utc()),
    };
    
    match PostOperation::update_post(&title, &updated_post, &db) {
        Ok(Status::Success) => Ok(redirect("/admin/dashboard/")),
        Ok(Status::Failure) => Ok(HttpResponse::InternalServerError().into()),
        Err(e) => Err(ErrorKind::DbOperationError(e.to_string()))
    }
}

#[login_required]
pub(crate) async fn today_comments(
    db: web::Data<PgPool>,
    identity: Identity
) -> Result<HttpResponse, ErrorKind> {
    let user_name = identity.identity().unwrap();
    let mut ctx = tera::Context::new();
    ctx.insert("username", &user_name);
    
    let all_today_comments = CommentOperation::get_today_comments(&db).unwrap(); // need to remove unwrap
    let mut maps: HashMap<&str, Vec<&Comment>> = HashMap::new();
    
    let ids: Vec<_> = all_today_comments.iter().map(|comment| comment.post_id).unique().collect();
    // user as_ref here duo to making sure maps has the same lifetime with these posts gotten back from database
    let found_posts = CommentOperation::get_posts_by_comments(ids, &db);
    let _ = found_posts.as_ref().map(|posts| {
        posts.iter().for_each(|post| {
            maps.insert(
                &post.title,
                all_today_comments.iter().filter(|comment| comment.post_id.eq(&post.id)).collect()
            );
        })
        // posts
    });
    
    ctx.insert("comments", &maps);
    let template = COMPILED_TEMPLATES.render("admin/today_comments.html", &ctx);
    match template {
        Ok(t) => Ok(HttpResponse::Ok().content_type("text/html").body(t)),
        Err(e) => Err(ErrorKind::TemplateError(e.to_string()))
    }
}

#[login_required]
pub(crate) async fn all_guests_messages(
    db: web::Data<PgPool>,
    identity: Identity
) -> Result<HttpResponse, ErrorKind> {
    let user_name = identity.identity().unwrap();
    let mut ctx = tera::Context::new();
    ctx.insert("username", &user_name);
    
    let guest_msgs = ContactOperation::get_all_contacts(&db);
    let _ = guest_msgs.map(|contacts| {
        ctx.insert("contacts", &contacts);
    });

    let template = COMPILED_TEMPLATES.render("admin/guest_messages.html", &ctx);
    match template {
        Ok(t) => Ok(HttpResponse::Ok().content_type("text/html").body(t)),
        Err(e) => Err(ErrorKind::TemplateError(e.to_string()))
    }
}

#[login_required]
pub(crate) async fn about_self(
    db: web::Data<PgPool>,
    identity: Identity
) -> Result<HttpResponse, ErrorKind> {
    let user_name = identity.identity().unwrap();
    let mut ctx = tera::Context::new();
    ctx.insert("username", &user_name);
    match UserOperation::get_user_by_name(&user_name, &db) {
        Ok(Some(myself)) => {
            ctx.insert("yourself", &myself);
            let template = COMPILED_TEMPLATES.render("admin/self_info.html", &ctx);

            match template {
                Ok(t) => Ok(HttpResponse::Ok().content_type("text/html").body(t)),
                Err(e) => Err(ErrorKind::TemplateError(e.to_string()))
            }
        }
        Ok(None) => {
            Ok(HttpResponse::Ok().content_type("text/html")
                                 .body("cannot find you detailed information in database."))
        }
        Err(e) => Err(ErrorKind::DbOperationError(e.to_string()))
    }
}

pub(crate) async fn redirect_admin() -> Result<HttpResponse, HttpResponseErr> {
    async_redirect("/admin/login/").await
}
