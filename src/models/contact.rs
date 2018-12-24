use chrono::prelude::*;
use crate::models::schema::contacts;
use crate::utils::utils::DBPool;
use diesel::{ prelude::*, pg::PgConnection, result::Error as DBError, associations::Identifiable };
use serde_derive::{ Deserialize, Serialize };


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
    pub fn new(new_contact: &CreateContact) -> NewContact {
        NewContact {
            tourist_name: new_contact.tourist_name.clone(),
            email: new_contact.email.clone(),
            message: new_contact.message.clone(),
            committed_time: Some(Utc::now().naive_utc()),
        }
    }
}


pub enum ContactHandle {
    AllContacts,
    InsertContact(NewContact),
}


impl actix::Message for ContactHandle {
    type Result = Result<Vec<Contact>, DBError>;
}


impl actix::Handler<ContactHandle> for DBPool {
    type Result = Result<Vec<Contact>, DBError>;

    fn handle(&mut self, msg: ContactHandle, _: &mut Self::Context) -> Self::Result {
        use crate::models::schema::contacts::dsl::*; // contacts imported
        use crate::models::schema;

        let conn: &PgConnection = &self.conn;
        let contact_items = match msg {
            ContactHandle::AllContacts => {
                contacts.order(schema::contacts::id.desc()).load::<Contact>(conn).expect("failed to load all contacts.")
            }
            ContactHandle::InsertContact(new_contact) => {
                diesel::insert_into(contacts).values(&new_contact).execute(conn).expect("failed to insert a new contact");
                contacts.filter(schema::contacts::tourist_name.eq(&new_contact.tourist_name))
                    .load::<Contact>(conn).expect("failed to load current contact.")
            }
        };
        Ok(contact_items)
    }
}