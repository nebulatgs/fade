use colored::Colorize;

use crate::config::Configs;

use super::*;

pub async fn command() -> Result<()> {
    let mut config = Configs::new().await?;
    let fly_organization = dialoguer::Input::new()
        .with_prompt("Fly organization slug")
        .default(config.root_config.fly_organization.clone())
        .interact_text()?;

    println!();
    println!(
        "Provide a {} and {} key for Tailscale",
        "reusable".bold(),
        "ephemeral".bold()
    );
    let tailscale_authkey = dialoguer::Input::new()
        .with_prompt("Tailscale authkey")
        .default(
            if config.root_config.tailscale_authkey.is_some() {
                "HIDDEN"
            } else {
                ""
            }
            .to_string(),
        )
        .interact_text()?;

    println!();
    let fly_pat = dialoguer::Input::new()
        .with_prompt("Fly personal access token")
        .default(
            if config.root_config.fly_pat.is_some() {
                "HIDDEN"
            } else {
                ""
            }
            .to_string(),
        )
        .interact_text()?;
    config.root_config.fly_organization = fly_organization;
    if tailscale_authkey != "HIDDEN" {
        config.root_config.tailscale_authkey = Some(tailscale_authkey);
    }
    if fly_pat != "HIDDEN" {
        config.root_config.fly_pat = Some(fly_pat);
    }
    config.write().await?;
    Ok(())
}
