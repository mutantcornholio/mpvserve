use rocket::request::{FromRequest, Outcome};
use rocket::{http, Request};

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
