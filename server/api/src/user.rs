use actix_web::{
    post, put, delete, HttpResponse, web
};
use sea_orm::TryIntoModel;
use super::{
    auth, AppState,
    entity::{
        entities::users,
        mutation::*
    },
};
use bcrypt;
use serde_json::json;
use myerror::ServerError;

#[post("/user/register")]
async fn signup_user(
    state: web::Data<AppState>, 
    user_form: web::Form<users::Model>
) -> Result<HttpResponse, ServerError> {
    let resp = Mutation::create_user(&state.conn, user_form.into_inner()).await?;
    Ok(HttpResponse::Created().json(resp))
}

#[post("/user/login")]
async fn login_user(
    state: web::Data<AppState>,
    login_form: web::Form<LoginForm>
) -> Result<HttpResponse, ServerError> {
    // Get login user data by username.
    let login_password = login_form.password.clone();
    let user_data = Mutation::login(&state.conn, login_form.into_inner()).await?;

    // User validation
    match user_data {
        // No user
        None => {
            Err(ServerError::UnauthorizedError { msg: "Username does not exist", detail: "No user exists".to_owned() })
        },
        // User Exist
        Some(user) => {
            // Password check
            if login_validate(login_password, &user.password).await? {
                let token = auth::generate_token(user.id).await?;
                return Ok(HttpResponse::Ok().json(json!({
                    "user" : user,
                    "access_token" : token
                })))
            }
            Err(ServerError::UnauthorizedError { msg: "Wrong password", detail: "Password is not same".to_owned() })
        }
    }
}

async fn login_validate(login_password: String, hash_password: &str) -> Result<bool, ServerError> {
    bcrypt::verify(login_password, hash_password)
        .map_err(|e|
            ServerError::InternalServerError { msg: "Password decrypt error", detail: e.to_string() }
        )
}

#[put("")]
async fn modify_user(
    state: web::Data<AppState>,
    req_data: web::ReqData<auth::JwtClaim>,
    user_form: web::Form<ModifyForm>
) -> Result<HttpResponse, ServerError> {
    let mut user_form = user_form.into_inner();
    user_form.id = req_data.user_id;
    let model = Mutation::update_user(&state.conn, user_form).await?;
    Ok(HttpResponse::Ok().json(model.try_into_model().unwrap()))
}

#[delete("")]
async fn withdrawal_user(
    state: web::Data<AppState>,
    req_data: web::ReqData<auth::JwtClaim>,
) -> Result<HttpResponse, ServerError> {
    Mutation::delete_user(&state.conn, req_data.user_id).await?;
    Ok(HttpResponse::NoContent().finish())
}   