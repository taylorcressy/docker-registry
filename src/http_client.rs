
use std::{collections::HashMap};
use futures::TryFutureExt;
use reqwest::{RequestBuilder};

#[derive(Debug)]
pub struct Error {
    status_code: u16
}

impl Error {
    pub fn status_code(self) -> u16 { self.status_code }
}

lazy_static! {
    static ref CLIENT: reqwest::Client = reqwest::Client::new();    
}

pub struct AuthContext<'a> {
    pub username: &'a String,
    pub password: &'a String,
}

pub async fn get<'a>(
    url: String, 
    headers: Option<HashMap<&str, &str>>,
    auth_context: Option<AuthContext<'a>>,
) ->  Result<reqwest::Response, Box<Error>> {
    if std::env::var("DOCKER_REG_VERBOSE").is_ok() {
        println!("GET: {}", url);
    }

    let builder = CLIENT.get(url);
    return request(builder, headers, auth_context).await;
}

pub async fn delete<'a>(
    url: String,
    headers: Option<HashMap<&str, &str>>,
    auth_context: Option<AuthContext<'a>>,
) ->  Result<reqwest::Response, Box<Error>> {
    let builder = CLIENT.delete(url);
    return request(builder, headers, auth_context).await;
}

async fn request<'a>(
    mut request_builder: RequestBuilder, 
    headers: Option<HashMap<&str, &str>>,
    auth_context: Option<AuthContext<'a>>,
) ->  Result<reqwest::Response, Box<Error>> {

    if let Some(headers_unwrapped) = headers {
        for (key, value) in headers_unwrapped {
            request_builder = request_builder.header(key, value)
        }
    }

    if let Some(auth) = auth_context {
        if std::env::var("DOCKER_REG_VERBOSE").is_ok() {
            println!("Using basic auth {}", auth.username);
        }
        request_builder = request_builder.basic_auth(auth.username, Some(auth.password));
    }
    
    
    let resp = request_builder
        .send()
        .map_err(|e| {
            if std::env::var("DOCKER_REG_VERBOSE").is_ok() {
                println!("Received error: {}", e.to_string());
            }
            Error { 
                status_code: e.status().map(|status| status.as_u16()).unwrap_or(0)
             }
        })
        .await?;
    
    if std::env::var("DOCKER_REG_VERBOSE").is_ok() {
        println!("{:#?}", resp);
    }

    Ok(resp)
}