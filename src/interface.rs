use bytes_size::ByteSize;
use ipnet::IpNet;
use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Interface {
    pub name: Option<String>,
    pub public_key: Option<String>,
    pub private_key: Option<String>,
    pub listening_port: Option<u16>,
    pub address: Option<IpNet>,
    pub dns: Option<IpAddr>,
    pub peers: Vec<Peer>,
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Peer {
    pub public_key: Option<String>,
    pub preshared_key: Option<String>,
    pub latest_handshake: Option<TimeDuration>,
    pub endpoint: AddrPort,
    pub allowed_ips: AllowedIPs,
    pub transfer: Option<Transfer>,
    pub persistent_keepalive: Option<TimeDuration>,
}
#[derive(Debug, Clone)]
pub struct AllowedIPs(pub Vec<IpNet>);

#[derive(Debug, Clone)]
pub struct Transfer {
    pub received: ByteSize,
    pub sent: ByteSize,
}
#[derive(Debug, Clone)]
/// 解析 例如：1day, 2 minute, 3 seconds
/// 单数字时默认解析为秒： 1 => 1s
pub struct TimeDuration(pub Duration);

#[derive(Debug, Clone)]
pub struct AddrPort {
    pub addr: IpAddr,
    pub port: u64,
}
impl Interface {
    pub fn new() -> Self {
        Self {
            name: None,
            public_key: None,
            private_key: None,
            listening_port: None,
            address: None,
            dns: None,
            peers: Default::default(),
        }
    }
    pub fn is_none(&self) -> bool {
        self.name.is_none()
    }
}

impl Peer {
    pub fn new() -> Self {
        Self {
            public_key: None,
            preshared_key: None,
            latest_handshake: Default::default(),
            endpoint: AddrPort::new(),
            allowed_ips: Default::default(),
            transfer: None,
            persistent_keepalive: None,
        }
    }
    pub fn is_none(&self) -> bool {
        self.public_key.is_none()
    }
}

impl AddrPort {
    pub fn new() -> Self {
        Self {
            addr: IpAddr::V4(Ipv4Addr::from(0)),
            port: 0,
        }
    }
}

impl Default for AllowedIPs {
    fn default() -> Self {
        Self(Vec::new())
    }
}
impl Default for TimeDuration {
    fn default() -> Self {
        Self(Duration::default())
    }
}
impl Default for Transfer {
    fn default() -> Self {
        Self {
            received: Default::default(),
            sent: Default::default(),
        }
    }
}

impl TimeDuration {
    pub fn to_duration(&self) -> Duration {
        self.0
    }
}
