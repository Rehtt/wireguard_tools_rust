mod parse;

use std::time::Duration;
use bytes_size::ByteSize;
use crate::parse::{AddrPort, TimeDuration};

#[derive(Clone)]
pub struct Interface {
    name: String,
    public_key: String,
    listening_port: u16,
    peers: Vec<Peer>,
}

#[derive(Clone)]
pub struct Peer {
    public_key: String,
    latest_handshake: TimeDuration,
    endpoint: AddrPort,
    allowed_ips: Vec<ipnet::IpNet>,
    transfer: Transfer,
    persistent_keepalive: TimeDuration,
}

#[derive(Clone)]
pub struct Transfer {
    received: ByteSize,
    sent: ByteSize,
}

impl Interface {
    fn new() -> Self {
        Self {
            name: String::new(),
            public_key: String::new(),
            listening_port: 0,
            peers: Vec::new(),
        }
    }
    fn is_empty(&self) -> bool {
        self.name.is_empty()
    }

    /// 解析wg命令输出结果
    ///
    /// # Example
    /// ```
    /// use wireguard_tools_rust::Interface;
    /// let a = "
    /// interface: wg0
    ///   public key: public_key1
    ///   private key: (hidden)
    ///   listening port: 51820
    ///
    /// peer: public_key2
    ///   preshared key: (hidden)
    ///   endpoint: 1.1.1.1:1
    ///   allowed ips: 1.1.2.0/24
    ///   latest handshake: 1 minute, 31 seconds ago
    ///   transfer: 1.18 MiB received, 3.89 MiB sent
    ///   persistent keepalive: every 25 seconds
    ///
    /// interface: wg1
    ///   public key: public_key3
    ///   private key: (hidden)
    ///   listening port: 51820
    ///
    /// peer: public_key4
    ///   preshared key: (hidden)
    ///   endpoint: 81.71.149.31:51820
    ///   allowed ips: 10.13.13.0/24
    ///   latest handshake: 1 minute, 31 seconds ago
    ///   transfer: 1.18 MiB received, 3.89 MiB sent
    ///   persistent keepalive: every 25 seconds
    /// ";
    ///         // 解析
    /// let b = Interface::from_wg(a);
    ///
    /// ```
    pub fn from_wg(s: &str) -> Result<Vec<Self>, String> {
        // 解析
        let b = s.split("\n")
            .map(|line| {
                line.trim().split(": ").collect::<Vec<&str>>()
            }).collect::<Vec<_>>();

        let mut interfaces: Vec<Self> = Vec::new();
        let mut tmp_interfacse = Self::new();
        let mut tmp_peer = Peer::new();
        for data in b {
            if data.len() < 2 {
                continue;
            }
            let (key, value) = (data[0], data[1]);
            match key {
                "interface" => {
                    if !tmp_peer.is_empty() {
                        tmp_interfacse.peers.push(tmp_peer);
                        tmp_peer = Peer::new();
                    }
                    if !tmp_interfacse.is_empty() {
                        interfaces.push(tmp_interfacse);
                        tmp_interfacse = Self::new();
                    }
                    tmp_interfacse.name.push_str(value);
                }
                "public key" => {
                    if tmp_interfacse.is_empty() {
                        println!("{}", value);
                        continue;
                    }
                    tmp_interfacse.public_key = value.to_string();
                }
                "listening port" => {
                    if tmp_interfacse.is_empty() {
                        continue;
                    }
                    let port = value.parse::<u16>();
                    tmp_interfacse.listening_port = match port {
                        Ok(port) => { port }
                        Err(_) => { return Err(format!("解析出错: {}: {}", key, value)); }
                    };
                }
                "peer" => {
                    if !tmp_peer.is_empty() {
                        if tmp_interfacse.is_empty() {
                            continue;
                        }
                        tmp_interfacse.peers.push(tmp_peer);
                        tmp_peer = Peer::new();
                    }
                    tmp_peer.public_key.push_str(value);
                }
                "endpoint" => {
                    if tmp_peer.is_empty() {
                        continue;
                    }
                    tmp_peer.endpoint = value.parse()?
                }
                "allowed ips" => {
                    if tmp_peer.is_empty() {
                        continue;
                    }
                }
                "latest handshake" => {
                    if tmp_peer.is_empty() {
                        continue;
                    }
                    tmp_peer.latest_handshake = value.parse()?
                }
                "transfer" => {
                    if tmp_peer.is_empty() {
                        continue;
                    }
                }
                "persistent keepalive" => {
                    if tmp_peer.is_empty() {
                        continue;
                    }
                    tmp_peer.persistent_keepalive = value.parse()?
                }
                &_ => {}
            }
        }
        if !tmp_peer.public_key.is_empty() {
            tmp_interfacse.peers.push(tmp_peer);
        }
        if !tmp_interfacse.is_empty() {
            interfaces.push(tmp_interfacse);
        }
        Ok(interfaces)
    }
}

impl Peer {
    fn new() -> Self {
        Self {
            public_key: String::new(),
            latest_handshake: Default::default(),
            endpoint: AddrPort::new(),
            allowed_ips: Vec::new(),
            transfer: Default::default(),
            persistent_keepalive: Default::default(),
        }
    }
    fn is_empty(&self) -> bool {
        self.public_key.is_empty()
    }
}

impl Default for Transfer {
    fn default() -> Self {
        Self { received: Default::default(), sent: Default::default() }
    }
}

mod t {
    use ipnet::IpNet;
    use crate::Interface;
    use crate::parse::AddrPort;

    #[test]
    fn aa() {
        let a: AddrPort = "127.0.0.100:123".parse().unwrap();
        println!("{}", a.to_string());
    }

    #[test]
    fn tt() {
        let a = "
interface: wg0
  public key: public_key1
  private key: (hidden)
  listening port: 51820

peer: public_key2
  preshared key: (hidden)
  endpoint: 1.1.1.1:1
  allowed ips: 1.1.2.0/24
  latest handshake: 1 minute, 31 seconds ago
  transfer: 1.18 MiB received, 3.89 MiB sent
  persistent keepalive: every 25 seconds

interface: wg1
  public key: public_key3
  private key: (hidden)
  listening port: 51820

peer: public_key4
  preshared key: (hidden)
  endpoint: 81.71.149.31:51820
  allowed ips: 10.13.13.0/24
  latest handshake: 1 minute, 31 seconds ago
  transfer: 1.18 MiB received, 3.89 MiB sent
  persistent keepalive: every 25 seconds";

        // 解析
        Interface::from_wg(a).unwrap();
    }
}