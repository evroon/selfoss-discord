use std::fmt;

#[derive(Debug)]
pub enum RequestError {
    Reqwest(reqwest::Error),
    ReqwestMiddleware(reqwest_middleware::Error),
    Serde(serde_json::Error),
}

impl From<reqwest::Error> for RequestError {
    fn from(value: reqwest::Error) -> Self {
        RequestError::Reqwest(value)
    }
}

impl From<reqwest_middleware::Error> for RequestError {
    fn from(value: reqwest_middleware::Error) -> Self {
        RequestError::ReqwestMiddleware(value)
    }
}

impl From<serde_json::Error> for RequestError {
    fn from(value: serde_json::Error) -> Self {
        RequestError::Serde(value)
    }
}

impl fmt::Display for RequestError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RequestError::Reqwest(ref e) => e.fmt(f),
            RequestError::ReqwestMiddleware(ref e) => e.fmt(f),
            RequestError::Serde(ref e) => e.fmt(f),
        }
    }
}
