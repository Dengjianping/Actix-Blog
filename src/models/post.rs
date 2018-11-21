use bcrypt;
use std::time::SystemTime;

use diesel::prelude::*;
use diesel::{ pg::PgConnection, 
              r2d2::{ ConnectionManager, Pool, PooledConnection }};
use diesel::result::Error as DBError;
use dotenv::dotenv;

use actix::prelude::*;
use chrono::prelude::*;

use models::schema::{users, posts};
use utils::utils::{DBPool, DBState};

use models::user::{User};


#[derive(Queryable, Debug, Serialize, Deserialize, AsChangeset, Clone, Identifiable, Associations)]
#[table_name = "posts"]
#[belongs_to(User)]
pub struct Post {
    pub id: i32,
    pub title: String,
    pub slug: String,
    pub body: String,
    pub publish: NaiveDateTime,
    pub created: NaiveDateTime,
    pub updated: NaiveDateTime,
    pub status: String,
    pub user_id: i32,
}

#[derive(Insertable, Serialize, Deserialize, Debug, AsChangeset)]
#[table_name = "posts"]
pub struct NewPost {
    pub title: String,
    pub slug: String,
    pub body: String,
    pub publish: NaiveDateTime,
    pub created: NaiveDateTime,
    pub updated: NaiveDateTime,
    pub status: String,
    pub user_id: i32,
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
    pub updated: NaiveDateTime,
}

impl NewPost {
    pub fn new(_title: &str, _slug: &str, _body: &str, _status: &str, _user_id: i32) -> Self {
        NewPost {
            title: String::from(_title),
            slug: String::from(_slug),
            body: String::from(_body),
            publish: Utc::now().naive_utc(),
            created: Utc::now().naive_utc(),
            updated: Utc::now().naive_utc(),
            status: String::from(_status),
            user_id: _user_id,
        }
    }
}


impl Message for NewPost {
    type Result = Result<Post, DBError>;
}


impl Handler<NewPost> for DBPool {
    type Result = Result<Post, DBError>;

    fn handle(&mut self, msg: NewPost, _: &mut Self::Context) -> Self::Result {
        use models::schema::posts::dsl::*; // posts imported
        use models::schema;
        
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
    // Author(String),
    Status(String),
    Title(String),
    AllPost(bool),
    AllPostByAuthor(String),
    UpdatePost(bool, String, UpdatedPost), // (true, old_title, change_part)
}

impl Message for PostFind {
    type Result = Result<Vec<Post>, DBError>;
}

impl Handler<PostFind> for DBPool {
    type Result = Result<Vec<Post>, DBError>;

    fn handle(&mut self, msg: PostFind, _: &mut Self::Context) -> Self::Result {
        use models::schema::posts::dsl::*; // posts imported
        use models::schema::users::dsl::*; // users imported
        use models::schema;

        let conn: &PgConnection = &self.conn;
        let mut post_items = match msg {
            PostFind::ID(idx) => {
                posts.filter(schema::posts::id.eq(&idx)).load::<Post>(conn)
                    .expect("failed to load posts")
            }
            PostFind::Status(sta) => {
                posts.filter(schema::posts::status.eq(&sta)).load::<Post>(conn)
                    .expect("failed to load posts")
            }
            PostFind::Title(tit) => {
                posts.filter(schema::posts::title.eq(&tit)).load::<Post>(conn)
                    .expect("failed to load posts")
            }
            PostFind::AllPost(_) => {
                posts.order(schema::posts::id.desc()).load::<Post>(conn).expect("failed to load posts")
            }
            PostFind::AllPostByAuthor(author) => { // find all posts by specified user
                println!("this author is {:?}", &author);
                let au = users.filter(schema::users::username.eq(&author)).load::<User>(conn)?;
                let author_posts = Post::belonging_to(&au).load::<Post>(conn)?;
                println!("this author post are {:?}", &author_posts);
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
                println!("updated my post");
                posts.filter(schema::posts::title.eq(&updated_post.title)).load::<Post>(conn)
                    .expect("failed to load posts")
            }
        };
        
        Ok(post_items)
    }
}