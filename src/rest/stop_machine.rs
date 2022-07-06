use super::RestQuery;
use serde::{Deserialize, Serialize};

pub struct StopMachine;

impl RestQuery for StopMachine {
    type RequestData = ();
    type ResponseData = StopMachineResponse;
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StopMachineResponse {
    pub ok: bool,
}
