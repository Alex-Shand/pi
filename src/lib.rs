//! RaspberryPi based service manager

#![warn(elided_lifetimes_in_paths)]
#![warn(missing_docs)]
#![warn(unreachable_pub)]
#![warn(unused_crate_dependencies)]
#![warn(unused_import_braces)]
#![warn(unused_lifetimes)]
#![warn(unused_qualifications)]
#![deny(unsafe_code)]
#![deny(unsafe_op_in_unsafe_fn)]
#![deny(unused_results)]
#![deny(missing_debug_implementations)]
#![deny(missing_copy_implementations)]
#![warn(clippy::pedantic)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::let_underscore_untyped)]
#![allow(clippy::similar_names)]

use std::process::ExitCode;

use anyhow::Result;
use argh::FromArgs;
use command_ext::CommandExt as _;

use self::identity::Identity;
pub use self::{
    cat::cat, mount::mount, pull::pull, push::push, resolve::resolve,
    send::send,
};

#[macro_use]
mod macros;
mod cat;
mod identity;
mod image;
mod mount;
mod pull;
mod push;
mod register;
mod resolve;
mod secure;
mod send;
mod setup;
mod ssh;
mod utils;

/// Extension trait for [Command](std::process::Command)
#[sealed::sealed]
pub trait CommandExt: Sized {
    /// Modify the command so that executing it ultimatly runs on the named pi
    ///
    /// # Errors
    /// If pi name resolution fails or identity files cannot be found
    fn run_on_pi(&mut self, name: &str) -> Result<Self>;
}

#[sealed::sealed]
impl CommandExt for std::process::Command {
    fn run_on_pi(&mut self, name: &str) -> Result<Self> {
        let ip = resolve(name)?;
        Ok(self.run_on_remote("pi", ip, Identity::private(name)?))
    }
}

/// Manager for services running on RaspberryPis
///
/// Order for setting up a new PI:
/// * image
/// * register
/// * secure
/// * setup
#[derive(Debug, FromArgs)]
pub struct Args {
    #[argh(subcommand)]
    command: Command,
}

#[derive(Debug, FromArgs)]
#[argh(subcommand)]
enum Command {
    Image(image::Args),
    Resolve(resolve::Args),
    Ssh(ssh::Args),
    Register(register::Args),
    Mount(mount::Args),
    Push(push::Args),
    Pull(pull::Args),
    Secure(secure::Args),
    Setup(setup::Args),
    Cat(cat::Args),
    Send(send::Args),
}

#[allow(missing_docs)]
#[allow(clippy::missing_errors_doc)]
pub fn main(Args { command }: Args) -> Result<ExitCode> {
    match command {
        Command::Image(args) => image::main(args)?,
        Command::Resolve(args) => resolve::main(args)?,
        Command::Register(args) => register::main(args)?,
        Command::Mount(args) => mount::main(args)?,
        Command::Push(args) => push::main(args)?,
        Command::Pull(args) => pull::main(args)?,
        Command::Secure(args) => secure::main(args)?,
        Command::Setup(args) => setup::main(args)?,
        Command::Cat(args) => cat::main(args)?,
        Command::Send(args) => send::main(args)?,
        Command::Ssh(args) => {
            return Ok(ssh::main(args)?
                .code()
                .map_or(ExitCode::FAILURE, |code| {
                    ExitCode::from(utils::truncate(code))
                }));
        }
    }
    Ok(ExitCode::SUCCESS)
}
