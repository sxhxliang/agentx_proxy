use anyhow::{anyhow, Result};
use std::net::SocketAddr;
use tokio::net::lookup_host;
use trust_dns_resolver::config::*;
use trust_dns_resolver::TokioAsyncResolver;

pub struct ServerEndpoints {
    pub control: SocketAddr,
    pub proxy: SocketAddr,
}

/// Resolve server endpoints from domain using DNS SRV records
pub async fn resolve_from_srv(domain: &str) -> Result<ServerEndpoints> {
    let resolver = TokioAsyncResolver::tokio(ResolverConfig::default(), ResolverOpts::default());

    // Query SRV records
    let control_srv = format!("_control._tcp.{}", domain);
    let proxy_srv = format!("_proxy._tcp.{}", domain);

    let control_lookup = resolver
        .srv_lookup(&control_srv)
        .await
        .map_err(|e| anyhow!("Failed to lookup {}: {}", control_srv, e))?;

    let proxy_lookup = resolver
        .srv_lookup(&proxy_srv)
        .await
        .map_err(|e| anyhow!("Failed to lookup {}: {}", proxy_srv, e))?;

    // Get first SRV record
    let control_record = control_lookup
        .iter()
        .next()
        .ok_or_else(|| anyhow!("No SRV record found for {}", control_srv))?;

    let proxy_record = proxy_lookup
        .iter()
        .next()
        .ok_or_else(|| anyhow!("No SRV record found for {}", proxy_srv))?;

    // Resolve target hosts
    let control = resolve_host(control_record.target().to_utf8().as_str(), control_record.port()).await?;
    let proxy = resolve_host(proxy_record.target().to_utf8().as_str(), proxy_record.port()).await?;

    Ok(ServerEndpoints { control, proxy })
}

/// Resolve server endpoints from domain using subdomain convention
pub async fn resolve_from_subdomain(domain: &str, default_control_port: u16, default_proxy_port: u16) -> Result<ServerEndpoints> {
    let control_host = format!("control.{}", domain);
    let proxy_host = format!("proxy.{}", domain);

    let control = resolve_host(&control_host, default_control_port).await?;
    let proxy = resolve_host(&proxy_host, default_proxy_port).await?;

    Ok(ServerEndpoints { control, proxy })
}

async fn resolve_host(host: &str, port: u16) -> Result<SocketAddr> {
    let addr_str = format!("{}:{}", host, port);
    let mut addrs = lookup_host(&addr_str)
        .await
        .map_err(|e| anyhow!("Failed to resolve {}: {}", addr_str, e))?;

    addrs
        .next()
        .ok_or_else(|| anyhow!("No address found for {}", addr_str))
}
