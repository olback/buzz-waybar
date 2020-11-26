use {
    dirs,
    serde::Deserialize,
    std::{collections::HashMap, fs, io},
    toml,
};

#[derive(Debug, Deserialize)]
struct ServerToml {
    address: String,
    port: u16,
    username: String,
    #[serde(rename = "password-cmd")]
    password_cmd: String,
    folder: Option<String>,
    #[serde(rename = "notification-cmd")]
    notification_cmd: Option<String>,
    #[serde(rename = "trim-password")]
    trim_password: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct DefaultsToml {
    #[serde(rename = "notification-cmd")]
    notification_cmd: Option<String>,
    #[serde(rename = "folder")]
    folder: Option<String>,
    #[serde(rename = "trim-password")]
    trim_password: Option<bool>,
}

#[derive(Clone, Debug)]
pub struct Server {
    pub address: String,
    pub port: u16,
    pub username: String,
    pub password_cmd: String,
    pub folder: String,
    pub notification_cmd: Option<String>,
    pub trim_password: bool,
}

pub fn load_servers() -> Result<HashMap<String, Server>, io::Error> {
    let path = dirs::config_dir()
        .expect("Failed to get default config dir")
        .join("waybar")
        .join("Buzz.toml");

    let toml_str = fs::read_to_string(&path)?;
    let toml_parsed: HashMap<String, toml::Value> = toml::from_str(&toml_str)?;
    let mut toml_servers: HashMap<String, ServerToml> = HashMap::with_capacity(toml_parsed.len());

    let mut defaults = DefaultsToml {
        notification_cmd: None,
        folder: None,
        trim_password: Some(true),
    };

    for (key, value) in &toml_parsed {
        if key == "defaults" {
            defaults = value
                .clone()
                .try_into::<DefaultsToml>()
                .expect("Defaults malformed");
        } else {
            if let Ok(server) = value.clone().try_into::<ServerToml>() {
                toml_servers.insert(key.clone(), server);
            }
        }
    }

    let mut servers = HashMap::<String, Server>::with_capacity(toml_servers.len());
    for (key, toml_server) in toml_servers {
        servers.insert(
            key.clone(),
            Server {
                address: toml_server.address.clone(),
                port: toml_server.port,
                username: toml_server.username.clone(),
                password_cmd: toml_server.password_cmd.clone(),
                folder: toml_server
                    .folder
                    .or(defaults.folder.clone())
                    .unwrap_or("INBOX".to_string()),
                notification_cmd: toml_server
                    .notification_cmd
                    .or(defaults.notification_cmd.clone()),
                trim_password: toml_server
                    .trim_password
                    .or(defaults.trim_password)
                    .unwrap_or(true),
            },
        );
    }

    Ok(servers)
}
