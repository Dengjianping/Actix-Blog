use failure::Fail;
use actix_web::{ error::ResponseError, HttpResponse };

#[derive(Debug, Fail)]
pub(crate) enum ErrorKind {
    #[fail(display = "failed to insert data to database due to {}", _0)]
    DbOperationError(String), // String => diesel error message
    #[fail(display = "failed to render the template file due to {}", _0)]
    TemplateError(
        // #[fail(cause)]
        // tera::Error
        String, // String => error message
    ),
}

impl ResponseError for ErrorKind {
    fn error_response(&self) -> HttpResponse {
        match self {
            ErrorKind::DbOperationError(e) => HttpResponse::Ok().content_type("text/html").body(e),
            ErrorKind::TemplateError(e) => {
                HttpResponse::Ok().content_type("text/html").body(e)
            }
        }
    }
}