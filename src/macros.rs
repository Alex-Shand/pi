macro_rules! prompt {
    ($($args:tt)*) => {
        { print!($($args)*); ::std::io::Write::flush(&mut ::std::io::stdout())? }
    }
}

macro_rules! ssh {
    ($name:expr => $cmd:expr $(, $args:expr)*) => {
        if $crate::ssh::ssh($name, $cmd, &[$($args),*])?.success() {
            Ok(())
        } else {
            let args: Vec<String> = vec![$(format!("{:?}", $args)),*];
            Err(::anyhow::anyhow!(
                "Command: {:?} {} failed",
                $cmd,
                args.join(" ")
            ))
        }
    }
}
