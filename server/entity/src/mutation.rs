use super::entities::{users, todos};
use bcrypt::DEFAULT_COST;
use sea_orm::*;
use serde::Deserialize;
use validator::Validate;
use myerror::ServerError;

/// Data for login.
#[derive(Deserialize)]
pub struct LoginForm {
    pub username: String,
    pub password: String,
}

/// Data for modify.
#[derive(Deserialize)]
pub struct ModifyForm {
    #[serde(skip_deserializing)]
    pub id: i32,

    pub password: String,
    pub email: String,
}

/// Communicate function class for database.
pub struct Mutation;

impl Mutation {
    /// Password crypt for save in database.
    fn password_hashing(pwd: String) -> Result<String, ServerError> {
        bcrypt::hash(pwd, DEFAULT_COST)
        .map_err(|e|
            ServerError::InternalServerError { msg: "Password crypt error", detail: e.to_string() }
        )
    }

    /// Validated password will be bcrypt.
    /// Password must be 8..=20 letter, at least 1 upper, lower and special($@$!%*?&) letter, number.
    pub async fn create_user(db: &DbConn, data: users::Model) -> Result<users::Model, ServerError> {
        // Validation detail refer to users::Model.
        data.validate()
            .map_err(|e|
                ServerError::BadRequestError { msg: "Validation error", detail: e.to_string() }
            )?;

        // Bcrypt hash password.
        let hashed_password = Self::password_hashing(data.password)?;

        // Insert sign up user data to database.
        users::ActiveModel {
            username: Set(data.username),
            password: Set(hashed_password),
            email: Set(data.email),
            ..Default::default()
        }
        .insert(db).await
        .map_err(|e| 
            ServerError::InternalServerError { msg: "Database insert error", detail: e.to_string() }
        )
    }
    
    /// Username can not change.
    /// Get user data by user_id.
    /// Modify data by form data. You can not change username.
    pub async fn update_user(db: &DbConn, data: ModifyForm) -> Result<users::Model, ServerError> {
        // Get user
        let mut model = users::Entity::find_by_id(data.id)
            .one(db).await
            .map_err(|e|
                ServerError::InternalServerError { msg: "Database fetch error", detail: e.to_string() }
            )?.unwrap();
        
        // Validate
        model.password = data.password;
        model.email = data.email;
        model.validate()
            .map_err(|e|
                ServerError::BadRequestError { msg: "Validation error", detail: e.to_string() }
            )?;

        // Bcrypt hash password.
        model.password = Self::password_hashing(model.password)?;

        // Update
        model.into_active_model().reset_all()
        .update(db).await
        .map_err(|e|
            ServerError::InternalServerError { msg: "Database update error", detail: e.to_string() }
        )
    }

    /// Get user data by username.
    /// In this function, only return user data.
    pub async fn login(db: &DbConn, data: LoginForm) -> Result<Option<users::Model>, ServerError> {
        users::Entity::find()
        .filter(users::Column::Username.eq(data.username))
        .one(db).await
        .map_err(|e|
            ServerError::InternalServerError { msg: "Database fetch error", detail: e.to_string() }
        )
    }

    /// Delete user by user_id.
    pub async fn delete_user(db: &DbConn, user_id: i32) -> Result<DeleteResult, ServerError>{
        users::Entity::delete_by_id(user_id).exec(db).await
            .map_err(|e|
                ServerError::InternalServerError { msg: "Database delete error", detail: e.to_string() }
            )
    }

    /// Get todo.
    pub async fn get_todo(db: &DbConn, user_id: i32, page: u64) -> Result<Vec<todos::Model>, ServerError> {
        // Each page's number of contents.
        let contents_per_page = 5;

        // Filter previous pages and limit todo datas.
        let start = page.saturating_mul(contents_per_page);

        // Fetch models.
        todos::Entity::find()
            .filter(todos::Column::UserId.eq(user_id))
            .order_by_desc(todos::Column::Id)
            .limit(contents_per_page)
            .offset(start)
            .all(db).await
            .map_err(|e|
                ServerError::InternalServerError { msg: "Database fetch error", detail: e.to_string() }
            )
    }

    /// Create todo.
    pub async fn create_todo(db: &DbConn, data: todos::Model) -> Result<todos::Model, ServerError> {
        // Validation detail refer to todos::Model.
        data.validate().map_err(|e|
            ServerError::BadRequestError { msg: "Validation error", detail: e.to_string() }
        )?;

        // Insert new todo data to database.
        todos::ActiveModel{
            user_id: Set(data.user_id),
            name: Set(data.name),
            contents: Set(data.contents),
            due_date: Set(data.due_date),
            completed: Set(data.completed),
            ..Default::default()
        }
        .insert(db).await
        .map_err(|e|
            ServerError::InternalServerError { msg: "Database insert error", detail: e.to_string() }
        )
    }

    /// Update todo.
    pub async fn update_todo(db: &DbConn, data: todos::Model) -> Result<todos::Model, ServerError> {
        // Get todo by user_id and todo_id
        let model = todos::Entity::find_by_id(data.id)
            .filter(todos::Column::UserId.eq(data.user_id))
            .one(db).await
            .map_err(|e|
                ServerError::InternalServerError { msg: "Database fetch error", detail: e.to_string() }
            )?;
        
        // No exist
        if model.is_none() {
            return Err(ServerError::NotFound)
        }

        // Validate data
        let mut model = model.unwrap();
        model.name = data.name;
        model.contents = data.contents;
        model.due_date = data.due_date;
        model.completed = data.completed;

        model.validate()
        .map_err(|e|
            ServerError::BadRequestError { msg: "validation error", detail: e.to_string() }
        )?;
        
        // Update        
        model.into_active_model().reset_all()
        .update(db).await
        .map_err(|e|
            ServerError::InternalServerError { msg: "Database update error", detail: e.to_string() }
        )
    }

    /// Delete todo.
    pub async fn delete_todo(db: &DbConn, user_id: i32, todo_id: i32) -> Result<DeleteResult, ServerError> {
        // Search delete model.
        let delete_model = todos::Entity::find_by_id(todo_id)
            .filter(todos::Column::UserId.eq(user_id))
            .one(db).await
            .map_err(|e|
                ServerError::InternalServerError { msg: "Database delete error", detail: e.to_string() }
            )?;

        // Not exist.
        if delete_model.is_none() {
            return Err(ServerError::NotFound)
        }

        // Exist.
        delete_model.unwrap()
            .delete(db).await
            .map_err(|e|
                ServerError::InternalServerError { msg: "Database delete error", detail: e.to_string() }
            )
    }
}
