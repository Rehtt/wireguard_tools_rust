mod parse;

use crate::parse::{AddrPort, TimeDuration};
use bytes_size::ByteSize;
use ini::inistr;
use ipnet::{IpAdd, IpNet};
use std::collections::HashMap;
use std::net::{AddrParseError, IpAddr, Ipv4Addr};
use std::ops::Add;
use std::str::FromStr;
use std::time::Duration;

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Interface {
    name: String,
    public_key: String,
    private_key: String,
    listening_port: u16,
    address: IpNet,
    dns: IpAddr,
    peers: Vec<Peer>,
}

#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct Peer {
    public_key: String,
    preshared_key: String,
    latest_handshake: TimeDuration,
    endpoint: AddrPort,
    allowed_ips: AllowedIPs,
    transfer: Transfer,
    persistent_keepalive: TimeDuration,
}

#[derive(Debug, Clone)]
pub struct AllowedIPs(Vec<IpNet>);

#[derive(Debug, Clone)]
pub struct Transfer {
    received: ByteSize,
    sent: ByteSize,
}

impl Interface {
    fn new() -> Self {
        Self {
            name: String::new(),
            public_key: String::new(),
            private_key: String::new(),
            listening_port: 0,
            address: Default::default(),
            dns: IpAddr::V4(Ipv4Addr::from(0)),
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
        let b = s
            .split("\n")
            .map(|line| line.trim().split(": ").collect::<Vec<&str>>())
            .collect::<Vec<_>>();

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
                        Ok(port) => port,
                        Err(_) => {
                            return Err(format!("解析出错: {}: {}", key, value));
                        }
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

macro_rules! parse_from_hashmap {
    ($from: expr,{$(($ty: ty) $key: expr => $to:expr),*}) => {
        {
            $({
                match &$from[$key] {
                    None => {}
                    Some(data) => {
                        match data.parse::<$ty>() {
                            Ok(data) => { $to = data }
                            Err(_) => {}
                        }
                    }
                }
            };)*
        }
    };
}

impl Interface {
    pub fn from_file_str(s: &str, name: &str) -> Result<Self, String> {
        let mut c = configparser::ini::Ini::new();
        let a = c.read(s.to_string()).unwrap();
        println!("{:#?}", a);
        let data = inistr!(s);
        // let mut imterface: Self = Self::new();
        // println!("{:#?}", data);
        // match data.get("interface") {
        //     None => {
        //         return Err(format!("解析错误，没有interface：{}", name.to_string()));
        //     }
        //     Some(interface) => {
        //         imterface.name = name.to_string();
        //         parse_from_hashmap!(interface,{
        //             (IpAddr) "dns"  => imterface.dns,
        //             (IpNet) "address" => imterface.address,
        //             (String) "privatekey" => imterface.private_key,
        //             (u16) "listenport" => imterface.listening_port
        //         });
        //     }
        // }
        // for (key, value) in data.iter() {
        //     if key.eq("peer") {
        //         let mut tmp_peer = Peer::new();
        //         parse_from_hashmap!(value,{
        //             (String) "publickey" =>tmp_peer.public_key,
        //             (String) "presharedkey" => tmp_peer.preshared_key,
        //             (AddrPort) "endpoint" => tmp_peer.endpoint,
        //             (AllowedIPs) "allowedips" => tmp_peer.allowed_ips
        //         });
        //         imterface.peers.push(tmp_peer);
        //     }
        // }
        Err("123".to_string())
        // Interface {
        //     name: "".to_string(),
        //     public_key: "".to_string(),
        //     listening_port: 0,
        //     dns: Default::default(),
        //     peers: vec![],
        // }
    }
}

impl Peer {
    fn new() -> Self {
        Self {
            public_key: String::new(),
            preshared_key: String::new(),
            latest_handshake: Default::default(),
            endpoint: AddrPort::new(),
            allowed_ips: Default::default(),
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
        Self {
            received: Default::default(),
            sent: Default::default(),
        }
    }
}

impl FromStr for AllowedIPs {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut ipnets: Vec<IpNet> = Vec::new();
        for datum in s.trim().split(",").collect::<Vec<_>>().iter() {
            if let Ok(x) = datum.parse::<IpNet>() {
                ipnets.push(x);
            }
        }
        Ok(Self { 0: ipnets })
    }
}

impl ToString for AllowedIPs {
    fn to_string(&self) -> String {
        let mut out = String::new();
        for (i, x) in self.0.iter().enumerate() {
            out.push_str(x.to_string().as_str());
            if i < self.0.len() - 1 {
                out.push_str(",")
            }
        }
        out
    }
}

impl Default for AllowedIPs {
    fn default() -> Self {
        Self(Vec::new())
    }
}

mod t {
    use crate::parse::{AddrPort, InI};
    use crate::Interface;
    use ipnet::IpNet;

    #[test]
    fn parse() {
        let a = "[Interface]
Address = 10.13.13.2/24
PrivateKey = OA2x4YFBii8pgEPvm9Nb7IsBamyfNlTg1lA5m5wyrUo=
ListenPort = 51820
DNS = 8.8.8.8

[Peer]
PublicKey = /A/8ru1OOVcrDMljZcHgxWYH5groyynHxcAdpRca21s=
Endpoint = 116.31.232.209:51820
AllowedIPs = 10.13.13.5/32

[Peer]
PublicKey = SoznFdDKSTgvAIeCMpYHH2y4xvaqJObS3l4AY3XVRzY=
PresharedKey = kguCX9oPV/ACCuaeVOX5OJ9YeLEywsn2oGkCTYN7Fco=
Endpoint = 81.71.149.31:51820
AllowedIPs = 10.13.13.0/24,192.168.31.1/32
PersistentKeepalive = 25";
        a.parse::<InI>();
    }

    #[test]
    fn bb() {
        let a = Interface::from_file_str(
            "[Interface]
Address = 10.13.13.2/24
PrivateKey = OA2x4YFBii8pgEPvm9Nb7IsBamyfNlTg1lA5m5wyrUo=
ListenPort = 51820
DNS = 8.8.8.8

[Peer]
PublicKey = /A/8ru1OOVcrDMljZcHgxWYH5groyynHxcAdpRca21s=
Endpoint = 116.31.232.209:51820
AllowedIPs = 10.13.13.5/32

[Peer]
PublicKey = SoznFdDKSTgvAIeCMpYHH2y4xvaqJObS3l4AY3XVRzY=
PresharedKey = kguCX9oPV/ACCuaeVOX5OJ9YeLEywsn2oGkCTYN7Fco=
Endpoint = 81.71.149.31:51820
AllowedIPs = 10.13.13.0/24,192.168.31.1/32
PersistentKeepalive = 25",
            "wg0",
        );
        println!("{:?}", a)
    }

    #[test]
    fn aa() {
        let a: IpNet = "127.0.0.10/24".parse().unwrap();
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
