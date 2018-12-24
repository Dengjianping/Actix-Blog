use bcrypt;
use chrono::prelude::*;
use crate::models::schema::users;
use crate::utils::utils::DBPool;
use diesel::{ prelude::*, pg::PgConnection, result::Error as DBError, associations::Identifiable };
use std::time::SystemTime;
use serde_derive::{ Deserialize, Serialize };


#[derive(Queryable, Debug, Serialize, Deserialize, Identifiable)]
pub struct User {
    pub id: i32,
    pub password: String,
    pub username: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub is_superuser: bool,
    pub is_staff: bool,
    pub is_active: bool,
    pub last_login: Option<NaiveDateTime>,
    pub date_joined: Option<NaiveDateTime>,
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
    pub last_login: Option<NaiveDateTime>,
    pub date_joined: Option<NaiveDateTime>,
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
            last_login: Some(Utc::now().naive_utc()),
            date_joined: Some(Utc::now().naive_utc()),
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


impl actix::Actor for DBPool {
    type Context = actix::SyncContext<Self>;
}


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PasswordChange {
    pub old_password: String,
    pub new_password: String,
}


pub enum UserFinds {
    ID(i32),
    UserName(String),
    Email(String),
    UpdatePassword(PasswordChange, String), // this string store username or email
    InsertUser(NewUser),
    CheckLoginUser(LoginUser),
}

impl actix::Message for UserFinds {
    type Result = Result<Vec<User>, DBError>;
}

impl actix::Handler<UserFinds> for DBPool {
    type Result = Result<Vec<User>, DBError>;

    fn handle(&mut self, msg: UserFinds, _: &mut Self::Context) -> Self::Result {
        use crate::models::schema::users::dsl::*; // users imported
        use crate::models::schema;

        let conn: &PgConnection = &self.conn;
        let post_items = match msg {
            UserFinds::ID(idx) => { // find user by id
                users.filter(schema::users::id.eq(&idx)).load::<User>(conn)
                    .expect("failed to load users")
            }
            UserFinds::UserName(user) => { // find user by username
                users.filter(schema::users::username.eq(&user)).load::<User>(conn)
                    .expect("failed to load posts")
            }
            UserFinds::Email(mail) => { // find user by email
                users.filter(schema::users::email.eq(&mail)).load::<User>(conn)
                    .expect("failed to load posts")
            }
            UserFinds::UpdatePassword(new_pwd, _username) => { // change password
                let targeted_user = users.filter(schema::users::username.eq(&_username))
                    .or_filter(schema::users::email.eq(&_username));
                diesel::update(targeted_user).set(schema::users::password.eq(&new_pwd.new_password)).
                    load::<User>(conn).expect("failed to change password")
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
                diesel::update(targeted_user).set(schema::users::last_login.eq(&Some(Utc::now().naive_utc()))).
                    load::<User>(conn).expect("failed to update login time")
            }
        };
        
        Ok(post_items)
    }
}