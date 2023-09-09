use crate::{utils::{self, DefaultPrompt}, resolve, identity::Identity};

use std::collections::HashMap;
use std::process::Command;
use std::fs::{self, File};
use std::io::Write as _;

use argh::FromArgs;
use anyhow::{Result, anyhow, bail};

/// Register a newly imaged pi
#[derive(Debug, FromArgs)]
#[argh(subcommand, name="register")]
pub(crate) struct Args {
    /// the pi to register
    #[argh(positional)]
    name: String
}

pub(crate) fn main(args: &Args) -> Result<()> {
    let id = Identity::new(&args.name)?;
    let id = if id.exists() {
        check_reuse(&args.name, id)?
    } else {
        generate_id(&args.name, id, false)?
    };

    prompt!("Attempting partial IP resolution for {}. This may take a while...", args.name);
    let ip = resolve::probe(&args.name)?;
    println!("Done");

    let mut known_hosts = read_known_hosts()?;
    if known_hosts.contains_key(&ip) {
        prompt!("IP Address {ip} is already in known_hosts, remove?: [Y/n]: ");
        if utils::read_prompt(DefaultPrompt::Yes)? {
            drop(known_hosts.remove(&ip));
            write_known_hosts(known_hosts)?;
        }
    }

    send_id(&id, &ip)?;

    prompt!("Identity installed, running full IP resolution...");
    drop(resolve(&args.name)?);
    println!("Done");

    Ok(())
}

fn check_reuse(name: &str, id: Identity) -> Result<Identity> {
    prompt!("Found existing identity for {}, reuse? [Y/n]: ", name);
    if utils::read_prompt(DefaultPrompt::Yes)? {
        return Ok(id);
    }

    prompt!("Overwriting previous identity for {}? [y/N]: ", name);
    if utils::read_prompt(DefaultPrompt::No)? {
        return generate_id(name, id, true);
    }

    bail!("Aborting identity creation")
}

fn generate_id(name: &str, id: Identity, force: bool) -> Result<Identity> {
    if force && id.exists() {
        id.delete()?;
    }
    let success = Command::new("ssh-keygen")
        .args(["-t", "rsa"]) // Key format
        .args(["-N", ""]) // No password
        .args(["-C", &format!("Auto-generated key for {name}.local")]) // Comment
        .arg("-f").arg(&id.private) // Key location
        .status()?.success();
    if !success {
        bail!("ssh-keygen failed")
    }
    Ok(id)
}

fn read_known_hosts() -> Result<HashMap<String, Vec<String>>> {
    let text = fs::read_to_string(utils::home()?.join(".ssh").join("known_hosts"))?;
    Ok(text.lines()
       .map(|s| s.split_once(char::is_whitespace))
       .collect::<Option<Vec<_>>>()
       .ok_or_else(|| anyhow!("Failed to parse known_hosts"))?
       .iter()
       .fold(HashMap::new(), |mut map, &(ip, key)| {
           map.entry(ip.to_string())
               .or_insert(Vec::new())
               .push(key.to_string());
           map
       }))
}

fn write_known_hosts(data: HashMap<String, Vec<String>>) -> Result<()> {
    let mut known_hosts = File::create(utils::home()?.join(".ssh").join("known_hosts"))?;
    for (ip, keys) in data {
        for key in keys {
            writeln!(known_hosts, "{ip} {key}")?;
        }
    }
    Ok(())
}

fn send_id(id: &Identity, ip: &str) -> Result<()> {
    let success = Command::new("ssh-copy-id")
        .arg("-i").arg(&id.public)
        .arg(format!("pi@{ip}"))
        .status()?.success();
    if !success {
        bail!("ssh-copy-id failed")
    }
    Ok(())
}
