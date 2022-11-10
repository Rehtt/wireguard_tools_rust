#![allow(dead_code)]

use crate::Interface;
use std::process::Command;

#[derive(Debug)]
pub struct CMD;
#[derive(Debug)]
pub struct CmdOutputWG(String);
#[derive(Debug)]
pub struct CmdOutputWgFile {
    name: String,
    raw: String,
}

macro_rules! wg_command {
    ($($a:expr),*) => {
        {
            Command::new("wg")$(.arg($a))*.output().expect("failed to execute process")
        }
    };
    (out $($a:expr),*) =>{
        {
            let out = wg_command!($($a),*);
            match out.status.success() {
                true => { Ok(String::from_utf8(out.stdout).unwrap()) }
                false => { Err(String::from_utf8(out.stderr).unwrap()) }
            }
        }
    };
    (out => ($ty:ident) $($a:expr),*) =>{
        {
            let out = wg_command!($($a),*);
            match out.status.success() {
                true => { Ok($ty(String::from_utf8(out.stdout).unwrap())) }
                false => { Err(String::from_utf8(out.stderr).unwrap()) }
            }
        }
    }
}

impl CMD {
    pub fn show(w: &str) -> Result<CmdOutputWG, String> {
        wg_command!(out => (CmdOutputWG) "show", w)
    }
    pub fn show_all() -> Result<CmdOutputWG, String> {
        wg_command!(out => (CmdOutputWG) "show")
    }
    pub fn showconf(w: &str) -> Result<CmdOutputWgFile, String> {
        let r = wg_command!(out "showconf", w)?;
        Ok(CmdOutputWgFile {
            name: w.to_string(),
            raw: r,
        })
    }
    pub fn synconf(interface: &str, config_file: &str) -> Result<String, String> {
        wg_command!(out "synconf", interface, config_file)
    }
    pub fn setconf(interface: &str, config_file: &str) -> Result<String, String> {
        wg_command!(out "setconf", interface, config_file)
    }
    pub fn genkey() -> Result<String, String> {
        wg_command!(out "genkey")
    }
    pub fn genpsk() -> Result<String, String> {
        wg_command!(out "genpsk")
    }
    pub fn pubkey() -> Result<String, String> {
        wg_command!(out "pubkey")
    }
}

impl CmdOutputWG {
    pub fn to_interface(&self) -> Result<Vec<Interface>, String> {
        Interface::from_wg(self.0.as_str())
    }
}

impl CmdOutputWgFile {
    pub fn to_interface(&self) -> Result<Interface, String> {
        Interface::from_file_str(self.raw.as_str(), self.name.as_str())
    }
}
