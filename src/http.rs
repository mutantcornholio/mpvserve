use rocket::http::Cookie;
use rocket::request::{FromRequest, Outcome};
use rocket::serde::Serialize;
use rocket::{http, Request};
use shrinkwraprs::Shrinkwrap;
use uuid::Uuid;

#[derive(Debug)]
pub enum NeverHappensError {
    MissingHostHeader,
}

#[derive(Debug)]
pub struct HostHeader(String);

impl HostHeader {
    pub fn to_string(&self) -> &String {
        match self {
            HostHeader(s) => s,
        }
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for HostHeader {
    type Error = NeverHappensError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let host = request.headers().get_one("host");
        match host {
            Some(host) => Outcome::Success(HostHeader(host.to_string())),
            None => Outcome::Failure((
                http::Status::Unauthorized,
                NeverHappensError::MissingHostHeader,
            )),
        }
    }
}

#[derive(Debug, Serialize, Shrinkwrap)]
pub struct UserId(String);

impl UserId {
    pub fn to_string(&self) -> &String {
        match self {
            UserId(s) => s,
        }
    }
}
#[rocket::async_trait]
impl<'r> FromRequest<'r> for UserId {
    type Error = NeverHappensError;

    async fn from_request(request: &'r Request<'_>) -> Outcome<Self, Self::Error> {
        let cookies = request.cookies();

        match cookies.get("mpvserve_user_id") {
            Some(mpvserve_user_id) => {
                Outcome::Success(UserId(mpvserve_user_id.value().to_string()))
            }
            None => {
                let mpvserve_user_id = Uuid::new_v4().to_string();
                cookies.add(Cookie::new("mpvserve_user_id", mpvserve_user_id.clone()));
                Outcome::Success(UserId(mpvserve_user_id))
            }
        }
    }
}
