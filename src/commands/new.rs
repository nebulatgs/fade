use std::collections::HashMap;

use anyhow::Context;
use indicatif::ProgressBar;
use tokio::{net::TcpListener, process::Command};

use crate::{
    client::{post_graphql, GQLClient},
    config::Configs,
    gql::{
        machine_config::{Guest, Init, MachineConfig},
        mutations::{
            create_app, launch_machine, remove_machine, CreateApp, LaunchMachine, RemoveMachine,
        },
        queries::{get_app_meta, get_organization_meta, GetAppMeta, GetOrganizationMeta},
    },
    interface::get_tailscale_ipv6,
    Kind,
};

use super::*;

pub async fn command(kind: Kind, memory: u16, region: Option<String>) -> Result<()> {
    let config = Configs::new().await?;
    let client = GQLClient::new_authorized(&config)?;
    let tailscale_addr = get_tailscale_ipv6()?;

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
        let data = res.data.context("Failed to retrieve response body")?;
    }
    let listener = TcpListener::bind((tailscale_addr, 0)).await?;

    let res = post_graphql::<LaunchMachine, _>(
        &client,
        "https://api.fly.io/graphql",
        launch_machine::Variables {
            input: launch_machine::LaunchMachineInput {
                app_id: Some(config.fly_app_name()),
                client_mutation_id: None,
                organization_id: Some(organization.id),
                id: None,
                name: None,
                region,
                config: MachineConfig {
                    env: Some(HashMap::from_iter([
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
                    ])),
                    init: Init {
                        cmd: None,
                        entrypoint: None,
                        exec: None,
                        tty: Some(false),
                    },
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
                        }
                    ),
                    metadata: None,
                    restart: None,
                    guest: Guest {
                        cpu_kind: "shared".to_string(),
                        cpus: 8,
                        memory_mb: memory.try_into()?,
                    },
                },
            },
        },
    )
    .await?;
    let data = res.data.context("Failed to retrieve response body")?;
    let machine = data
        .launch_machine
        .context("Failed to launch machine")?
        .machine;
    let spinner = ProgressBar::new_spinner().with_message("Launching machine");
    spinner.enable_steady_tick(50);

    let ip = listener.accept().await.map(|(_, addr)| addr.ip());

    // {
    //     use std::net::ToSocketAddrs;
    //     while let Err(_) = format!("{}:22", machine.id).to_socket_addrs() {}
    // }

    spinner.finish_with_message("\x1b[2J\x1b[1;1H");

    let mut ssh_process = Command::new("tailscale")
        .arg("ssh")
        .arg(format!("fade@{}", ip?))
        .arg("-o")
        .arg("BatchMode=yes")
        .arg("-o")
        .arg("StrictHostKeyChecking=no")
        .spawn()?;
    ssh_process.wait().await?;

    println!("Cleaning up machine...");

    let res = post_graphql::<RemoveMachine, _>(
        &client,
        "https://api.fly.io/graphql",
        remove_machine::Variables {
            input: remove_machine::RemoveMachineInput {
                client_mutation_id: None,
                id: machine.id,
                app_id: Some(config.fly_app_name()),
                kill: Some(true),
            },
        },
    )
    .await?;
    res.data.context("Failed to retrieve response body")?;

    Ok(())
}
