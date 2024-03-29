use std::{collections::HashMap, process::Stdio, time::Duration};

use anyhow::Context;
use indicatif::ProgressBar;
use rand::Rng;
use tokio::{
    net::{TcpListener, TcpStream},
    process::Command,
};

use crate::{
    client::{post_graphql, post_rest, AuthorizedClient},
    config::Configs,
    gql::{
        machine_config::{Guest, Init, MachineConfig, Port, Service},
        mutations::{create_app, CreateApp},
        queries::{get_app_meta, get_organization_meta, GetAppMeta, GetOrganizationMeta},
    },
    interface::get_tailscale_ipv6,
    rest::{
        launch_machine::{self, LaunchMachine},
        stop_machine::StopMachine,
    },
    Kind,
};

use super::*;

pub fn parse_port(port: &str) -> Result<(u16, u16)> {
    let mut parts = port.split(':');
    let internal = parts.next().context("Invalid port")?.parse::<u16>()?;
    let external = parts.next().map_or(Ok(internal), |p| p.parse::<u16>())?;
    Ok((internal, external))
}

pub async fn command(
    kind: Kind,
    memory: u16,
    region: Option<String>,
    ports: Vec<String>,
    tailscale: bool,
) -> Result<()> {
    let config = Configs::new().await?;
    let client = AuthorizedClient::new(&config)?;

    let tailscale_addr = if tailscale {
        Some(get_tailscale_ipv6()?)
    } else {
        None
    };

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
            .arg(&organization.slug)
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

    let res = post_graphql::<GetAppMeta, _>(
        &client,
        "https://api.fly.io/graphql",
        get_app_meta::Variables {
            name: Some(config.fly_app_name()),
            internal_id: None,
        },
    )
    .await?;
    let data = res.data.context("Failed to retrieve response body")?;

    if data.app.is_none() {
        let res = post_graphql::<CreateApp, _>(
            &client,
            "https://api.fly.io/graphql",
            create_app::Variables {
                input: create_app::CreateAppInput {
                    client_mutation_id: None,
                    organization_id: organization.id.clone(),
                    runtime: None,
                    name: Some(config.fly_app_name()),
                    preferred_region: None,
                    heroku: None,
                    network: None,
                    app_role_id: None,
                },
            },
        )
        .await?;
        res.data.context("Failed to retrieve response body")?;
    }

    let listener = if let Some(tailscale_addr) = tailscale_addr {
        Some(TcpListener::bind((tailscale_addr, 0)).await?)
    } else {
        None
    };

    let services = ports
        .iter()
        .map(|port| parse_port(port))
        .map(|res| {
            res.map(|(internal, external)| Service {
                internal_port: internal,
                protocol: "tcp".to_string(),
                ports: vec![Port { port: external }],
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let res = post_rest::<LaunchMachine, _>(
        &client,
        format!(
            "http://{FLY_API_HOSTNAME}/v1/apps/{}/machines",
            config.fly_app_name()
        ),
        launch_machine::LaunchMachineRequest {
            name: None,
            region,
            config: MachineConfig {
                env: if let Some(ref listener) = listener {
                    Some(HashMap::from_iter([
                        (
                            "TAILSCALE_ADDR".to_string(),
                            listener.local_addr()?.to_string(),
                        ),
                        (
                            "TAILSCALE_AUTHKEY".to_string(),
                            config
                                .root_config
                                .tailscale_authkey
                                .clone()
                                .context("Missing Tailscale authkey")?,
                        ),
                    ]))
                } else {
                    None
                },
                init: Init {
                    cmd: None,
                    entrypoint: None,
                    exec: None,
                    tty: Some(false),
                },
                services: Some(services),
                image: format!(
                    "nebulatgs/fade-stamp{}:{}",
                    if std::env::var("CARGO").is_err() {
                        ""
                    } else {
                        "-dev"
                    },
                    match kind {
                        Kind::Min => "minimal",
                        Kind::Docker => "minimal-docker",
                        Kind::Full => "full",
                    },
                ),
                metadata: None,
                restart: None,
                guest: Some(Guest {
                    cpu_kind: "shared".to_string(),
                    cpus: 8,
                    memory_mb: memory.try_into()?,
                }),
            },
        },
    )
    .await?;
    let spinner = ProgressBar::new_spinner().with_message("Launching machine");
    spinner.enable_steady_tick(50);

    let port = rand::thread_rng().gen_range(2000..3000);

    let ip = if let Some(listener) = listener {
        Some(listener.accept().await.map(|(_, addr)| addr.ip()))
    } else {
        Command::new("flyctl")
            .arg("proxy")
            .arg(format!("{port}:23"))
            .arg("-q")
            .arg("-o")
            .arg(&organization.slug)
            .arg(format!("{}", &res.private_ip))
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()?;
        None
    };

    spinner.finish_with_message("\x1b[2J\x1b[1;1H");

    if let Some(ip) = ip {
        let mut ssh_process = Command::new("tailscale")
            .arg("ssh")
            .arg(format!("fade@{}", ip?))
            .arg("-o")
            .arg("BatchMode=yes")
            .arg("-o")
            .arg("StrictHostKeyChecking=no")
            .spawn()?;
        ssh_process.wait().await?;
    } else {
        let mut ssh_process = Command::new("ssh")
            .arg("-o")
            .arg("BatchMode=yes")
            .arg("-o")
            .arg("StrictHostKeyChecking=no")
            .arg("-o")
            .arg("UserKnownHostsFile=/dev/null")
            .arg("fade@127.0.0.1")
            .arg("-p")
            .arg(port.to_string())
            .spawn()?;
        ssh_process.wait().await?;
    }

    post_rest::<StopMachine, _>(
        &client,
        format!(
            "http://{FLY_API_HOSTNAME}/v1/apps/{}/machines/{}/stop",
            config.fly_app_name(),
            res.id
        ),
        (),
    )
    .await?;
    Ok(())
}
