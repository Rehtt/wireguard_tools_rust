use std::net::IpAddr;
use std::ops::Add;
use std::str::FromStr;
use std::time::Duration;

#[derive(Debug)]
#[derive(Clone)]
pub struct AddrPort {
    addr: IpAddr,
    port: u64,
}

impl FromStr for AddrPort {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let a = s.trim().split(":").collect::<Vec<&str>>();
        if a.len() != 2 {
            return Err(format!("解析错误"));
        }

        Ok(Self {
            addr: match a[0].parse() {
                Ok(a) => { a }
                Err(_) => { return Err(format!("地址错误：{}", a[0])); }
            },
            port: match a[1].parse() {
                Ok(p) => { p }
                Err(_) => { return Err(format!("端口错误：{}", a[1])); }
            },
        })
    }
}

impl ToString for AddrPort {
    fn to_string(&self) -> String {
        format!("{}:{}", self.addr, self.port)
    }
}

impl AddrPort {
    pub fn new() -> Self {
        Self { addr: "0.0.0.0".parse().unwrap(), port: 0 }
    }
}

#[derive(Debug)]
#[derive(Clone)]
pub struct TimeDuration(Duration);

impl Default for TimeDuration {
    fn default() -> Self {
        Self(Duration::default())
    }
}

impl FromStr for TimeDuration {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let a = s.trim().trim_matches(|x| x == ',' || x == '.').split(" ").collect::<Vec<&str>>();
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