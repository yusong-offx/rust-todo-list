# Rust Simple User Todos
Rust를 사용하여 간단한 Todo-list API를 작성하였습니다.

## Skills
- [Actix-Web](https://actix.rs/)(rust)
- [SeaORM](https://github.com/SeaQL/sea-orm)(postgresql)

## Features
- ORM 사용
  -  SeaORM 사용
- JWT Auth
  - Refresh token은 구현하지 않았음
  - token의 만료시간은 15분
  - jwt claims에 user_id를 저장하고 path의 user_id와 비교  
    (e.g. jwt's {___user_id___} == localhost:8080/user/{___user_id___})
- Bcrypt를 사용한 비밀번호 저장
  - 비밀번호 취급시 bcrypt 저장 및 복호화
- docker-compose 사용
  - rust app, postgresql, 필요시 pgadmin 을 실행
- test code 작성 
  - 통합(Integration) 테스트만 작성
  - coverage x%

## Schemas
![schemas d2.svg image](./postgre-db/schemas.svg)   
**[raw sql schema](./postgre-db/schemas.sql)*

## API
```
http://localhost:8080/user/register
```
- __POST__  
    아래와 같은 데이터를 요구합니다.
    ```rust
    // Content-Type: Application/x-www-form-urlencode
    {
        // 로그인시 식별자로 사용되며
        // 1-20자 사이의 문자열이며 중복은 허용하지 않습니다.
        "username" : String,

        // 8-20자
        // 최소 한자 이상의 영대문자
        // 최소 한자 이상의 영소문자
        // 최소 한자 이상의 특수문자($@$!%*?&)
        // 를 포함해야합니다.
        "password" : String, 

        // 따로 이메일 형식의 검증을 하지않고 있으며
        // 중복은 허용하지 않습니다.
        "email" : String
    }
    ```
    아래와 같은 데이터를 반환합니다.
    ```rust
    // Status Code : 201
    {
        "id" : i32,
        "username" : String,
        "email" : String,
        "created_at" : Datetime
    }
    ```
<br/>

```
http://localhost:8080/user/login
```
- __POST__   
    다음과 같은 데이터를 요구합니다
    ```rust
    // Content-Type: Application/x-www-form-urlencode
    {
        "username" : String,
        "password" : String
    }
    ```
    다음과 같은 데이터를 반환합니다.
    ```rust
    // Status Code : 200
    {
        "access_token" : String,
        "user" : {
            "id" : i32,
            "username" : String,
            "email" : String,
            "created_at" : Datetime
        }   
    }
    ```
<br/>

```
http://localhost:8080/user/{user_id}
```
- __PUT__   
    다음과 같은 데이터를 요구합니다.   
    username은 변경 할수 없습니다.   
    비밀번호와 이메일만 변경가능하며 반드시 변수 모두 입력해야합니다.
    ```rust
    // Content-Type: Application/x-www-form-urlencode
    {
        "password" : String,
        "email" : String
    }
    ```
    다음과 같은 데이터를 반환합니다.
    ```rust
    // Status Code : 200
    {
        "id" : i32,
        "username" : String,
        "email" : String,
        "created_at" : Datetime
    }
    ```
- __DELETE__   
    상태코드(status code) 204(No content)를 반환합니다.
<br/>

```
http://localhost:8080/user/{user_id}/todo/register
```
- __POST__   
    다음과 같은 데이터를 요구합니다.
    ```rust
    ```
    다음과 같은 데이터를 반환합니다.
    ```rust
    ```
<br/>

```
http://localhost:8080/user/{user_id}/todo?page=<u64>
```
- __GET__   
    다음과 같은 데이터를 요구합니다.
    ```rust
    ```
    다음과 같은 데이터를 반환합니다.
    ```rust
    ```
<br/>

```
http://localhost:8080/user/{user_id}/todo/{todo_id}
```
- __PUT__   
    다음과 같은 데이터를 요구합니다.
    ```rust
    ```
    다음과 같은 데이터를 반환합니다.
    ```rust
    ```
- __DELETE__   
    다음과 같은 데이터를 요구합니다.
    ```rust
    ```
    다음과 같은 데이터를 반환합니다.
    ```rust
    ```