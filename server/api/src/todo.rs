use actix_web::{
    get, post, put, delete, HttpResponse, web
};
use serde::Deserialize;
use super::{
    AppState,
    entity::{
        entities::todos,
        mutation::*
    },
};
use myerror::ServerError;

#[derive(Deserialize)]
struct Page {
    page: u64
}

#[get("")]
async fn fetch_todos(
    state: web::Data<AppState>,
    user_id: web::Path<i32>,
    page: web::Query<Page>
) -> Result<HttpResponse, ServerError> {
    let models = Mutation::get_todo(&state.conn, user_id.into_inner(), page.page).await?;
    Ok(HttpResponse::Ok().json(models))
}

#[post("/register")]
async fn create_todo(
    state: web::Data<AppState>,
    user_id: web::Path<i32>,
    todo_data: web::Json<todos::Model>,
) -> Result<HttpResponse, ServerError> {
    let mut todo_data = todo_data.into_inner();
    todo_data.user_id = user_id.into_inner();
    let model = Mutation::create_todo(&state.conn, todo_data).await?;
    Ok(HttpResponse::Created().json(model))
}

#[put("/{todo_id}")]
async fn modify_todo(
    state: web::Data<AppState>,
    path_para: web::Path<(i32, i32)>,
    todo_data: web::Json<todos::Model>,
) -> Result<HttpResponse, ServerError> {
    let (user_id, todo_id) = path_para.into_inner();
    let mut todo_data = todo_data.into_inner();
    todo_data.id = todo_id;
    todo_data.user_id = user_id;
    let model = Mutation::update_todo(&state.conn, todo_data).await?;
    Ok(HttpResponse::Created().json(model))
}

#[delete("/{todo_id}")]
async fn remove_todo(
    state: web::Data<AppState>,
    path_para: web::Path<(i32, i32)>,
) -> Result<HttpResponse, ServerError> {
    let (user_id, todo_id) = path_para.into_inner();
    Mutation::delete_todo(&state.conn, user_id, todo_id).await?;
    Ok(HttpResponse::NoContent().finish())
}
