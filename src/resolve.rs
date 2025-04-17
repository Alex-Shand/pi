use std::{
    collections::HashMap,
    fs::{self, File},
    io::{ErrorKind, Write as _},
    net::Ipv4Addr,
    process::Command,
};

use anyhow::{Context, Result, anyhow, bail};
use argh::FromArgs;
use command_ext::CommandExt;

use crate::{identity::Identity, utils};

const SSH_DB: &str = "ssh_db";

/// Find the current IP address of a managed pi
#[derive(Debug, FromArgs)]
#[argh(subcommand, name = "resolve")]
pub(crate) struct Args {
    /// the hostname of the pi
    #[argh(positional)]
    name: String,
}

pub(crate) fn main(Args { name }: Args) -> Result<()> {
    println!("{}", resolve(name)?);
    Ok(())
}

/// Resolve the IP address of the pi
///
/// Checks initially against the cached IP address using ssh, failing that
/// probes for a new address using avahai
///
/// # Errors
/// If the ssh or avahai processes can't be launched
pub fn resolve(name: impl AsRef<str>) -> Result<Ipv4Addr> {
    let name = name.as_ref();
    let mut db = load_db().context("Failed to load IP Database")?;
    let need_new_ip = if let Some(ip) = db.get(name) {
        !ssh_works(name, *ip)?
    } else {
        true
    };

    if need_new_ip {
        let _ = db.insert(
            String::from(name),
            probe(name).context("IP probe failed")?,
        );
    }

    save_db(&db).context("Failed to save IP Database")?;
    Ok(db[name])
}

pub(crate) fn probe(name: &str) -> Result<Ipv4Addr> {
    loop {
        if let Some(result) = try_probe(name)? {
            return Ok(result);
        }
    }
}

fn ssh_works(name: &str, ip: Ipv4Addr) -> Result<bool> {
    // Not using run_on_pi since we can't go through resolve
    let output = Command::new("hostname")
        .run_on_remote("pi", ip, Identity::private(name)?)
        .check_output()?;
    Ok(output.trim() == name)
}

fn try_probe(name: &str) -> Result<Option<Ipv4Addr>> {
    let hostname = format!("{name}.local");
    let stdout = Command::new("avahi-resolve")
        .args(["--name", &hostname])
        .check_output()?;

    if stdout.is_empty() {
        return Ok(None);
    }

    let parts = stdout
        .split_whitespace()
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>();
    if parts.len() != 2 || parts[0] != hostname {
        bail!("Couldn't get IP Address for {name}")
    }
    Ok(Some(parts[1].parse()?))
}

fn load_db() -> Result<HashMap<String, Ipv4Addr>> {
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
        let ip = parts[1].parse::<Ipv4Addr>()?;
        let _ = result.insert(String::from(parts[0]), ip);
    }
    Ok(result)
}

fn save_db(db: &HashMap<String, Ipv4Addr>) -> Result<()> {
    let mut file = File::create(utils::app_config()?.join(SSH_DB))?;
    for (name, ip) in db {
        writeln!(file, "{name} {ip}")?;
    }
    Ok(())
}
