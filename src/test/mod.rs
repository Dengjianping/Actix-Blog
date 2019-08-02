pub(self) mod test_auth_views;
pub(self) mod test_post_views;

use actix_web::web;
use chrono::Utc;
use std::iter;
use rand::{Rng, thread_rng};
use rand::distributions::Alphanumeric;

use diesel::{ r2d2::{ ConnectionManager, Pool }, pg::PgConnection };
use failure;

use crate::models::post::{ NewPost, PostOperation, PostStatus };
use crate::models::user::{ NewUser, UserOperation };
use crate::utils::utils::{ PgPool, Status };

const USERNAME_WITH_PWD: &[u8] = b"username=actix&password=welcome";

pub(self) fn generate_random_string(length: usize) -> String {
    iter::repeat(()).map(|()| thread_rng().sample(Alphanumeric))
                    .take(length)
                    .collect::<String>()
}

pub(self) fn test_db_pool() -> Result<PgPool, failure::Error> {
    dotenv::dotenv().ok();
    let database_url = dotenv::var("TEST_DATABASE_URL")?;
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = Pool::new(manager)?;
    Ok(pool)
}

// for testing
pub(self) fn insert_posts() {
    let db = web::Data::new(test_db_pool().unwrap().clone());
    // It will insert post if there're less than 4 posts.
    match PostOperation::get_all_posts(PostStatus::Published, &db).map(|v| v.len()) {
        Ok(count) => {
            if count.lt(&4) {
                (0..4).for_each(|_| {
                    let new_post = NewPost {
                        title: generate_random_string(10),
                        slug: generate_random_string(5),
                        body: generate_random_string(40),
                        publish: Some(Utc::now().naive_utc()),
                        created: Some(Utc::now().naive_utc()),
                        updated: Some(Utc::now().naive_utc()),
                        status: "publish".to_owned(),
                        user_id: 1,
                        likes: 0,
                    };
                    match PostOperation::insert_post(&new_post, &db) {
                        Ok(lhs) => assert_eq!(lhs, Status::Success),
                        _ => assert!(false),
                    }
                });
            }
        }
        _ => assert!(false),
    }
    
    match PostOperation::get_post_by_title("python", &db) {
        Ok(Some(_)) => assert!(true),
        Ok(None) => {
            // insert a default post due to there're some hardcode in test cases.
            let new_post = NewPost {
                title: "python".to_owned(),
                slug: "python".to_owned(),
                body: generate_random_string(40),
                publish: Some(Utc::now().naive_utc()),
                created: Some(Utc::now().naive_utc()),
                updated: Some(Utc::now().naive_utc()),
                status: "publish".to_owned(),
                user_id: 1,
                likes: 0,
            };
            match PostOperation::insert_post(&new_post, &db) {
                Ok(lhs) => assert_eq!(lhs, Status::Success),
                _ => assert!(false),
            }
        }
        Err(_) => assert!(false),
    }
}

// for testing
pub(self) fn insert_new_user() {
    let db = web::Data::new(test_db_pool().unwrap().clone());
    // It will not insert user if this user exists
    if let Ok(Some(_)) = UserOperation::get_user_by_name("actix", &db) {
        assert!(true);
    } else {
        let new_user = NewUser {
            username: "actix".to_owned(),
            // welcome as password
            password: "$2y$12$G6QbkGaOodmtzMZg5N29ReuOiJFB0/pFhnqEA3TOBlefDDzUUMmES".to_owned(),
            first_name: "Jim".to_owned(),
            last_name: "Bob".to_owned(),
            email: "jim.bob@actix.com".to_owned(),
            is_staff: false,
            last_login: Some(Utc::now().naive_utc()),
            date_joined: Some(Utc::now().naive_utc()),
        };
        match UserOperation::insert_user(&new_user, &db) {
            Ok(lhs) => assert!(true),
            _ => assert!(false),
        }
    }
}