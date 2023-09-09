//! RaspberryPi based service manager

#![warn(elided_lifetimes_in_paths)]
#![warn(missing_docs)]
#![warn(noop_method_call)]
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
//#![deny(dead_code)]
#![warn(clippy::pedantic)]
#![allow(clippy::doc_markdown)]
#![allow(clippy::let_underscore_untyped)]

#[macro_use]
mod macros;

mod image;
mod resolve;
mod ssh;
mod register;
mod utils;
mod identity;
mod mount;
mod push;
mod secure;
mod setup;

use std::process::ExitCode;

use argh::FromArgs;
use anyhow::Result;

pub use ssh::ssh;
pub use resolve::resolve;

/// Manager for services running on RaspberryPis
#[derive(Debug, FromArgs)]
struct Args {
    #[argh(subcommand)]
    command: Command
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
    Secure(secure::Args),
    Setup(setup::Args),
}

macro_rules! x {
    ($e:expr) => { {
        $e?;
        Ok(ExitCode::SUCCESS)
    } }
}

#[allow(missing_docs)]
#[allow(clippy::missing_errors_doc)]
pub fn main() -> Result<ExitCode> {
    let args: Args = argh::from_env();
    match args.command {
        Command::Image(args) => x!(image::main(&args)),
        Command::Resolve(args) => x!(resolve::main(&args)),
        Command::Register(args) => x!(register::main(&args)),
        Command::Mount(args) => x!(mount::main(&args)),
        Command::Push(args) => x!(push::main(&args)),
        Command::Secure(args) => x!(secure::main(&args)),
        Command::Setup(args) => x!(setup::main(&args)),
        Command::Ssh(args) => Ok(
            ssh::main(&args)?.code()
                .map_or(
                    ExitCode::FAILURE,
                    |code| ExitCode::from(utils::truncate(code))
                )
        ),
    }
}
