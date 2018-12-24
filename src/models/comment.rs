use chrono::prelude::*;
use crate::models::{ schema::comments, post::Post };
use crate::utils::utils::DBPool;
use diesel::{ prelude::*, pg::PgConnection, result::Error as DBError, associations::Identifiable };
use serde_derive::{ Deserialize, Serialize };


#[derive(Queryable, Serialize, Deserialize, AsChangeset, Debug, Identifiable, Associations)]
#[table_name="comments"]
#[belongs_to(Post)] // must derive Associations
pub struct Comment {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub comment: String,
    pub committed_time: Option<NaiveDateTime>,
    pub post_id: i32,
}

#[derive(Insertable, Queryable, Serialize, Deserialize, AsChangeset, Debug)]
#[table_name="comments"]
pub struct NewComment {
    pub username: String,
    pub email: String,
    pub comment: String,
    pub committed_time: Option<NaiveDateTime>,
    pub post_id: i32,
}

impl NewComment {
    pub fn new(comment: &CreateComment, article_id: i32) -> NewComment {
        NewComment {
            username: comment.username.clone(),
            email: comment.email.clone(),
            comment: comment.comment.clone(),
            committed_time: Some(Utc::now().naive_utc()),
            post_id: article_id,
        }
    }
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateComment {
    pub comment: String,
    pub username: String,
    pub email: String,
}


pub enum CommentHandle {
    AllComment,
    TodayComments, // how many new comments added
    AllCommentByPost(i32),
    InsertComment(NewComment),
}


// 'b: 'a means 'b lives as long as 'a at least, that I need.
// impl<'a> From<actix_web::Form<CreateComment>> for NewComment<'a> {
impl From<actix_web::Form<CreateComment>> for NewComment {
    fn from(new_comment: actix_web::Form<CreateComment>) -> NewComment {
        NewComment{
            comment: new_comment.comment.clone(),
            committed_time: Some(Utc::now().naive_utc()),
            username: new_comment.username.clone(),
            email: new_comment.email.clone(),
            post_id: 1,
        }
    }
}


impl actix::Message for CommentHandle {
    type Result = Result<Vec<Comment>, DBError>;
}

impl actix::Handler<CommentHandle> for DBPool {
    type Result = Result<Vec<Comment>, DBError>;

    fn handle(&mut self, msg: CommentHandle, _: &mut Self::Context) -> Self::Result {
        use crate::models::schema::comments::dsl::*; // comments imported
        use crate::models::schema;
        // to use now fucntion, and IntervalDsl lets int and f64 can construct intervals.
        // http://docs.diesel.rs/diesel/pg/expression/extensions/trait.IntervalDsl.html
        // use diesel::dsl::{ now, IntervalDsl };
        // use diesel::pg::data_types::PgInterval;

        let conn: &PgConnection = &self.conn;
        let comment_items = match msg {
            CommentHandle::AllComment => {
                comments.order(schema::comments::id.desc()).load::<Comment>(conn).expect("failed to load all comments.")
            }
            CommentHandle::TodayComments => {
                let now_is: NaiveDateTime = Utc::now().naive_utc();
                let today_begin: NaiveDateTime = NaiveDate::from_ymd(now_is.year(), now_is.month(), now_is.day()).and_hms(0, 0, 0);
                let today_end: NaiveDateTime = NaiveDate::from_ymd(now_is.year(), now_is.month(), now_is.day() + 1).and_hms(0, 0, 0);
                comments.filter(schema::comments::committed_time.between(today_begin, today_end)).load::<Comment>(conn).expect("failed to load related comments.")
                // comments.filter(schema::comments::committed_time.gt(now - 1.0_f64.days())).load::<Comment>(conn).expect("failed to load related comments.")
            }
            CommentHandle::AllCommentByPost(comment_id) => {
                comments.filter(schema::comments::post_id.eq(&comment_id)).load::<Comment>(conn).expect("failed to load related comments.")
            }
            CommentHandle::InsertComment(new_comment) => {
                diesel::insert_into(comments).values(&new_comment).execute(conn).expect("failed to insert a new comment");
                comments.filter(schema::comments::username.eq(&new_comment.username)).load::<Comment>(conn).expect("failed to load a comment.")
            }
        };
        Ok(comment_items)
    }
}