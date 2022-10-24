use std::{borrow::Cow, process::Stdio, sync::Arc, time::Duration};

use anyhow::{Context, Result};
use tokio::{
    io::{AsyncBufReadExt, BufReader},
    process::Command,
};

#[tokio::main]
async fn main() -> Result<()> {
    if std::env::var("TAILSCALE_ADDR")
        .unwrap_or_default()
        .is_empty()
    {
        eprintln!("Running without tailscale");
        let mut sshd = Command::new("/usr/sbin/sshd")
            .arg("-D")
            .arg("-p")
            .arg("23")
            .arg("-o")
            .arg("PermitEmptyPasswords=yes")
            .spawn()?;
        sshd.wait().await?;
        return Ok(());
    }

    let tailscale_authkey = std::env::var("TAILSCALE_AUTHKEY")?;
    let tailscale_addr = std::env::var("TAILSCALE_ADDR")?;

    let connected = Arc::new(tokio::sync::Notify::new());
    let task_notify = connected.clone();

    let mut tailscaled = Command::new("tailscaled")
        .arg("-verbose")
        .arg("10")
        .stderr(Stdio::piped())
        .spawn()?;
    let tailscaled_stderr = tailscaled.stderr.take().context("Failed to take stderr")?;
    let mut tailscaled_stderr_reader = BufReader::new(tailscaled_stderr).lines();

    // Intercept tailscaled's stderr and forward to our stderr.
    tokio::spawn(async move {
        while let Ok(Some(line)) = tailscaled_stderr_reader.next_line().await {
            eprintln!("{}", &line);
            if line.contains(r#"[RATELIMIT] format("control: [v\x00JSON]%d%s")"#) {
                task_notify.notify_one();
                task_notify.notify_waiters();
            }
        }
    });

    let mut tailscale_up = Command::new("tailscale")
        .arg("up")
        .arg("--ssh")
        .arg("--auth-key")
        .arg(tailscale_authkey)
        .spawn()?;
    tailscale_up.wait().await?;
    println!("[fade] tailscale is up");

    tokio::select! {
        _ = connected.notified() => {println!("[fade] tailscale is connected");}
        _ = tokio::time::sleep(Duration::from_secs(5)) => {eprintln!("[fade] (ASSUMING) tailscale is connected");}
    }

    let (sender, mut receiver) = tokio::sync::broadcast::channel::<()>(1);

    let spawner_sender = sender.clone();
    let spawner = tokio::spawn(async move {
        let addr = Cow::from(tailscale_addr);
        let mut spawner_receiver = spawner_sender.subscribe();
        loop {
            let addr = addr.clone();
            let mut receiver = spawner_sender.subscribe();
            let task_sender = sender.clone();
            let receiver_task = spawner_receiver.recv();
            tokio::spawn(async move {
                tokio::select! {
                    Ok(_) = tokio::time::timeout(Duration::from_millis(500), tokio::net::TcpStream::connect(addr.as_ref())) => {task_sender.send(()).unwrap();}
                    _ = receiver.recv() => {}
                };
            });
            tokio::select! {
                _ = tokio::time::sleep(Duration::from_millis(100)) => {}
                _ = receiver_task => {break;}
            };
        }
    });

    tokio::select! {
        _ = receiver.recv() => {}
        _ = spawner => {}
    }
    println!("[fade] signalled client");
    tailscaled.wait().await?;
    Ok(())
}
