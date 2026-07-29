use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInfo {
    pub name: String,
    pub exe_name: String,
    pub exe_path: String,
    pub icon_path: Option<String>,
    pub category: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AetherConfig {
    pub protocol: String,
    pub scan_mode: String,
    pub ip_version: String,
    pub quick_reconnect: bool,
}

impl Default for AetherConfig {
    fn default() -> Self {
        Self {
            protocol: "auto".into(),
            scan_mode: "balanced".into(),
            ip_version: "v4".into(),
            quick_reconnect: true,
        }
    }
}

impl AetherConfig {
    pub fn to_cli_args(&self) -> Vec<String> {
        let mut args = vec!["--bind".into(), "127.0.0.1:1819".into()];

        match self.protocol.as_str() {
            "masque" => args.push("--masque".into()),
            "wireguard" => args.push("--wg".into()),
            "gool" => args.push("--gool".into()),
            _ => {}
        }

        args.push("--scan".into());
        args.push(self.scan_mode.clone());

        match self.ip_version.as_str() {
            "v4" => args.push("-4".into()),
            "v6" => args.push("-6".into()),
            "both" => args.push("--dual".into()),
            _ => args.push("-4".into()),
        }

        if self.quick_reconnect {
            args.push("--quick-reconnect".into());
        } else {
            args.push("--no-quick-reconnect".into());
        }

        args
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionStatus {
    pub state: String,
    pub message: String,
    pub socks_port: Option<u16>,
    pub connected_at: Option<u64>,
    pub protocol: Option<String>,
    pub scan_mode: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct V2RayConfig {
    pub config: String,
    pub auto_tune_mtu: bool,
    pub enable_dns_optimization: bool,
    pub protocol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WireGuardConfig {
    pub config: String,
    pub config_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpenVpnConfig {
    pub config_path: String,
    pub username: String,
    pub password: String,
    pub private_key_password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SocksConfig {
    pub protocol: String,
    pub server: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum ProviderConfig {
    Aether(AetherConfig),
    V2Ray(V2RayConfig),
    WireGuard(WireGuardConfig),
    OpenVpn(OpenVpnConfig),
    Socks(SocksConfig),
}
