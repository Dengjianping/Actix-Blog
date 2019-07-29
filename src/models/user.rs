use actix_web::web::{ Form, Data };
use chrono::{ NaiveDateTime, Utc };
use diesel::prelude::*;
use serde_derive::{ Deserialize, Serialize };
use std::convert::TryFrom;

use crate::utils::utils::{ Status, PgPool };
use super::schema::{ self, users };

#[derive(Queryable, Debug, Serialize, Deserialize, Identifiable)]
pub(crate) struct User {
    pub(crate) id: i32,
    pub(crate) password: String,
    pub(crate) username: String,
    pub(crate) first_name: String,
    pub(crate) last_name: String,
    pub(crate) email: String,
    pub(crate) is_superuser: bool,
    pub(crate) is_staff: bool,
    pub(crate) is_active: bool,
    pub(crate) last_login: Option<NaiveDateTime>,
    pub(crate) date_joined: Option<NaiveDateTime>,
}

#[derive(Insertable, Serialize, Deserialize, Debug)]
#[table_name = "users"]
pub(crate) struct NewUser {
    pub(crate) username: String,
    pub(crate) password: String,
    pub(crate) first_name: String,
    pub(crate) last_name: String,
    pub(crate) email: String,
    pub(crate) is_staff: bool,
    pub(crate) last_login: Option<NaiveDateTime>,
    pub(crate) date_joined: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub(crate) struct LoginUser {
    pub(crate) username: String,
    pub(crate) password: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct CreateUser {
    pub(crate) username: String,
    pub(crate) password: String,
    pub(crate) first_name: String,
    pub(crate) last_name: String,
    pub(crate) email: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub(crate) struct PasswordChange {
    pub(crate) old_password: String,
    pub(crate) new_password: String,
}

impl TryFrom<Form<CreateUser>> for NewUser {
    type Error = failure::Error;
    
    fn try_from(user: Form<CreateUser>) -> Result<Self, Self::Error> {
        let new_user = NewUser {
            username: user.username.clone(), 
            password: bcrypt::hash(&user.password, bcrypt::DEFAULT_COST)?, 
            first_name: user.first_name.clone(), 
            last_name: user.last_name.clone(), 
            email: user.email.clone(),
            is_staff: false,
            last_login: Some(Utc::now().naive_utc()),
            date_joined: Some(Utc::now().naive_utc()),
        };
        Ok(new_user)
    }
}

pub(crate) struct UserOperation;

impl UserOperation {
    pub(crate) fn get_user_by_name(user_name: &str, pool: &Data<PgPool>) -> Result<Option<User>, failure::Error> {
        use schema::users::dsl::*;
        let conn = &*pool.get()?;
        
        // either email or username
        let user_filter = users.filter(schema::users::username.eq(&user_name))
                                 .or_filter(schema::users::email.eq(&user_name));
        
        // if found, update the time of last login
        let mut user_found = diesel::update(user_filter).set(schema::users::last_login.eq(&Some(Utc::now().naive_utc()))).load::<User>(conn)?;
        Ok(user_found.pop())
    }
    
    pub(crate) fn get_user_by_email(email_addr: &str, pool: &Data<PgPool>) -> Result<Option<User>, failure::Error> {
        use schema::users::dsl::*;
        let conn = &*pool.get()?;
        
        let mut user_found = users.filter(schema::users::email.eq(&email_addr)).load::<User>(conn)?;
        Ok(user_found.pop())
    }
    
    #[allow(dead_code)]
    pub(crate) fn get_user_by_id(uid: i32, pool: &Data<PgPool>) -> Result<Option<User>, failure::Error> {
        use schema::users::dsl::*;
        let conn = &*pool.get()?;
        
        let mut user_found = users.filter(schema::users::id.eq(&uid)).load::<User>(conn)?;
        Ok(user_found.pop())
    }
    
    pub(crate) fn insert_user(new_user: &NewUser, pool: &Data<PgPool>) -> Result<Status, failure::Error> {
        use schema::users::dsl::*;
        let conn = &*pool.get()?;
        
        // user name or email must be unique
        let dupliacted_username_or_email = users.filter(schema::users::username.eq(&new_user.username)) 
                                                .or_filter(schema::users::email.eq(&new_user.email))
                                                .load::<User>(conn)?;
        
        if dupliacted_username_or_email.len().eq(&0) {
            diesel::insert_into(users).values(new_user).execute(conn)?;
            Ok(Status::Success)
        } else {
            Ok(Status::Failure)
        }
    }
    
    pub(crate) fn modify_password(new_password: &str, user_name: &str, pool: &Data<PgPool>) -> Result<Status, failure::Error> {
        use schema::users::dsl::*;
        let conn = &*pool.get()?;
        
        // find the current user
        let user_filter = users.filter(schema::users::username.eq(&user_name))
                                 .or_filter(schema::users::email.eq(&user_name));
        diesel::update(user_filter).set(schema::users::password.eq(new_password)).load::<User>(conn)?;
        Ok(Status::Success)
    }
    
    pub(crate) fn get_id_by_username(user_name: &str, pool: &Data<PgPool>) -> Result<i32, failure::Error> {
        use schema::users::dsl::*;
        let conn = &*pool.get()?;
        
        let mut user_found = users.filter(schema::users::username.eq(&user_name))
                               .or_filter(schema::users::email.eq(&user_name))
                               .load::<User>(conn)?;
        // let uid = user_found.pop().ok_or("didn't find this user in database".to_string())?;
        let uid = user_found.pop().ok_or(failure::err_msg("didn't find this user in database"))?;
        Ok(uid.id)
    }
}