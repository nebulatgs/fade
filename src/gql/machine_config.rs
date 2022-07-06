use std::collections::HashMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct MachineConfig {
    pub env: Option<HashMap<String, String>>,
    pub init: Init,
    pub image: String,
    pub services: Option<Vec<Service>>,
    pub metadata: Option<HashMap<String, String>>,
    pub restart: Option<Restart>,
    pub guest: Option<Guest>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Service {
    pub internal_port: u16,
    pub ports: Vec<Port>,
    pub protocol: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Port {
    pub port: u16,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Guest {
    pub cpu_kind: String,
    pub cpus: i64,
    pub memory_mb: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Init {
    pub exec: Option<String>,
    pub entrypoint: Option<String>,
    pub cmd: Option<String>,
    pub tty: Option<bool>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Restart {
    pub policy: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageRef {
    pub registry: String,
    pub repository: String,
    pub tag: String,
    pub digest: String,
    pub labels: HashMap<String, String>,
}
