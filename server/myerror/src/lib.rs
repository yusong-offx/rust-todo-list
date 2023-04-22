use actix_web::{error, HttpResponse, http::{header::ContentType, StatusCode}};
use derive_more::{Display, Error as Err};
use serde::Serialize;
use serde_json::json;

#[derive(Serialize)]
struct ErrorJson {
    msg: &'static str,
}

#[derive(Debug, Display, Err, Serialize)]
pub enum ServerError {
    #[display(fmt = "{msg}")]
    InternalServerError { msg: &'static str , detail: String},

    #[display(fmt = "{msg}")]
    BadRequestError { msg: &'static str , detail: String},

    #[display(fmt = "{msg}")]
    UnauthorizedError { msg: &'static str , detail: String},

    NotFound
}



impl error::ResponseError for ServerError {
    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        let (status_code, error_json) = self.return_body();
        HttpResponse::build(status_code)
            .insert_header(ContentType::json())
            .json(error_json)
    }
}

impl ServerError {
    fn return_body(&self) -> (actix_web::http::StatusCode, serde_json::Value) {
        match self {
            Self::InternalServerError{msg: m, detail: d} => (StatusCode::INTERNAL_SERVER_ERROR, json!({"msg":m, "detail":d})),
            Self::BadRequestError{msg: m, detail: d} => (StatusCode::BAD_REQUEST, json!({"msg":m, "detail":d})),
            Self::UnauthorizedError{msg: m, detail: d} => (StatusCode::UNAUTHORIZED, json!({"msg":m, "detail":d})),
            Self::NotFound => (StatusCode::NOT_FOUND, json!({"msg":"Not Found"}))
        }
    }
}
