use std::{process::Stdio, time::Duration};

use anyhow::Context;
use indicatif::ProgressBar;
use tokio::{net::TcpStream, process::Command};

use crate::{
    client::{delete_rest, get_rest, post_graphql, AuthorizedClient},
    config::Configs,
    gql::queries::{get_organization_meta, GetOrganizationMeta},
    rest::list_machines::ListMachines,
};

use super::*;

pub async fn command() -> Result<()> {
    let config = Configs::new().await?;
    let client = AuthorizedClient::new(&config)?;

    let res = post_graphql::<GetOrganizationMeta, _>(
        &client,
        "https://api.fly.io/graphql",
        get_organization_meta::Variables {
            id: None,
            name: None,
            slug: Some(config.root_config.fly_organization.clone()),
        },
    )
    .await?;
    let data = res.data.context("Failed to retrieve response body")?;
    let organization = data
        .organization
        .context("Failed to retrieve organization")?;

    let _proxy_command = if tokio::time::timeout(
        Duration::from_millis(200),
        TcpStream::connect(FLY_API_HOSTNAME),
    )
    .await
    .ok()
    .transpose()
    .ok()
    .flatten()
    .is_none()
    {
        let spinner = ProgressBar::new_spinner().with_message("Starting fly api proxy");
        spinner.enable_steady_tick(50);

        let mut proxy_command = Command::new("flyctl")
            .arg("machines")
            .arg("api-proxy")
            .arg("-o")
            .stdout(Stdio::piped())
            .arg(organization.slug)
            .kill_on_drop(true)
            .spawn()?;
        let stdout = proxy_command
            .stdout
            .take()
            .context("Failed to take stdout")?;

        tokio::spawn(async move {
            use tokio::io::{AsyncBufReadExt, BufReader};
            let mut lines = BufReader::new(stdout).lines();
            while let Ok(Some(line)) = lines.next_line().await {
                if line.contains("Proxying local port") {
                    break;
                }
            }
        })
        .await?;
        spinner.finish_with_message("Started fly api proxy");
        Some(proxy_command)
    } else {
        None
    };

    let machines = get_rest::<ListMachines, _>(
        &client,
        format!(
            "http://{FLY_API_HOSTNAME}/v1/apps/{}/machines",
            config.fly_app_name(),
        ),
    )
    .await?;

    let stopped = machines.iter().filter(|m| m.state == "stopped");

    for machine in stopped {
        delete_rest::<_>(
            &client,
            format!(
                "http://{FLY_API_HOSTNAME}/v1/apps/{}/machines/{}",
                config.fly_app_name(),
                machine.id,
            ),
        )
        .await?;
        println!("Deleted machine {}", machine.id);
    }
    Ok(())
}
