
#![allow(dead_code)]

use actix_web::{
    App, web, test, http::header::ContentType,
};
use actix_web_httpauth::middleware::HttpAuthentication;
use serde::{Serialize, Deserialize};
use chrono::naive::NaiveDateTime;
use super::{user, auth};


// Database connect
pub async fn db_connect() -> super::AppState {
    let conn = entity::database_connect().await.unwrap();
    super::AppState {conn}
}

#[derive(Serialize, Clone)]
pub  struct UserSignUpForm {
    pub username: &'static str,
    pub password: &'static str,
    pub email: &'static str,
}

#[derive(Deserialize, Clone, Debug)]
pub struct UserSignUpReturnForm {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub created_at: NaiveDateTime,
}

#[derive(Serialize)]
pub struct UserLoginForm {
    pub username: &'static str,
    pub password: &'static str,
}

#[derive(Deserialize)]
pub struct UserLoginReturnForm {
    pub access_token: String,
    pub user: UserSignUpReturnForm,
}

#[actix_web::test]
async fn test_user() {
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
            )
    ).await;

    // SIGNUP USER //
    // Valid test
    let test_data = UserSignUpForm{
        username : "hello",
        password : "World123!!",
        email : "mymail@gmail.com",
    };

    let req = test::TestRequest::post().uri("/user/register")
        .insert_header(ContentType::form_url_encoded())
        .set_form(test_data.clone())
        .to_request();

    let signup_resp: UserSignUpReturnForm = test::call_and_read_body_json(&app, req).await;
    
    assert_eq!(signup_resp.username, test_data.username);
    assert_eq!(signup_resp.email, test_data.email);

    // Invalid Test
    let test_datas = [
        // Username over 20 char
        UserSignUpForm{
            username : "hellohellohellohellohello",
            password : "World123!!",
            email : "mymail2@gmail.com",

        },
        // Password no special char
        UserSignUpForm{
            username : "hello4",
            password : "World123",
            email : "mymail3@gmail.com",

        },
        // Password no number char
        UserSignUpForm{
            username : "hello5",
            password : "World!!!",
            email : "mymail4@gmail.com",

        },
        // Password no upper char
        UserSignUpForm{
            username : "hello6",
            password : "world123!!",
            email : "mymail5@gmail.com",

        },
        // Password no lower char
        UserSignUpForm{
            username : "hello7",
            password : "WORLD123!!",
            email : "mymail6@gmail.com",

        },
        // Password under 8
        UserSignUpForm{
            username : "hello8",
            password : "world1!",
            email : "mymail7@gmail.com",

        },
        // Password over 20
        UserSignUpForm{
            username : "hello9",
            password : "world1!world1!world1!world1!",
            email : "mymail8@gmail.com",

        },
    ];

    for data in test_datas.iter() {
        let req = test::TestRequest::post().uri("/user/register")
            .insert_header(ContentType::form_url_encoded())
            .set_form(data)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 400);
    }

    let test_datas = [
        // Same username
        UserSignUpForm{
            username : "hello",
            password : "World123!!",
            email : "mymail1@gmail.com",

        },
        // Same email
        UserSignUpForm{
            username : "hello2",
            password : "World123!!",
            email : "mymail@gmail.com",

        },
    ];

    for data in test_datas.iter() {
        let req = test::TestRequest::post().uri("/user/register")
            .insert_header(ContentType::form_url_encoded())
            .set_form(data)
            .to_request();

        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 500);
    }

    // LOGIN USER //
    // Valid test
    let login_data: UserLoginForm = UserLoginForm {
        username : "hello",
        password : "World123!!",
    };
    let req = test::TestRequest::post().uri("/user/login")
        .insert_header(ContentType::form_url_encoded())
        .set_form(login_data)
        .to_request();
    let login_resp: UserLoginReturnForm = test::call_and_read_body_json(&app, req).await;

    assert_eq!(signup_resp.id, login_resp.user.id);
    assert_eq!(signup_resp.username, login_resp.user.username);
    assert_eq!(signup_resp.email, login_resp.user.email);
    assert_eq!(signup_resp.created_at, login_resp.user.created_at);

    // Invalid test
    let test_datas = [
        // Wrong username
        UserLoginForm {
            username : "hello1",
            password : "World123!!",
        },
        // Wrong password
        UserLoginForm {
            username : "hello",
            password : "World123!",
        }
    ];
    for data in test_datas {
        let req = test::TestRequest::post().uri("/user/login")
            .insert_header(ContentType::form_url_encoded())
            .set_form(data)
            .to_request();
        let login_resp = test::call_service(&app, req).await;
        assert_eq!(login_resp.status(), 401);
    }

    // MODIFY USER //
    let test_data = UserSignUpForm{
        username: "Does not matter",
        password: "Hello123!!",
        email: "newemail@gmail.com"
    };
    
    let req = test::TestRequest::put()
        .uri(format!("/user/{}", login_resp.user.id).as_str())
        .insert_header(("Authorization", format!("Bearer {}", login_resp.access_token)))
        .insert_header(ContentType::form_url_encoded())
        .set_form(test_data.clone())
        .to_request();
    let resp: UserSignUpReturnForm = test::call_and_read_body_json(&app, req).await;

    assert_eq!(resp.id, login_resp.user.id);
    assert_eq!(resp.username, login_resp.user.username);
    assert_eq!(resp.email, test_data.email);
    assert_eq!(resp.created_at, login_resp.user.created_at);

    // DELETE USER //
    let req = test::TestRequest::delete()
        .uri(format!("/user/{}", login_resp.user.id).as_str())
        .insert_header(("Authorization", format!("Bearer {}", login_resp.access_token)))
        .to_request();
    let resp = test::call_service(&app, req).await;

    assert_eq!(resp.status(), 204)
}