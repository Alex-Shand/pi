use crate::utils;

use std::io::Read as _;
use std::io::Write as _;
use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::Command;

use once_cell::sync::Lazy;
use argh::FromArgs;
use nix::unistd::Uid;
use anyhow::{Result, anyhow, bail};
#[cfg(debug_assertions)]
use sliding_windows as _;

const SDPATH: &str = "/dev/sdb";
const BOOT_PARTITION: &str = "/dev/sdb1";
const ROOT_PARTITION: &str = "/dev/sdb2";

const WIFI: &str = "country=GB\n\
                    ctrl_interface=DIR=/var/run/wpa_supplicant GROUP=netdev\n\
                    update_config=1\n\
                    \n\
                    network={\n\
                       ssid=\"{ssid}\"\n\
                       psk=\"{pass}\"\n\
                       }";
// Password hash generated using: echo "raspberry" | openssl passwd -6 -stdin
const USERCONF: &str = "pi:$6$0tJ78aVORQEk8spk$LT66yvA.gwx7jGxJFBSoQF7GTeJDzrqJNuQWdHg8y05917vCWHVqb9ECH0EDspGT7zJz81Z8Rs6vD0Cq3Kthb1";

#[cfg(debug_assertions)]
static RASPBIAN: Lazy<Result<Vec<u8>>> = Lazy::new(|| {
    Ok(File::open(env!("IMAGE"))?.bytes().collect::<Result<Vec<_>, _>>()?)
});

#[cfg(not(debug_assertions))]
static RASPBIAN: Lazy<Result<Vec<u8>>> = Lazy::new(|| {
    use std::env;
    use sliding_windows::{Storage, IterExt as _};

    const MARKER: &[u8] = "PI_END".as_bytes();

    let bytes = File::open(env::current_exe()?)?.bytes().collect::<Result<Vec<_>, _>>()?;

    let mut storage: Storage<(usize, u8)> = Storage::new(MARKER.len());
    let end = bytes.iter()
        .copied()
        .enumerate()
        .rev()
        .sliding_windows(&mut storage)
        .map(|window| {
            let mut window = window.iter().copied().collect::<Vec<_>>();
            window.reverse();
            let start = window[0].0;
            let bytes = window.into_iter().map(|(_, byte)| byte).collect::<Vec<_>>();
            (start, bytes)
        })
        .find(|(_, window)| window == MARKER)
        .ok_or_else(|| anyhow!("Couldn't find the end of the embedded ISO"))?.0;
    let start = usize::try_from(
        u64::from_le_bytes(
            <[u8; 8]>::try_from(
                bytes.iter()
                    .copied()
                    .skip(end + MARKER.len())
                    .collect::<Vec<_>>()
            ).map_err(|v| anyhow!("Expected to find 8 bytes after the end marker, got {}", v.len()))?
        )
    )?;
    Ok(
        bytes.into_iter()
            .skip(start)
            .enumerate()
            .take_while(|(index, _)| *index < end - start)
            .map(|(_, byte)| byte)
            .collect::<Vec<_>>()
    )
});

/// Flash a Raspbian image onto the system's SDCard device, enable ssh access
/// and set the hostname
#[derive(Debug, FromArgs)]
#[argh(subcommand, name="image")]
pub(crate) struct Args {
    /// the hostname for the new image
    #[argh(positional)]
    name: String
}

pub(crate) fn main(args: &Args) -> Result<()> {
    if !Uid::effective().is_root() {
        bail!("The image subcommand requires root permissions")
    }

    if !PathBuf::from(SDPATH).exists() {
        bail!("{SDPATH} doesn't exist. Is the SDCard plugged in?")
    }

    prompt!("WARNING: This command will overwrite the SDCard in {SDPATH}. Continue? [y/N]: ");
    if utils::read_line()?.to_lowercase() != "y" {
        bail!("Aborted image operation")
    }

    prompt!("Wifi SSID: ");
    let ssid = utils::read_line()?;

    prompt!("Wifi Password: ");
    let password = utils::read_line()?;

    prompt!("Imaging (this may take a while)...");
    File::create(SDPATH)?.write_all(RASPBIAN.as_ref().map_err(|e| anyhow!(e))?)?;
    println!("Done");

    prompt!("Setting up network & ssh...");
    with(BOOT_PARTITION, |path| {
        drop(File::create(path.join("ssh"))?);

        let mut wpa_supplicant = File::create(path.join("wpa_supplicant.conf"))?;
        write!(
            wpa_supplicant,
            "{}",
            WIFI.replace("{ssid}", &ssid).replace("{pass}", &password)
        )?;

        let cmdline = path.join("cmdline.txt");
        let contents = fs::read_to_string(&cmdline)?;
        let contents = contents.trim();
        let mut cmdline = File::create(cmdline)?;
        write!(cmdline, "{contents} ipv6.disable=1")?;

        let mut userconf = File::create(path.join("userconf"))?;
        write!(userconf, "{USERCONF}")?;

        Ok(())
    })?;
    println!("Done");

    prompt!("Setting hostname to {}...", args.name);
    with(ROOT_PARTITION, |path| {
        let mut hostname = File::create(path.join("etc/hostname"))?;
        write!(hostname, "{}", args.name)?;

        let hosts = path.join("etc/hosts");
        let contents = fs::read_to_string(&hosts)?.replace("raspberrypi", &args.name);
        let mut hosts = File::create(hosts)?;
        write!(hosts, "{contents}")?;

        Ok(())
    })?;
    println!("Done");

    Ok(())
}

fn with(partition: impl AsRef<Path>, f: impl FnOnce(&Path) -> Result<()>) -> Result<()> {
    let tempdir = tempfile::tempdir()?;
    mount(partition, tempdir.path())?;
    let _umount = defer::defer(|| umount(tempdir.path()));
    f(tempdir.path())
}

fn mount(src: impl AsRef<Path>, target: impl AsRef<Path>) -> Result<()> {
    let src = src.as_ref();
    let target = target.as_ref();
    let result = Command::new("mount")
        .args([src.as_os_str(), target.as_os_str()])
        .status()?
        .success();
    if result {
        Ok(())
    } else {
        bail!("mount {src:?} {target:?} failed")
    }
}

fn umount<P: AsRef<Path>>(path: P) {
    fn aux(path: &Path) -> Result<()> {
        let result = Command::new("umount")
            .arg(path.as_os_str())
            .status()?
            .success();
        if result {
            Ok(())
        } else {
            bail!("umount {path:?} failed")
        }
    }
    let path = path.as_ref();
    aux(path).unwrap_or_else(|e| panic!("{e}"));
}
