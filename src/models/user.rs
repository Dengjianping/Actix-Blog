use bcrypt;
use std::time::SystemTime;

use diesel::prelude::*;
use diesel::{ pg::PgConnection, 
              r2d2::{ ConnectionManager, Pool, PooledConnection }};
use diesel::result::Error as DBError;
use dotenv::dotenv;

use actix::prelude::*;
use chrono::prelude::*;

use models::schema::{users};
use utils::utils::{DBPool, DBState};

#[derive(Queryable, Debug, Serialize, Deserialize, Identifiable)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub is_staff: bool,
    pub is_active: bool,
    pub is_superuser: bool,
    pub date_joined: Option<SystemTime>,
    pub last_login: Option<SystemTime>,
}

#[derive(Insertable, Serialize, Deserialize, Debug)]
#[table_name = "users"]
pub struct NewUser {
    pub username: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub is_staff: bool,
    pub date_joined: Option<SystemTime>,
    pub last_login: Option<SystemTime>,
}


impl From<actix_web::Form<CreateUser>> for NewUser {
    fn from(user: actix_web::Form<CreateUser>) -> Self {
        NewUser{
            username: user.username.clone(), 
            password: bcrypt::hash(&user.password, bcrypt::DEFAULT_COST).expect("Failed to hash the password"), 
            first_name: user.first_name.clone(), 
            last_name: user.last_name.clone(), 
            email: user.email.clone(),
            is_staff: false,
            date_joined: Some(SystemTime::now()),
            last_login: Some(SystemTime::now()),
        }
    }
}


#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct LoginUser {
    pub username: String,
    pub password: String,
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateUser {
    pub username: String,
    pub password: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
}


impl Actor for DBPool {
    type Context = SyncContext<Self>;
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PasswordChange {
    pub old_password: String,
    pub new_password: String,
}


pub enum UserFinds {
    ID(i32),
    UserName(String),
    EMAIL(String),
    UpdatePassword(PasswordChange),
    InsertUser(NewUser),
    CheckLoginUser(LoginUser),
}

impl Message for UserFinds {
    type Result = Result<User, DBError>;
}

impl Handler<UserFinds> for DBPool {
    type Result = Result<User, DBError>;

    fn handle(&mut self, msg: UserFinds, _: &mut Self::Context) -> Self::Result {
        use models::schema::users::dsl::*; // users imported
        use models::schema;

        let conn: &PgConnection = &self.conn;
        let mut post_items = match msg {
            UserFinds::ID(idx) => { // find user by id
                users.filter(schema::users::id.eq(&idx)).load::<User>(conn)
                    .expect("failed to load users")
            }
            UserFinds::UserName(user) => { // find user by username
                users.filter(schema::users::username.eq(&user)).load::<User>(conn)
                    .expect("failed to load posts")
            }
            UserFinds::EMAIL(mail) => { // find user by email
                users.filter(schema::users::email.eq(&mail)).load::<User>(conn)
                    .expect("failed to load posts")
            }
            UserFinds::UpdatePassword(new_pwd) => { // change password
                users.order(schema::users::id.desc()).load::<User>(conn).expect("failed to change password")
            }
            UserFinds::InsertUser(new_user) => { // create a new user
                let dupliacted_username_or_email = users
                    .filter(schema::users::username.eq(&new_user.username)) // user name or email must be unique
                    .or_filter(schema::users::email.eq(&new_user.email))
                    .load::<User>(conn).unwrap();
                if dupliacted_username_or_email.len().eq(&0) {
                    diesel::insert_into(users)
                        .values(&new_user)
                        .execute(conn)
                        .expect("failed to insert a new user");
                    users.filter(schema::users::username.eq(&new_user.username)).load::<User>(conn).unwrap()
                } else {
                    println!("duplicated username");
                    dupliacted_username_or_email
                    // users.filter(schema::users::username.eq(&new_user.username)).load::<User>(conn).unwrap()
                }
            }
            UserFinds::CheckLoginUser(user_info) => { // login by email or username
                let targeted_user = users.filter(schema::users::username.eq(&user_info.username))
                    .or_filter(schema::users::email.eq(&user_info.username));
                diesel::update(targeted_user).set(schema::users::last_login.eq(&Some(SystemTime::now()))).
                    load::<User>(conn).expect("failed to update login time")
            }
        };
        
        Ok(post_items.pop().unwrap())
    }
}