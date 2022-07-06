use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::gql::machine_config::MachineConfig;

use super::RestQuery;

pub struct LaunchMachine;

impl RestQuery for LaunchMachine {
    type RequestData = LaunchMachineRequest;
    type ResponseData = LaunchMachineResponse;
}

#[derive(Debug, Serialize)]
pub struct LaunchMachineRequest {
    pub name: Option<String>,
    pub region: Option<String>,
    pub config: MachineConfig,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LaunchMachineResponse {
    pub id: String,
    pub name: String,
    pub state: String,
    pub region: String,
    pub instance_id: String,
    pub private_ip: String,
    pub config: MachineConfig,
    pub image_ref: ImageRef,
    pub created_at: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImageRef {
    pub registry: String,
    pub repository: String,
    pub tag: String,
    pub digest: String,
    pub labels: HashMap<String, String>,
}
