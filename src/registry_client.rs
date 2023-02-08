use std::fmt::format;

use crate::http_client::{get, AuthContext, self};

const ROOT_URI: &str = "/v2";
const LIST_CONTAINER_URI: &str = "/_catalog";
const GET_TAGS: &str = "/tags/list";
const MANIFEST: &str = "/manifests/";

#[derive(Debug, Clone)]
pub struct CommandContext<'a> {
    pub username: &'a String,
    pub password: &'a String,
    pub proto: Option<String>,  // https/http. If not present, https assumed.
    pub image_name: Option<String>,
    pub tag: Option<String>,
    pub digest: Option<String>,
}

pub async fn is_v2_supported<'a>(domain: &String, command_context: &'a CommandContext<'a>) -> Result<(), String> {
    let url = format!("{}://{}{}/", get_proto(&command_context.proto), domain, ROOT_URI);
    let response = match do_get(url, &command_context).await {
        Ok(v) => {
            if v.status().as_u16() != 200 {
                return Err(String::from("The Docker v2 API is not supported."));
            } 
            else {
                return Ok(());
            }
        },
        Err(e) => return Err(e),
    };
}

pub async fn list_images <'a>(
    domain: String,
    command_context: CommandContext<'a>) -> Result<Vec<String>, String> {

    let url = format!("{}://{}{}{}/", get_proto(&command_context.proto), domain, ROOT_URI, LIST_CONTAINER_URI);
    match do_get(url, &command_context).await {
        Ok(value) => {
            let status_code = value.status().as_u16();
            if status_code != 200 {
                return Err(String::from(map_response_codes_to_error_message(status_code)));
            }
            
            let json = match value.json::<serde_json::Value>().await {
                Ok(value) => value,
                // TODO: Handle this better
                Err(err) => panic!("Failed to deserialize response {}", err),
            };
            // TODO: Handle this better
            
            let lambda = | j: serde_json::Value | -> Option<Vec<String>> {
                let vec = j.as_object()?
                    .get("repositories")?
                    .as_array()?
                    .iter()
                    .map(|f| String::from(f.as_str().unwrap()))
                    .collect();
                return Some(vec);
            };

            let str_arr_res = parse_json(json, &lambda);
            match str_arr_res {
                Some(str_arr) => return Ok(str_arr),
                // TODO: Want something better here
                None => return Err(String::from("Failed to parse response")),
            }
        },
        Err(error) => return Err(error)
    };
}

pub async fn get_image_tags <'a>(
    domain: String,
    command_context: CommandContext<'a>
) -> Result<Vec<String>, String> {

    if command_context.image_name.is_none() {
        return Err(String::from("Must provide the container name"));
    }

    let url = format!("{}://{}{}/{}{}/", get_proto(&command_context.proto), domain, ROOT_URI, command_context.image_name.clone().unwrap(), GET_TAGS);
    let response = match do_get(url, &command_context).await {
        Ok(v) => v,
        Err(e) => return Err(e),
    };

    if response.status().as_u16() != 200 {
        return Err(String::from(map_response_codes_to_error_message(response.status().as_u16())));
    }
    
    let json = match response.json::<serde_json::Value>().await {
        Ok(value) => value,
        // TODO: Handle this better
        Err(err) => panic!("Failed to deserialize response {}", err),
    };

    let lambda = | j: serde_json::Value | -> Option<Vec<String>> {
        let vec = j.as_object()?
         .get("tags")?
         .as_array()?
         .iter()
         .map(|f| String::from(f.as_str().unwrap()))
         .collect();

        return Some(vec);
    };  

    return match parse_json(json, & lambda) {
        Some(v) => Ok(v),
        None => Err(String::from("Failed to parse response")),
    };

}

