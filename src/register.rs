use std::{
    collections::HashMap,
    fs::{self, File},
    io::Write as _,
    net::Ipv4Addr,
    process::Command,
};

use anyhow::{Result, anyhow, bail};
use argh::FromArgs;
use command_ext::CommandExt as _;

use crate::{
    identity::{Created, Identity, Unknown},
    resolve,
    utils::{self, Prompt},
};

/// Register a newly imaged pi
#[derive(Debug, FromArgs)]
#[argh(subcommand, name = "register")]
pub(crate) struct Args {
    /// the pi to register
    #[argh(positional)]
    name: String,
}

pub(crate) fn main(Args { name }: Args) -> Result<()> {
    let id = match Identity::new_unknown(&name)?.exists() {
        Ok(id) => check_reuse(&name, id)?,
        Err(id) => generate_id(&name, id)?,
    };

    prompt!(
        "Attempting partial IP resolution for {name}. This may take a while...",
    );
    let ip = resolve::probe(&name)?;
    println!("Done");

    let mut known_hosts = read_known_hosts()?;
    if known_hosts.contains_key(&ip) {
        prompt!("IP Address {ip} is already in known_hosts, remove?: [Y/n]: ");
        if utils::read_prompt(Prompt::Yes)?.is_yes() {
            drop(known_hosts.remove(&ip));
            write_known_hosts(known_hosts)?;
        }
    }

    send_id(&id, ip)?;

    prompt!("Identity installed, running full IP resolution...");
    let _ = resolve(&name)?;
    println!("Done");

    Ok(())
}

fn check_reuse(name: &str, id: Identity<Created>) -> Result<Identity<Created>> {
    prompt!("Found existing identity for {}, reuse? [Y/n]: ", name);
    if utils::read_prompt(Prompt::Yes)?.is_yes() {
        return Ok(id);
    }

    prompt!("Overwrite previous identity for {}? [y/N]: ", name);
    if utils::read_prompt(Prompt::No)?.is_yes() {
        let id = id.delete()?;
        return generate_id(name, id);
    }

    bail!("Aborting identity creation")
}

fn generate_id(name: &str, id: Identity<Unknown>) -> Result<Identity<Created>> {
    let success = Command::new("ssh-keygen")
        .args(["-t", "rsa"]) // Key format
        .args(["-N", ""]) // No password
        .args(["-C", &format!("Auto-generated key for {name}.local")]) // Comment
        .arg("-f")
        .arg(&id.private) // Key location
        .status()?
        .success();
    if !success {
        bail!("ssh-keygen failed")
    }
    Ok(id
        .exists()
        .ok()
        .expect("Identity should exist because we just created it"))
}

fn read_known_hosts() -> Result<HashMap<Ipv4Addr, Vec<String>>> {
    let text =
        fs::read_to_string(utils::home()?.join(".ssh").join("known_hosts"))?;
    Ok(text
        .lines()
        .map(|s| s.split_once(char::is_whitespace))
        .collect::<Option<Vec<_>>>()
        .ok_or_else(|| anyhow!("Failed to parse known_hosts"))?
        .into_iter()
        .map(|(ip, key)| Ok((ip.parse::<Ipv4Addr>()?, key)))
        .collect::<Result<Vec<_>>>()?
        .into_iter()
        .fold(HashMap::new(), |mut map, (ip, key)| {
            map.entry(ip).or_default().push(key.to_string());
            map
        }))
}

fn write_known_hosts(data: HashMap<Ipv4Addr, Vec<String>>) -> Result<()> {
    let mut known_hosts =
        File::create(utils::home()?.join(".ssh").join("known_hosts"))?;
    for (ip, keys) in data {
        for key in keys {
            writeln!(known_hosts, "{ip} {key}")?;
        }
    }
    Ok(())
}

fn send_id(id: &Identity<Created>, ip: Ipv4Addr) -> Result<()> {
    Command::new("ssh-copy-id")
        .arg("-i")
        .arg(&id.public)
        .arg(format!("pi@{ip}"))
        .check_status()
}
