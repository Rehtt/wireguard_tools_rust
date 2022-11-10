use crate::interface::{AddrPort, AllowedIPs, Interface, Peer, TimeDuration, Transfer};
use bytes_size::ByteSize;
use ini_lib::ini_str;
use ipnet::IpNet;
use std::fs::File;
use std::io::Read;
use std::net::IpAddr;
use std::ops::Add;
use std::path::Path;
use std::str::FromStr;
use std::time::Duration;

impl FromStr for AddrPort {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let a = s.trim().split(":").collect::<Vec<&str>>();
        if a.len() != 2 {
            return Err(format!("解析错误"));
        }

        Ok(Self {
            addr: match a[0].parse() {
                Ok(a) => a,
                Err(_) => {
                    return Err(format!("地址错误：{}", a[0]));
                }
            },
            port: match a[1].parse() {
                Ok(p) => p,
                Err(_) => {
                    return Err(format!("端口错误：{}", a[1]));
                }
            },
        })
    }
}

impl ToString for AddrPort {
    fn to_string(&self) -> String {
        format!("{}:{}", self.addr, self.port)
    }
}

impl FromStr for TimeDuration {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // 单数字时默认秒
        if let Ok(x) = s.parse::<u64>() {
            return Ok(Self(Duration::from_secs(x)));
        }

        // 解析 例如：1day, 2 minute, 3 seconds
        let a = s
            .trim()
            .trim_matches(|x| x == ',' || x == '.')
            .split(" ")
            .collect::<Vec<&str>>();
        let mut num = Duration::default();
        let mut tmp: u64 = 0;
        for x in a {
            if let Ok(a) = x.parse::<u64>() {
                tmp += a;
                continue;
            }
            if x.contains("second") {
                if tmp != 0 {
                    num = num.add(Duration::from_secs(tmp));
                    tmp = 0
                }
            } else if x.contains("minute") {
                if tmp != 0 {
                    num = num.add(Duration::from_secs(tmp * 60));
                    tmp = 0
                }
            } else if x.contains("day") {
                if tmp != 0 {
                    num = num.add(Duration::from_secs(tmp * 60 * 24));
                    tmp = 0
                }
            }
        }
        Ok(TimeDuration(num))
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
impl Interface {
    /// 解析wg命令输出结果
    ///
    /// # Example
    /// ```
    /// use wireguard_tools_rehtt::Interface;
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
            .filter(|x| x.len() == 2)
            .collect::<Vec<_>>();

        let mut interfaces: Vec<Self> = Vec::new();
        let mut tmp_interfacse = Self::new();
        let mut tmp_peer = Peer::new();
        for data in b {
            let (key, value) = (data[0], data[1]);
            match key {
                "interface" => {
                    if !tmp_peer.is_none() {
                        tmp_interfacse.peers.push(tmp_peer);
                        tmp_peer = Peer::new();
                    }
                    if !tmp_interfacse.is_none() {
                        interfaces.push(tmp_interfacse);
                        tmp_interfacse = Self::new();
                    }
                    tmp_interfacse.name = Some(value.to_string());
                }
                "public key" => {
                    if tmp_interfacse.is_none() {
                        continue;
                    }
                    tmp_interfacse.public_key = Some(value.to_string());
                }
                "listening port" => {
                    if tmp_interfacse.is_none() {
                        continue;
                    }
                    if let Ok(p) = value.parse::<u16>() {
                        tmp_interfacse.listening_port = Some(p)
                    }
                }
                "peer" => {
                    if !tmp_peer.is_none() {
                        if tmp_interfacse.is_none() {
                            continue;
                        }
                        tmp_interfacse.peers.push(tmp_peer);
                        tmp_peer = Peer::new();
                    }
                    tmp_peer.public_key = Some(value.to_string());
                }
                "endpoint" => {
                    if tmp_peer.is_none() {
                        continue;
                    }
                    if let Ok(e) = value.parse::<AddrPort>() {
                        tmp_peer.endpoint = e
                    }
                }
                "allowed ips" => {
                    if tmp_peer.is_none() {
                        continue;
                    }
                    for x in value.split(",") {
                        if let Ok(data) = x.parse::<IpNet>() {
                            tmp_peer.allowed_ips.0.push(data)
                        }
                    }
                }
                "latest handshake" => {
                    if tmp_peer.is_none() {
                        continue;
                    }
                    if let Ok(l) = value.parse::<TimeDuration>() {
                        tmp_peer.latest_handshake = Some(l);
                    }
                }
                "transfer" => {
                    if tmp_peer.is_none() {
                        continue;
                    }
                    let mut tmp = Transfer::default();
                    for x in value.split(", ") {
                        if let Some(index) = x.find(" received") {
                            if let Ok(data) = ByteSize::from_str(&x[0..index]) {
                                tmp.received = data;
                            }
                        } else if let Some(index) = x.find(" sent") {
                            if let Ok(data) = ByteSize::from_str(&x[0..index]) {
                                tmp.sent = data;
                            }
                        }
                    }
                    tmp_peer.transfer = Some(tmp);
                }
                "persistent keepalive" => {
                    if tmp_peer.is_none() {
                        continue;
                    }
                    if let Ok(t) = value.parse::<TimeDuration>() {
                        tmp_peer.persistent_keepalive = Some(t)
                    }
                }
                &_ => {}
            }
        }
        if !tmp_peer.public_key.is_none() {
            tmp_interfacse.peers.push(tmp_peer);
        }
        if !tmp_interfacse.is_none() {
            interfaces.push(tmp_interfacse);
        }
        Ok(interfaces)
    }
}

