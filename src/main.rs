use std::{sync::Arc}; // Thing means that you will be clonning a lot? What?
use std::collections::HashMap;
use std::convert::Infallible;
use auth::{with_auth, Role};
use error::Error::*;
use serde::{Deserialize, Serialize};
use warp::{reject, reply, Filter, Rejection, Reply};
use tokio::main;

type Result<T> = std::result::Result<T, error::Error>;
type WebResult<T> = std::result::Result<T, Rejection>;
type Users = Arc<HashMap<String, User>>; // Yeah... Here we clonning within Arc HashMap and HashMap will store string value and information of Users that described in ~User~ struct  
				     
mod auth;
mod error;

// Will stored in Session memory as a hash map
#[derive(Clone)] // Here will be a lot of operations with users, where we need to have a multiple copies of users. So we will derive ~clone~. !!!
pub struct User {
    pub uid: String,
    pub email: String,
    pub password: String,
    pub role: String,
}

#[derive(Deserialize)] // For JSON understanding
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}

#[derive(Serialize)] // For JSON understanding
pub struct LoginResponse {
    pub token: String,
}

#[main] // To enable async function
async fn main() {
    let users = Arc::new(init_users()); // init_users() function will create users at the initing of program. It means when we start the program for the first time, we already will have 2 users in hash map

    let login_route = warp::path("login") // If you have login route
	.and(warp::post())
	.and(with_users(users.clone()))
	.and(warp::body::json())
	.and_then(login_handler); // Therefore you need to have login handler
    
    let user_route = warp::path!("user")
	.and(with_auth(Role::User))
	.and_then(user_handler);
    
    let admin_route = warp::path!("admin")
	.and(with_auth(Role::Admin))
	.and_then(admin_handler);

    let routes = login_route
	.or(user_route)
	.or(admin_route)
	.recover(error::handle_rejection);

    warp::serve(routes).run(([127, 0, 0, 1], 8000)).await;
}

fn with_users(users: Users) -> impl Filter<Extract = (Users,), Error = Infallible> + Clone {
    warp::any().map(move || users.clone())
}

pub async fn login_handler(users: Users, body: LoginRequest) -> WebResult<impl Reply> {
    match users
	.iter()
	.find(|(_uid, user)| user.email == body.email && user.password == body.password)
    {
	Some((uid, user)) => {
	    let token = auth::create_jwt(&uid, &Role::from_str(&user.role))
		.map_err(|e| reject::custom(e))?;
	    Ok(reply::json(&&LoginResponse { token }))
	}
	
	None => Err(reject::custom(WrongCredentialsError)),
	
    }
}

pub async fn user_handler(uid: String) -> WebResult<impl Reply> {
 Ok(format!("Hello User {}", uid))
}

pub async fn admin_handler(uid: String) -> WebResult<impl Reply> {
 Ok(format!("Hello Admin {}", uid))
}

fn init_users() -> HashMap<String, User> {
 let mut map = HashMap::new();
 map.insert(
     String::from("1"),
     User {
	 uid: String::from("1"),
	 email: String::from("user@userland.com"),
	 password: String::from("12345678"),
	 role: String::from("User")
     },
 );

 map.insert(
     String::from("2"),
     User {
	 uid: String::from("2"),
	 email: String::from("berezhnev@berezhnevland.com"),
	 password: String::from("12345678"),
	 role: String::from("Admin")
     },
 );

 map
}

