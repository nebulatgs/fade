pub trait RestQuery {
    type RequestData: serde::ser::Serialize;
    type ResponseData: for<'de> serde::Deserialize<'de>;
}

pub mod launch_machine;
pub mod list_machines;
pub mod stop_machine;
