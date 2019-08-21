use actix_web::web::Data;
use chrono::{ NaiveDateTime, NaiveDate, Utc };
use diesel::prelude::*;
use serde_derive::{ Deserialize, Serialize };

use crate::utils::utils::{ Status, PgPool };
use super::{ schema::{ self, posts }, user::User };

#[derive(Queryable, Debug, Serialize, Deserialize, AsChangeset, Clone, Identifiable, Associations, QueryableByName)]
#[table_name = "posts"]
#[belongs_to(User)]
pub(crate) struct Post {
    pub(crate) id: i32,
    pub(crate) title: String,
    pub(crate) slug: String,
    pub(crate) body: String,
    pub(crate) publish: Option<NaiveDateTime>,
    pub(crate) created: Option<NaiveDateTime>,
    pub(crate) updated: Option<NaiveDateTime>,
    pub(crate) status: String,
    pub(crate) user_id: i32,
    pub(crate) likes: i32,
}

#[derive(Insertable, Serialize, Deserialize, Debug, AsChangeset)]
#[table_name = "posts"]
pub(crate) struct NewPost {
    pub(crate) title: String,
    pub(crate) slug: String,
    pub(crate) body: String,
    pub(crate) publish: Option<NaiveDateTime>,
    pub(crate) created: Option<NaiveDateTime>,
    pub(crate) updated: Option<NaiveDateTime>,
    pub(crate) status: String,
    pub(crate) user_id: i32,
    pub(crate) likes: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub(crate) struct SubmitPost {
    pub(crate) title: String,
    pub(crate) slug: String,
    pub(crate) body: String,
    pub(crate) status: String,
}


#[derive(Queryable, Serialize, Deserialize, AsChangeset, Debug)]
#[table_name="posts"]
pub(crate) struct UpdatedPost {
    pub(crate) title: String,
    pub(crate) slug: String,
    pub(crate) body: String,
    pub(crate) status: String,
    pub(crate) updated: Option<NaiveDateTime>,
}

impl NewPost {
    pub(crate) fn new(new_post: &SubmitPost, uid: i32) -> Self {
        NewPost {
            title: String::from(&new_post.title),
            slug: String::from(&new_post.slug),
            body: String::from(&new_post.body),
            publish: Some(Utc::now().naive_utc()),
            created: Some(Utc::now().naive_utc()),
            updated: Some(Utc::now().naive_utc()),
            status: String::from(&new_post.status),
            likes: 0,
            user_id: uid,
        }
    }
}

#[allow(dead_code)]
pub(crate) enum PostStatus {
    All,
    Draft, // draft
    Published, // publish
}

// use a struct to organize the post operation in database
pub(crate) struct PostOperation;

impl PostOperation {
    pub(crate) fn get_all_posts(post_status: PostStatus, pool: &Data<PgPool>) -> Result<Vec<Post>, failure::Error> {
        use schema::posts::dsl::*;
        let conn = &*pool.get()?;
        
        let all_posts = match post_status {
            PostStatus::All => posts.order(schema::posts::id.desc()).load::<Post>(conn)?,
            PostStatus::Draft => posts.order(schema::posts::status.eq("draft")).load::<Post>(conn)?,
            PostStatus::Published => posts.filter(schema::posts::status.eq("publish")).order(schema::posts::id.asc()).load::<Post>(conn)?,
        };
        Ok(all_posts)
    }

    pub(crate) fn get_post_by_title(post_title: &str, pool: &Data<PgPool>) -> Result<Option<Post>, failure::Error> {
        use schema::posts::dsl::*;
        let conn = &*pool.get()?;
        let mut post = posts.filter(schema::posts::title.eq(&post_title)).load::<Post>(conn)?;
        Ok(post.pop())
    }
    
    pub(crate) fn get_posts_by_author(author: &str, pool: &Data<PgPool>) -> Result<Vec<Post>, failure::Error> {
        use schema::users::dsl::*;
        let conn = &*pool.get()?;
        
        let user_filter = users.filter(schema::users::username.eq(&author)).load::<User>(conn)?;
        let all_posts = Post::belonging_to(&user_filter).load::<Post>(conn)?;
        Ok(all_posts)
    }
    
    pub(crate) fn update_likes((likes_num, post_id): (i32, i32), pool: &Data<PgPool>) -> Result<(), failure::Error> {
        use schema::posts::dsl::*;
        let conn = &*pool.get()?;
        
        let target_post = posts.filter(schema::posts::id.eq(&post_id));
        diesel::update(target_post).set(schema::posts::likes.eq(&likes_num)).load::<Post>(conn)?;
        Ok(())
    }
    
    pub(crate) fn update_post(old_title: &str, updated_post: &UpdatedPost, pool: &Data<PgPool>) -> Result<Status, failure::Error> {
        use schema::posts::dsl::*;
        let conn = &*pool.get()?;
        
        // every post has unique title
        let current_post = posts.filter(schema::posts::title.eq(&old_title)).load::<Post>(conn)?;
        let is_updated = current_post.get(0).map_or_else(
            || {
                Ok(Status::Failure)
            },
            |post| {
                let post_filter = posts.filter(schema::posts::id.eq(&post.id));
                diesel::update(post_filter).set(updated_post).load::<Post>(conn)?;
                Ok(Status::Success)
            }
        );
        is_updated
    }
    
    pub(crate) fn insert_post(new_post: &NewPost, pool: &Data<PgPool>) -> Result<Status, failure::Error> {
        use schema::posts::dsl::*;
        let conn = &*pool.get()?;
        
        let dup_title = posts.filter(schema::posts::title.eq(&new_post.title)).load::<Post>(conn)?;
        if dup_title.len().eq(&0) {
            diesel::insert_into(posts).values(new_post).execute(conn)?;
            Ok(Status::Success)
        } else {
            Ok(Status::Failure)
        }
    }
    
    pub(crate) fn get_posts_by_year(year: i32, pool: &Data<PgPool>) -> Result<Vec<Post>, failure::Error> {
        use schema::posts::dsl::*;
        let conn = &*pool.get()?;
        
        let year_begin: NaiveDateTime = NaiveDate::from_ymd(year, 1, 1).and_hms(0, 0, 0);
        let year_end: NaiveDateTime = NaiveDate::from_ymd(year + 1, 1, 1).and_hms(0, 0, 0);
        let all_posts = posts.filter(schema::posts::status.eq("publish"))
                             .filter(schema::posts::created.between(year_begin, year_end))
                             .order(schema::posts::id.asc()).load::<Post>(conn)?;
        Ok(all_posts)
    }
}