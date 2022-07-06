pub mod cleanup;
pub mod new;
pub mod setup;

pub(super) static FLY_API_HOSTNAME: &str = "localhost:4280";
pub(super) use anyhow::Result;