/// 解析hashmap并赋值
macro_rules! parse_from_hashmap {
    ($from: expr,<$ty: ty> $key: expr) => {
        match $from.get($key) {
            None => None,
            Some(data) => match data {
                None => None,
                Some(data) => match data.parse::<$ty>() {
                    Ok(data) => Some(data),
                    Err(_) => None,
                },
            },
        }
    };
    ($from: expr,<$ty: ty> $key: expr => $to:expr) => {
        {
            if let Some(data) = parse_from_hashmap!($from,<$ty> $key){
                $to = data
            }
        }
    };
    ($from: expr, Some <$ty: ty> $key: expr => $to:expr) => {
        {
            $to = parse_from_hashmap!($from,<$ty> $key);
        }
    };
}

impl Interface {
    pub fn from_file_str(s: &str, name: &str) -> Result<Self, String> {
        return match ini_str!(s) {
            Ok(data) => {
                let mut imterface: Self = Self::new();
                imterface.name = Some(name.to_string());

                for value in data.iter() {
                    if value.name.eq("Interface") {
                        parse_from_hashmap!(value.sub,Some<IpAddr>"DNS"=>imterface.dns);
                        parse_from_hashmap!(value.sub,Some<IpNet>"Address"=>imterface.address);
                        parse_from_hashmap!(value.sub,Some<String>"PrivateKey"=>imterface.private_key);
                        parse_from_hashmap!(value.sub,Some<u16>"ListenPort"=>imterface.listening_port);
                    }
                    if value.name.eq("Peer") {
                        let mut tmp_peer = Peer::new();
                        parse_from_hashmap!(value.sub,Some<String>"PublicKey"=>tmp_peer.public_key);
                        parse_from_hashmap!(value.sub,Some<String>"PresharedKey"=>tmp_peer.preshared_key);
                        parse_from_hashmap!(value.sub,Some<TimeDuration>"PersistentKeepalive"=>tmp_peer.persistent_keepalive);
                        parse_from_hashmap!(value.sub,<AddrPort>"Endpoint"=>tmp_peer.endpoint);
                        parse_from_hashmap!(value.sub,<AllowedIPs>"AllowedIPs"=>tmp_peer.allowed_ips);
                        imterface.peers.push(tmp_peer.clone());
                    }
                }
                Ok(imterface)
            }
            Err(e) => Err(e),
        };
    }

    /// 解析wireguard配置文件
    ///
    /// # Example
    /// ```
    /// use wireguard_tools_rehtt::Interface;
    /// let data = Interface::from_file("/etc/wireguard/wg0.conf");
    /// println!("{:#?}",data);
    /// ```
    pub fn from_file(path: &str) -> Result<Self, String> {
        return match File::open(path) {
            Ok(mut file) => {
                let name = Path::new(path).file_stem().unwrap().to_str().unwrap();
                let mut data = String::new();
                match file.read_to_string(&mut data) {
                    Ok(_) => Self::from_file_str(data.as_str(), name),
                    Err(err) => Err(err.to_string()),
                }
            }
            Err(e) => Err(e.to_string()),
        };
    }
}
