use failure::Fail;
use actix_web::{ error::ResponseError, HttpResponse };

#[allow(dead_code)]
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
    #[fail(display = "The identify is expired, you have to login again")]
    IdentityExpiredError,
    #[fail(display = "You might input a wrong password or {}, try again", _0)]
    PasswordVerificationError(
        String, // String => error message
    ),
    #[fail(display = "failed to change password due to {}", _0)]
    PasswordModificationError(
        String, // String => error message
    )
}

impl ResponseError for ErrorKind {
    fn error_response(&self) -> HttpResponse {
        match self {
            ErrorKind::DbOperationError(e) => HttpResponse::Ok().content_type("text/html").body(e),
            ErrorKind::TemplateError(e) => {
                HttpResponse::Ok().content_type("text/html").body(e)
            }
            ErrorKind::IdentityExpiredError => HttpResponse::TemporaryRedirect().header("Location", "/admin/login/").finish(),
            ErrorKind::PasswordVerificationError(e) => {
                HttpResponse::Ok()
                    .content_type("text/html")
                    .body(format!("<h1 style='text-align: center;'>Wrong Password or {}.</h1>
                                   <h2 style='text-align: center;'><a href='.'>Go back</a></h2>", e))
            }
            ErrorKind::PasswordModificationError(e) => {
                HttpResponse::Ok()
                    .content_type("text/html")
                    .body(format!("<h1 style='text-align: center;'>Failed to reset password due to {}.</h1> 
                                   <h2 style='text-align: center;'><a href='.'>Go back to to reset again</a></h2>", e))
            }
        }
    }
}