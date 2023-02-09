use dirs::home_dir;


const DEFAULT_CONF: &str = "/.docker-registry/config";
const AUTH_SECTION: &str = "auth";

#[derive(Clone)]
pub struct AppConf {
    pub username: Option<String>,
    pub password: Option<String>,
}

pub fn load_conf(override_path: Option<String>) -> Result<AppConf, String> {
    let path = match dirs::home_dir() {
        Some(path) => {
            override_path.unwrap_or(String::from(format!("{}{}", path.to_str().unwrap(), DEFAULT_CONF)))
        },
        None => {
            if override_path.is_some() {
                let p = override_path.unwrap();
                p
            }
            else {
                return Err(String::from("Unable to detect home directory"))
            }      
        }
    };
    

    let conf = match ini::Ini::load_from_file(path.clone()) {
        Ok(v) => v,
        Err(error) => {
            if std::env::var("DOCKER_REG_VERBOSE").is_ok() {
                println!("Couldn't load conf file: {} - Checked {}", error.to_string(), path);
            }
            return Ok(AppConf { username: None, password: None });
        },
    };
    
    let app_conf = AppConf {
        username: conf.get_from(Some(AUTH_SECTION), "username").map(|s| String::from(s)),
        password: conf.get_from(Some(AUTH_SECTION), "password").map(|s| String::from(s)),
    };

    return Ok(app_conf);
}