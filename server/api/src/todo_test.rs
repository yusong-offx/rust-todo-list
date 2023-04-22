#![allow(dead_code)]

use actix_web::{
    App, web, test, http::header::ContentType,
};
use actix_web_httpauth::middleware::HttpAuthentication;
use serde::{Serialize, Deserialize};
use chrono::{NaiveDate, NaiveDateTime};
use super::{user, auth, todo};
use super::user_test::*;

#[derive(Serialize, Clone)]
struct TodoForm {
    name: String,
    contents: Option<String>,
    due_date: Option<NaiveDate>,
    completed: bool,
}

#[derive(Deserialize)]
struct TodoReturnForm {
    id: i32,
    user_id: i32,
    name: String,
    contents: Option<String>,
    due_date: Option<NaiveDate>,
    completed: bool,
    created_at: NaiveDateTime,
}

#[actix_web::test]
async fn test_todo() {
    let app = test::init_service(
        App::new()
            .app_data(web::Data::new(db_connect().await.clone()))
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
    ).await;

    // Create user
    let test_data = UserSignUpForm{
        username : "todo",
        password : "World123!!",
        email : "todo@gmail.com",
    };
    let req = test::TestRequest::post().uri("/user/register")
        .insert_header(ContentType::form_url_encoded())
        .set_form(test_data.clone())
        .to_request();

    let _: UserSignUpReturnForm = test::call_and_read_body_json(&app, req).await;

    // Login
    let login_data: UserLoginForm = UserLoginForm {
        username : "todo",
        password : "World123!!",
    };
    let req = test::TestRequest::post().uri("/user/login")
        .insert_header(ContentType::form_url_encoded())
        .set_form(login_data)
        .to_request();
    let login_resp: UserLoginReturnForm = test::call_and_read_body_json(&app, req).await;

    // CREATE TODO //
    // Valid
    let test_data = TodoForm{
        name: "New Title".to_owned(),
        contents: Some("New Contents".to_owned()),
        due_date: Some(NaiveDate::from_ymd_opt(2023, 10, 18).unwrap()),
        completed: true,
    };
    let req = test::TestRequest::post()
        .uri(format!("/user/{}/todo/register", login_resp.user.id).as_str())
        .insert_header(("Authorization", format!("Bearer {}", login_resp.access_token)))
        .set_json(test_data.clone())
        .to_request();
    let todo_resp: TodoReturnForm= test::call_and_read_body_json(&app, req).await;

    assert_eq!(test_data.name, todo_resp.name);
    assert_eq!(test_data.contents, todo_resp.contents);
    assert_eq!(test_data.due_date, todo_resp.due_date);
    assert_eq!(test_data.completed, todo_resp.completed);

    // Invalid
    let test_datas = [
        // No name
        TodoForm{
            name: "".to_owned(),
            contents: Some("Contents".to_owned()),
            due_date: Some(NaiveDate::from_ymd_opt(2023, 10, 18).unwrap()),
            completed: true
        },
        // Name over 100
        TodoForm{
            name: "a".repeat(101).to_owned(),
            contents: Some("Contents".to_owned()),
            due_date: Some(NaiveDate::from_ymd_opt(2023, 10, 18).unwrap()),
            completed: true
        },
        // Contents over 255
        TodoForm{
            name: "Name".to_owned(),
            contents: Some("a".repeat(256).to_owned()),
            due_date: Some(NaiveDate::from_ymd_opt(2023, 10, 18).unwrap()),
            completed: true
        },
    ];
    for data in test_datas {
        let req = test::TestRequest::post()
        .uri(format!("/user/{}/todo/register", login_resp.user.id).as_str())
        .insert_header(("Authorization", format!("Bearer {}", login_resp.access_token)))
        .set_json(data)
        .to_request();
        let resp = test::call_service(&app, req).await;

        assert_eq!(resp.status(), 400);
    }

    // MODIFY TODO //
    let test_data = TodoForm{
        name: "Change Title".to_owned(),
        contents: Some("Change Contents".to_owned()),
        due_date: Some(NaiveDate::from_ymd_opt(2024, 11, 19).unwrap()),
        completed: false,
    };
    let req = test::TestRequest::put()
        .uri(format!("/user/{}/todo/{}", login_resp.user.id, todo_resp.id).as_str())
        .insert_header(("Authorization", format!("Bearer {}", login_resp.access_token)))
        .set_json(test_data.clone())
        .to_request();
    let modify_resp: TodoReturnForm = test::call_and_read_body_json(&app, req).await;

    assert_eq!(todo_resp.id, modify_resp.id);
    assert_eq!(test_data.name, modify_resp.name);
    assert_eq!(test_data.contents, modify_resp.contents);
    assert_eq!(test_data.due_date, modify_resp.due_date);
    assert_eq!(test_data.completed, modify_resp.completed);
    assert_eq!(todo_resp.created_at, modify_resp.created_at);

    // Invalid todo_id
    let req = test::TestRequest::put()
        .uri(format!("/user/{}/todo/0", login_resp.user.id).as_str())
        .insert_header(("Authorization", format!("Bearer {}", login_resp.access_token)))
        .set_json(test_data.clone())
        .to_request();
    let resp = test::call_service(&app, req).await;
    
    assert_eq!(resp.status(), 404);

    // REMOVE TODO //
    let req = test::TestRequest::delete()
        .uri(format!("/user/{}/todo/{}", login_resp.user.id, todo_resp.id).as_str())
        .insert_header(("Authorization", format!("Bearer {}", login_resp.access_token)))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 204);

    // FETCH TODO //
    // Insert mock data
    let mut mock_datas = Vec::with_capacity(20);
    for i in 1..=20 {
        let test_data = TodoForm{
            name: format!("Todo {}", i),
            contents: Some(format!("Contents {}", i)),
            due_date: Some(NaiveDate::from_ymd_opt(2023, 10, 18).unwrap()),
            completed: true,
        };
        let req = test::TestRequest::post()
            .uri(format!("/user/{}/todo/register", login_resp.user.id).as_str())
            .insert_header(("Authorization", format!("Bearer {}", login_resp.access_token)))
            .set_json(test_data)
            .to_request();
        let todo_resp: TodoReturnForm = test::call_and_read_body_json(&app, req).await;
        mock_datas.push(todo_resp);
    }

    // Fetch
    let mut fetch_datas = Vec::with_capacity(20);
    for i in 0..4 {
        let req = test::TestRequest::get()
            .uri(format!("/user/{}/todo?page={}", login_resp.user.id, i).as_str())
            .insert_header(("Authorization", format!("Bearer {}", login_resp.access_token)))
            .to_request();
        let todo_resp: Vec<TodoReturnForm> = test::call_and_read_body_json(&app, req).await;
        fetch_datas.extend(todo_resp);
    }

    // Check
    for (a, b) in mock_datas.iter().rev().zip(fetch_datas) {
        println!("{} | {}", a.id, b.id);
        assert_eq!(a.id, b.id);
        assert_eq!(a.name, b.name);
        assert_eq!(a.contents, b.contents);
        assert_eq!(a.created_at, b.created_at);
        assert_eq!(a.completed, b.completed);
        assert_eq!(a.due_date, b.due_date);
    }

    // Delete test user
    // Todo list also remove all cause by on_delete_cascade
    let req = test::TestRequest::delete()
        .uri(format!("/user/{}", login_resp.user.id).as_str())
        .insert_header(("Authorization", format!("Bearer {}", login_resp.access_token)))
        .to_request();
    test::call_service(&app, req).await;
}