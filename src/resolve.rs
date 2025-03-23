use std::{
    collections::HashMap,
    fs::{self, File},
    io::{ErrorKind, Write as _},
    process::Command,
};

use anyhow::{anyhow, bail, Context, Result};
use argh::FromArgs;

use crate::utils;

const SSH_DB: &str = "ssh_db";

/// Find the current IP address of a managed pi
#[derive(Debug, FromArgs)]
#[argh(subcommand, name = "resolve")]
pub(crate) struct Args {
    /// the hostname of the pi
    #[argh(positional)]
    name: String,
}

pub(crate) fn main(args: &Args) -> Result<()> {
    println!("{}", resolve(&args.name)?);
    Ok(())
}

/// Resolve the IP address of the pi
///
/// Checks initially against the cached IP address using ssh, failing that
/// probes for a new address using avahai
///
/// # Errors
/// If the ssh or avahai processes can't be launched
pub fn resolve(name: &str) -> Result<String> {
    let mut db = load_db().context("Failed to load IP Database")?;
    let need_new_ip = if let Some(ip) = db.get(name) {
        !ssh_works(name, ip)?
    } else {
        true
    };

    if need_new_ip {
        drop(db.insert(
            String::from(name),
            probe(name).context("IP probe failed")?,
        ));
    }

    save_db(&db).context("Failed to save IP Database")?;
    Ok(db[name].clone())
}

pub(crate) fn probe(name: &str) -> Result<String> {
    loop {
        if let Some(result) = try_probe(name)? {
            return Ok(result);
        }
    }
}

fn ssh_works(name: &str, ip: &str) -> Result<bool> {
    let output = utils::ssh_cmd(name, ip)?.arg("hostname").output()?;
    Ok(output.status.success()
        && String::from_utf8(output.stdout)?.trim() == name)
}

fn try_probe(name: &str) -> Result<Option<String>> {
    let hostname = format!("{name}.local");
    let output = Command::new("avahi-resolve")
        .args(["--name", &hostname])
        .output()
        .context("Unable to execute avahi-resolve")?;
    if !output.status.success() {
        bail!("avahi failed")
    }

    if output.stdout.is_empty() {
        return Ok(None);
    }

    let parts = String::from_utf8(output.stdout)?
        .split_whitespace()
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    if parts.len() != 2 || parts[0] != hostname {
        bail!("Couldn't get IP Address for {name}")
    }
    Ok(Some(parts[1].clone()))
}

fn load_db() -> Result<HashMap<String, String>> {
    let db = utils::app_config()?.join(SSH_DB);
    let contents = match fs::read_to_string(&db) {
        Ok(contents) => contents,
        Err(e) => {
            if let ErrorKind::NotFound = e.kind() {
                return Ok(HashMap::new());
            }
            return Err(
                anyhow!(e).context(format!("Failed to read {}", db.display()))
            );
        }
    };
    let mut result = HashMap::new();
    for line in contents.lines() {
        let parts = line.split_whitespace().collect::<Vec<_>>();
        if parts.len() != 2 {
            bail!("Invalid DB format")
        }
        drop(result.insert(String::from(parts[0]), String::from(parts[1])));
    }
    Ok(result)
}

fn save_db(db: &HashMap<String, String>) -> Result<()> {
    let mut file = File::create(utils::app_config()?.join(SSH_DB))?;
    for (name, ip) in db {
        writeln!(file, "{name} {ip}")?;
    }
    Ok(())
}
