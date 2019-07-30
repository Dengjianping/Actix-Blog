pub(self) mod test_auth_views;
pub(self) mod test_post_views;

use actix_web::dev::Service;
use actix_web::{test, web, App};
use actix_files as fs;

use crate::views;
use crate::utils::utils::db_pool;
use lazy_static::lazy_static;

/*
lazy_static! {
    pub(crate) static ref A: impl Service = test::init_service(
        App::new()
            .service(web::resource("/test").to(|| HttpResponse::Ok()))
    );
}
*/

pub(self) fn service_on() -> impl Service {
    test::init_service(App::new().data(db_pool().unwrap().clone())
        .service(fs::Files::new("/static", "static/").show_files_listing())
        .service(
            web::scope("/").service(web::resource("").route(web::get().to_async(views::post::show_all_posts)))
        )
    )
}