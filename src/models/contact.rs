use actix_web::web::Data;
use chrono::{ Utc, NaiveDateTime };
use diesel::prelude::*;
use serde_derive::{ Deserialize, Serialize };

use crate::utils::utils::PgPool;
use super::schema::{ self, contacts };


#[derive(Queryable, Serialize, Deserialize, AsChangeset, Debug)]
#[table_name="contacts"]
pub struct Contact {
    pub id: i32,
    pub tourist_name: String,
    pub email: String,
    pub message: String,
    pub committed_time: Option<NaiveDateTime>,
}

#[derive(Insertable, Queryable, Serialize, Deserialize, AsChangeset, Debug)]
#[table_name="contacts"]
pub struct NewContact {
    pub tourist_name: String,
    pub email: String,
    pub message: String,
    pub committed_time: Option<NaiveDateTime>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CreateContact {
    pub tourist_name: String,
    pub email: String,
    pub message: String,
}

impl NewContact {
    pub fn new(new_contact: &CreateContact) -> Self {
        NewContact {
            tourist_name: new_contact.tourist_name.clone(),
            email: new_contact.email.clone(),
            message: new_contact.message.clone(),
            committed_time: Some(Utc::now().naive_utc()),
        }
    }
}

pub(crate) struct ContactOperation;

impl ContactOperation {
    pub(crate) fn insert_contact(new_contact: NewContact, pool: &Data<PgPool>) -> Result<(), failure::Error> {
        use super::schema::contacts::dsl::*;
        let conn = &*pool.get()?;
        
        diesel::insert_into(contacts).values(&new_contact).execute(conn)?;
        Ok(())
    }
    
    pub(crate) fn get_all_contacts(pool: &Data<PgPool>) -> Result<Vec<Contact>, failure::Error> {
        use super::schema::contacts::dsl::*;
        let conn = &*pool.get()?;
        
        let all_contacts = contacts.order(schema::contacts::id.desc()).load::<Contact>(conn)?;
        Ok(all_contacts)
    }
}