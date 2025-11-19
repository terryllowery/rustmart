use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct DatabaseConfig{
    pub url: String,
    pub max_connections: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ServerConfig{
    pub host: String,
    pub port: u16,
}