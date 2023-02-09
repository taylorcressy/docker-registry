use std::io::Write;

use atty::Stream;
use rpassword::read_password;

use crate::conf::AppConf;

pub struct DerivedCredentials {
    pub username: String, 
    pub password: String,
}

/**
 * Derive the username and password to utilize while using docker-registry. 
 * The auth chain will prioritize credentials that are passed in via cli args. 
 */
pub fn derive_credentials_through_chain(
    username_provided: Option<String>,
    password_provided: Option<String>,
    app_conf: AppConf,
) -> Result<DerivedCredentials, String> {
    
    if username_provided.is_some() && password_provided.is_some() {
        return Ok(DerivedCredentials {
            username: username_provided.unwrap(),
            password: password_provided.unwrap(),
        });
    }


    let username: String = if username_provided.is_some() {
        let u = username_provided.unwrap();
        u
    } 
    else if app_conf.username.is_some() {
        let u = app_conf.username.unwrap();
        u
    }
    else {
        return Err(String::from("Could not find username in credential chain."));
    };
    

    let password: String = if password_provided.is_some() {
        let u = password_provided.unwrap();
        u
    } 
    else if app_conf.password.is_some() {
        let u = app_conf.password.unwrap();
        u
    }
    else if atty::is(Stream::Stdin) {
        print!("Enter Password: ");
        std::io::stdout().flush().unwrap();
        let u = read_password().unwrap();
        u
    }
    else {
        return Err(String::from("Could not find password in credential chain."));
    };

    return Ok(DerivedCredentials { username: username, password: password })
}