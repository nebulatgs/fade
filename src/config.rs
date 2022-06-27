use std::path::PathBuf;

use anyhow::{Context, Result};
use rand::Rng;
use serde::{Deserialize, Serialize};
use tokio::{
    fs::{create_dir_all, File},
    io::{AsyncReadExt, AsyncWriteExt},
};

#[derive(Serialize, Deserialize, Debug)]
pub struct FadeConfig {
    fly_app_name: String,
    pub tailscale_authkey: Option<String>,
    pub fly_pat: Option<String>,
    pub fly_organization: String,
}

#[derive(Debug)]
pub struct Configs {
    pub root_config: FadeConfig,
    root_config_path: PathBuf,
}

impl Configs {
    pub async fn new() -> Result<Self> {
        let root_config_partial_path = ".fade/config.json";

        let home_dir = dirs::home_dir().context("Unable to get home directory")?;
        let root_config_path = std::path::Path::new(&home_dir).join(root_config_partial_path);

        if let Ok(mut file) = File::open(&root_config_path).await {
            let mut serialized_config = vec![];
            file.read_to_end(&mut serialized_config).await?;

            let root_config: FadeConfig = serde_json::from_slice(&serialized_config)?;
            return Ok(Self {
                root_config,
                root_config_path,
            });
        }
        let random_string = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(8)
            .map(char::from)
            .collect::<String>()
            .to_lowercase();
        Ok(Self {
            root_config_path,
            root_config: FadeConfig {
                fly_app_name: format!("fade-{}", random_string),
                fly_organization: "personal".to_string(),
                tailscale_authkey: None,
                fly_pat: None,
            },
        })
    }

    pub fn fly_app_name(&self) -> String {
        let mut name = self.root_config.fly_organization.clone();
        name.push('-');
        name.push_str(&self.root_config.fly_app_name);
        name
    }

    pub async fn write(&self) -> Result<()> {
        create_dir_all(self.root_config_path.parent().unwrap()).await?;
        let mut file = File::create(&self.root_config_path).await?;
        let serialized_config = serde_json::to_vec_pretty(&self.root_config)?;
        file.write_buf(&mut serialized_config.as_slice()).await?;
        file.sync_all().await?;
        Ok(())
    }
}
