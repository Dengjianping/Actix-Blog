use chrono::prelude::*;
use crate::models::{ schema::posts, user::User, comment::Comment };
use crate::utils::utils::DBPool;
use diesel::{ prelude::*, pg::PgConnection, result::Error as DBError, associations::Identifiable };
use serde_derive::{ Deserialize, Serialize };


// QueryableByName for use sql_query that using raw sql expression
// http://docs.diesel.rs/diesel/fn.sql_query.html
#[derive(Queryable, Debug, Serialize, Deserialize, AsChangeset, Clone, Identifiable, Associations, QueryableByName)]
#[table_name = "posts"]
#[belongs_to(User)]
pub struct Post {
    pub id: i32,
    pub title: String,
    pub slug: String,
    pub body: String,
    pub publish: Option<NaiveDateTime>,
    pub created: Option<NaiveDateTime>,
    pub updated: Option<NaiveDateTime>,
    pub status: String,
    pub user_id: i32,
    pub likes: i32,
}

#[derive(Insertable, Serialize, Deserialize, Debug, AsChangeset)]
#[table_name = "posts"]
pub struct NewPost {
    pub title: String,
    pub slug: String,
    pub body: String,
    pub publish: Option<NaiveDateTime>,
    pub created: Option<NaiveDateTime>,
    pub updated: Option<NaiveDateTime>,
    pub status: String,
    pub user_id: i32,
    pub likes: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SubmitPost {
    pub title: String,
    pub slug: String,
    pub body: String,
    pub status: String,
}


#[derive(Queryable, Serialize, Deserialize, AsChangeset, Debug)]
#[table_name="posts"]
pub struct UpdatedPost {
    pub title: String,
    pub slug: String,
    pub body: String,
    pub status: String,
    pub updated: Option<NaiveDateTime>,
}

impl NewPost {
    pub fn new(_title: &str, _slug: &str, _body: &str, _status: &str, _user_id: i32) -> Self {
        NewPost {
            title: String::from(_title),
            slug: String::from(_slug),
            body: String::from(_body),
            publish: Some(Utc::now().naive_utc()),
            created: Some(Utc::now().naive_utc()),
            updated: Some(Utc::now().naive_utc()),
            status: String::from(_status),
            likes: 0,
            user_id: _user_id,
        }
    }
}


impl actix::Message for NewPost {
    type Result = Result<Post, DBError>;
}


impl actix::Handler<NewPost> for DBPool {
    type Result = Result<Post, DBError>;

    fn handle(&mut self, msg: NewPost, _: &mut Self::Context) -> Self::Result {
        use crate::models::schema::posts::dsl::*; // posts imported
        use crate::models::schema;
        
        let conn: &PgConnection = &self.conn;
        let dupliacted_title = posts
            .filter(schema::posts::title.eq(&msg.title)) // title must be unique
            .load::<Post>(conn).unwrap();
        if dupliacted_title.len() == 0 {
            diesel::insert_into(posts)
                .values(&msg)
                .execute(conn)
                .expect("failed to insert a new post");
        } else {
            println!("the title already in using, try another.");
        }

        let mut items = posts.filter(schema::posts::title.eq(&msg.title)).load::<Post>(conn)
            .expect("failed to load a new post");
        Ok(items.pop().unwrap())
    }
}


pub enum PostFind {
    ID(i32),
    // store the ids related to posts, like vec![1,3]
    // and now, rust doesn't support define enum as unsized type
    // or I cound merge the first two lines
    AllPost,
    AllPostByAuthor(String),
    AllPostByComment(Vec<i32>), 
    Status(String),
    Title(String),
    UpdatePost(bool, String, UpdatedPost), // (true, old_title, change_part)
    UpdateLikes(i32, i32), // (likes, post_id)
}

impl actix::Message for PostFind {
    type Result = Result<Vec<Post>, DBError>;
}

impl actix::Handler<PostFind> for DBPool {
    type Result = Result<Vec<Post>, DBError>;

    fn handle(&mut self, msg: PostFind, _: &mut Self::Context) -> Self::Result {
        use crate::models::schema::posts::dsl::*; // posts imported
        use crate::models::schema::users::dsl::*; // users imported
        use crate::models::schema::comments::dsl::*; // users imported
        use crate::models::schema;

        let conn: &PgConnection = &self.conn;
        let post_items = match msg {
            PostFind::ID(idx) => {
                posts.filter(schema::posts::id.eq(&idx)).load::<Post>(conn)
                    .expect("failed to load posts")
            }
            PostFind::AllPostByComment(ids) => {
                use diesel::sql_query;
                /*use diesel::sql_types::Bool;
                use diesel::pg::Pg;
                // http://docs.diesel.rs/diesel/expression/trait.BoxableExpression.html
                type EXP = Box<BoxableExpression<schema::posts::table, Pg, SqlType=Bool>>;

                let init_exp = Box::new(schema::posts::id.eq(ids[0].clone()));
                let ex: EXP = ids.into_iter().fold(init_exp, |acc: EXP, i| Box::new(acc.and(schema::posts::id.eq(i.clone()))));
                posts.filter(ex).load::<Post>(conn).expect("failed to load posts");*/

                // become like [1, 2, 3, 4] => 1, 2, 3, 4
                // and now, slice pattern doesn't meet the request, vec2tuple
                let rm_brackets: String = format!("{:?}", ids).chars().filter(|c| c.ne(&'[') && c.ne(&']')).collect();
                let raw_sql = format!("SELECT * FROM posts WHERE id in ({})", rm_brackets);
                sql_query(raw_sql).load::<Post>(conn).expect("failed to load posts")
            }
            PostFind::Status(sta) => {
                posts.filter(schema::posts::status.eq(&sta)).load::<Post>(conn)
                    .expect("failed to load posts")
            }
            PostFind::Title(tit) => {
                posts.filter(schema::posts::title.eq(&tit)).load::<Post>(conn)
                    .expect("failed to load posts")
            }
            PostFind::AllPost => {
                posts.order(schema::posts::id.desc()).load::<Post>(conn).expect("failed to load posts")
            }
            PostFind::AllPostByAuthor(author) => { // find all posts by specified user
                let au = users.filter(schema::users::username.eq(&author)).load::<User>(conn)?;
                let author_posts = Post::belonging_to(&au).load::<Post>(conn)?;
                // posts.order(schema::posts::id.desc()).load::<Post>(conn).expect("failed to load posts")
                author_posts
            }
            PostFind::UpdatePost(update, old_title, updated_post) => {
                let mut current_post = posts.filter(schema::posts::title.eq(&old_title)).load::<Post>(conn)
                    .expect("failed to load posts");
                let idx = current_post.pop().unwrap().id;
                let targeted_post = posts.filter(schema::posts::id.eq(&idx));
                if (update) {
                    let ss = diesel::update(targeted_post).set(&updated_post)
                        .load::<Post>(conn).expect("failed to load posts"); // update the modified post
                }
                posts.filter(schema::posts::title.eq(&updated_post.title)).load::<Post>(conn)
                    .expect("failed to load posts")
            }
            PostFind::UpdateLikes(user_likes, idx) => {
                let targeted_post = posts.filter(schema::posts::id.eq(&idx));
                diesel::update(targeted_post).set(schema::posts::likes.eq(&user_likes)).load::<Post>(conn).expect("failed to load posts")
            }
        };
        Ok(post_items)
    }
}