use std::net::Ipv6Addr;

use anyhow::{Context, Result};
use ipnet::Ipv6Net;
use network_interface::{Addr, NetworkInterface, NetworkInterfaceConfig};

pub fn get_tailscale_ipv6() -> Result<Ipv6Addr> {
    let network_interfaces = NetworkInterface::show()?;
    let mask: Ipv6Net = "fd7a:115c:a1e0:ab12::/64".parse()?;
    let filtered = network_interfaces
        .iter()
        .filter_map(|i| match i.addr {
            Some(Addr::V6(v6)) => Some(v6.ip),
            _ => None,
        })
        .filter(|i| mask.contains(i))
        .next();
    filtered.context("No Tailscale interface found")
}
