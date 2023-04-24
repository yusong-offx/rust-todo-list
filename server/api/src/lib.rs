use env_logger::Env;
use actix_web::{
    middleware::{Logger},
    HttpServer, App, web
};
use actix_web_httpauth::middleware::HttpAuthentication;
use sea_orm::DatabaseConnection;
use entity;

mod user;
mod todo;
mod auth;

#[cfg(test)]
pub mod user_test;
#[cfg(test)]
pub mod todo_test;


#[derive(Clone)]
pub struct AppState {
    pub conn: DatabaseConnection
}

#[actix_web::main]
async fn server_run() {
    env_logger::init_from_env(Env::default().default_filter_or("info"));
    let logger_format = r#"%a %t "%r" %s %b "%{Referer}i" "%{User-Agent}i" %Dms"#;
    let conn = entity::database_connect().await.unwrap();
    let state = AppState {conn};
    HttpServer::new(move ||
        App::new()
            .wrap(Logger::new(logger_format))
            .app_data(web::Data::new(state.clone()))
            .service(user::signup_user)
            .service(user::login_user)
            .service(
                web::scope("/user/{user_id}")
                    .wrap(HttpAuthentication::bearer(auth::jwt_validator))
                    .service(user::modify_user)
                    .service(user::withdrawal_user)
                    .service(
                        web::scope("/todo")
                            .service(todo::fetch_todos)
                            .service(todo::create_todo)
                            .service(todo::modify_todo)
                            .service(todo::remove_todo)
                    )
            )

    )
    .bind(("0.0.0.0", 8080)).unwrap()
    .run()
    .await.unwrap();
}

pub fn main() {
    server_run();
}