pub async fn get_image_manifest<'a> (
    domain: String,
    command_context: CommandContext<'a>,
) -> Result<serde_json::Value, String> {
    let url = match get_manifest_uri(&domain, &command_context) {
        Ok(v) => v,
        Err(e) => return Err(e),
    };
    
    let response = match do_get(url, &command_context).await {
        Ok(v) =>  v,
        Err(err) => return Err(err),
    };

    if response.status().as_u16() != 200 {
        return Err(String::from(map_response_codes_to_error_message(response.status().as_u16())));
    }

    let json = match response.json::<serde_json::Value>().await {
        Ok(v) => v,
        Err(_) => panic!("Failed to deserialize response"),
    };

    return Ok(json);
}

pub async fn delete_image<'a>(
    domain: String,
    command_context: CommandContext<'a>
) -> Result<(), String> {
    let url = match get_manifest_uri(&domain, &command_context) {
        Ok(v) => v,
        Err(e) => return Err(e),
    };
    let auth_context = AuthContext {
        username: command_context.username,
        password: command_context.password,
    };

    match http_client::delete(url, None, Some(auth_context)).await {
        Ok(response) => {
            let status_code = response.status().as_u16();
            if status_code == 200 || status_code == 202 || status_code == 204 {
                return Ok(());
            }
            else {
                return Err(map_response_codes_to_error_message(status_code));
            }
        },
        Err(err) => return Err(map_response_codes_to_error_message(err.status_code())),
    }
}

pub async fn get_image_digest<'a>(domain: String, command_context: CommandContext<'a>) -> Result<String, String> {
    let response = get_image_manifest(domain, command_context).await;

    let json = match response {
        Ok(j) => j,
        Err(e) => return Err(e),
    };

    let lambda = | j: serde_json::Value | -> Option<String> {
        let s = String::from(
            j.as_object()?
                .get("config")?
                .as_object()?
                .get("digest")?
                .as_str()?
            );
        return Some(s);
    };

    match parse_json(json, lambda) {
        Some(v) => return Ok(v),
        None => return Err(String::from("Failed to parse response object")),
    };
}

async fn do_get<'a>(url: String, command_context: &'a CommandContext<'a>) -> Result<reqwest::Response, String> {
    let auth_context = AuthContext {
        username: command_context.username,
        password: command_context.password,
    };
    let response = get( url, None, Some(auth_context) ).await;

    match response {
        Ok(value) => {
            let status_code = value.status().as_u16();
            if status_code == 401 || status_code == 402 {
                return Err(String::from("Invalid username and password"));
            }
            else {
                return Ok(value);
            }
        }
        Err(error) => {
            return Err(map_response_codes_to_error_message(error.status_code()));
        }
    }
}

fn get_manifest_uri<'a>(domain: &'a String, command_context: &'a CommandContext<'a>) -> Result<String, String> {
    let container_name = match command_context.image_name.clone() {
        Some(v) => v,
        None => return Err(String::from("Must provide an image name.")),
    };

    let digest = command_context.digest.clone();
    let tag = command_context.tag.clone();
    
    if (digest.is_none() && tag.is_none()) || (tag.is_some() && digest.is_some()) {
        return Err(String::from("Must either specify a tag or a digest."));
    }
    let reference = digest.unwrap_or(tag.unwrap());
    let url = format!("{}://{}{}/{}{}{}/", get_proto(&command_context.proto), domain, ROOT_URI, container_name, MANIFEST, reference);
    return Ok(url);
}

fn map_response_codes_to_error_message(status_code: u16) -> String {
    return match reqwest::StatusCode::from_u16(status_code) {
        Ok(value) => String::from(value.canonical_reason().unwrap()),
        Err(_) => String::from("Unknown status code received"),
    }
}

fn parse_json<T, F>(json: serde_json::Value, mut func: F) -> Option<T> 
    where F: FnMut(serde_json::Value) -> Option<T> {
    return func(json)
}

fn get_proto(proto_opt: &Option<String>) -> String {
    match proto_opt {
        Some(v) => return String::from(v),
        None => return String::from("https"),
    }
}