use actix_web::web::Data;
use chrono::{ Utc, NaiveDateTime, NaiveDate, Datelike }; // Datelike for month(), year(), day()
use diesel::prelude::*;
use serde_derive::{ Deserialize, Serialize };

use crate::utils::utils::PgPool;
use super::{ schema::{ self, comments }, post::Post };

#[derive(Queryable, Serialize, Deserialize, AsChangeset, Debug, Identifiable, Associations)]
#[table_name="comments"]
#[belongs_to(Post)] // must derive Associations
pub(crate) struct Comment {
    pub(crate) id: i32,
    pub(crate) username: String,
    pub(crate) email: String,
    pub(crate) comment: String,
    pub(crate) committed_time: Option<NaiveDateTime>,
    pub(crate) post_id: i32,
}

#[derive(Insertable, Queryable, Serialize, Deserialize, AsChangeset, Debug)]
#[table_name="comments"]
pub(crate) struct NewComment {
    pub(crate) username: String,
    pub(crate) email: String,
    pub(crate) comment: String,
    pub(crate) committed_time: Option<NaiveDateTime>,
    pub(crate) post_id: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct CreateComment {
    pub(crate) comment: String,
    pub(crate) username: String,
    pub(crate) email: String,
}

impl NewComment {
    pub(crate) fn new(comment: &CreateComment, article_id: i32) -> Self {
        NewComment {
            username: comment.username.clone(),
            email: comment.email.clone(),
            comment: comment.comment.clone(),
            committed_time: Some(Utc::now().naive_utc()),
            post_id: article_id,
        }
    }
}

pub(crate) struct CommentOperation;

impl CommentOperation {
    pub(crate) fn get_all_comments(pool: &Data<PgPool>) -> Result<Vec<Comment>, failure::Error> {
        use super::schema::comments::dsl::*;
        let conn = &*pool.get()?;
    
        let all_comments = comments.order(schema::comments::id.desc()).load::<Comment>(conn)?;
        Ok(all_comments)
    }

    pub(crate) fn get_comments_by_post(comment_id: i32, pool: &Data<PgPool>) -> Result<Vec<Comment>, failure::Error> {
        use super::schema::comments::dsl::*;
        let conn = &*pool.get()?;
        
        let all_comments = comments.filter(schema::comments::post_id.eq(&comment_id)).load::<Comment>(conn)?;
        Ok(all_comments)
    }
    
    pub(crate) fn insert_comment(new_comment: NewComment, pool: &Data<PgPool>) -> Result<(), failure::Error> {
        use super::schema::comments::dsl::*;
        let conn = &*pool.get()?;
        diesel::insert_into(comments).values(&new_comment).execute(conn)?;
        Ok(())
    }
    
    pub(crate) fn get_today_comments(pool: &Data<PgPool>) -> Result<Vec<Comment>, failure::Error> {
        use super::schema::comments::dsl::*;
        let conn = &*pool.get()?;
        
        let now_is: NaiveDateTime = Utc::now().naive_utc();
        let today_begin: NaiveDateTime = NaiveDate::from_ymd(now_is.year(), now_is.month(), now_is.day()).and_hms(0, 0, 0);
        let today_comments = comments.filter(schema::comments::committed_time.between(today_begin, now_is)).load::<Comment>(conn)?;
        Ok(today_comments)
    }
    
    pub(crate) fn get_posts_by_comments(ids: impl AsRef<[i32]>, pool: &Data<PgPool>) -> Result<Vec<Post>, failure::Error> {
        let conn = &*pool.get()?;
        
        // like [1, 2, 3, 4] => 1, 2, 3, 4
        let rm_brackets: String = format!("{:?}", ids.as_ref()).chars().filter(|c| c.ne(&'[') && c.ne(&']')).collect();
        let raw_sql = format!("SELECT * FROM posts WHERE id in ({})", rm_brackets);
        let posts = diesel::sql_query(raw_sql).load::<Post>(conn)?;
        Ok(posts)
    }
}