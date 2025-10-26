use clap::Parser;
use std::{env, fs};
use uuid::Uuid;

/// Configuration for the agentc client
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct ClientConfig {
    /// Unique ID for this client instance.
    #[arg(short, long, default_value_t = default_client_id())]
    pub client_id: String,

    /// Address of the agents server.
    #[arg(short, long, default_value = "127.0.0.1")]
    pub server_addr: String,

    /// Port for the agents control connection.
    #[arg(long, default_value_t = 17001)]
    pub control_port: u16,

    /// Port for the agents proxy connection.
    #[arg(long, default_value_t = 17002)]
    pub proxy_port: u16,

    /// Address of the local service to expose.
    #[arg(long, default_value = "127.0.0.1")]
    pub local_addr: String,

    /// Port of the local service to expose.
    #[arg(long, default_value_t = 3000)]
    pub local_port: u16,

    /// Enable command mode (execute a command instead of TCP proxy)
    #[arg(long)]
    pub command_mode: bool,

    /// Command to execute in command mode
    #[arg(long)]
    pub command_path: Option<String>,

    /// Command arguments (comma-separated)
    #[arg(long)]
    pub command_args: Option<String>,

    /// Enable MCP (Model Context Protocol) server
    #[arg(long)]
    pub enable_mcp: bool,

    /// Port for the MCP server
    #[arg(long, default_value_t = 9021)]
    pub mcp_port: u16,
}

fn default_client_id() -> String {
    ClientConfig::generate_machine_code()
}

impl ClientConfig {
    /// Get the server control address
    pub fn control_addr(&self) -> String {
        format!("{}:{}", self.server_addr, self.control_port)
    }

    /// Get the server proxy address
    pub fn proxy_addr(&self) -> String {
        format!("{}:{}", self.server_addr, self.proxy_port)
    }

    /// Get the local service address
    pub fn local_service_addr(&self) -> String {
        format!("{}:{}", self.local_addr, self.local_port)
    }

    /// Ensure a valid client_id is present, generating one if needed.
    pub fn ensure_client_id(&mut self) -> bool {
        if self.client_id.trim().is_empty() {
            self.client_id = Self::generate_machine_code();
            true
        } else {
            false
        }
    }

    fn generate_machine_code() -> String {
        let entropy = Self::collect_device_entropy();

        let mut raw = if entropy.is_empty() {
            Uuid::new_v4().to_string()
        } else {
            Uuid::new_v5(&Uuid::NAMESPACE_OID, entropy.as_bytes()).to_string()
        };

        raw.retain(|c| c != '-');
        raw.make_ascii_uppercase();
        raw
    }

    fn collect_device_entropy() -> String {
        let mut parts = Vec::new();

        if let Some(host) = Self::hostname() {
            parts.push(format!("host:{host}"));
        }

        if let Some(machine_id) = Self::machine_id() {
            parts.push(format!("machine_id:{machine_id}"));
        }

        if let Ok(user) = env::var("USER").or_else(|_| env::var("USERNAME")) {
            if !user.is_empty() {
                parts.push(format!("user:{user}"));
            }
        }

        parts.push(format!("os:{}", env::consts::OS));
        parts.push(format!("arch:{}", env::consts::ARCH));

        #[cfg(target_os = "windows")]
        {
            if let Ok(name) = env::var("COMPUTERNAME") {
                if !name.is_empty() {
                    parts.push(format!("computer:{name}"));
                }
            }
        }

        #[cfg(target_os = "linux")]
        {
            if let Some(distro) = Self::linux_os_release() {
                parts.push(format!("distro:{distro}"));
            }
        }

        parts.join("|")
    }

    fn hostname() -> Option<String> {
        hostname::get().ok()?.into_string().ok()
    }

    fn machine_id() -> Option<String> {
        #[cfg(target_os = "linux")]
        {
            const PATHS: [&str; 2] = ["/etc/machine-id", "/var/lib/dbus/machine-id"];
            for path in PATHS {
                if let Some(value) = Self::read_trimmed(path) {
                    return Some(value);
                }
            }
        }

        #[cfg(target_os = "macos")]
        {
            if let Some(value) = Self::read_trimmed("/etc/hostid") {
                return Some(value);
            }
        }

        None
    }

    #[cfg(target_os = "linux")]
    fn linux_os_release() -> Option<String> {
        use std::io::Read;

        let mut file = fs::File::open("/etc/os-release").ok()?;
        let mut content = String::new();
        file.read_to_string(&mut content).ok()?;

        for line in content.lines() {
            if let Some(value) = line.strip_prefix("PRETTY_NAME=") {
                return Some(value.trim_matches('"').to_string());
            }
        }

        None
    }

    #[cfg(not(target_os = "linux"))]
    fn linux_os_release() -> Option<String> {
        None
    }

    fn read_trimmed(path: &str) -> Option<String> {
        let content = fs::read_to_string(path).ok()?;
        let trimmed = content.trim();
        if trimmed.is_empty() {
            None
        } else {
            Some(trimmed.to_string())
        }
    }
}
