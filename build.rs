use std::env;

type Result<T = ()> = std::result::Result<T, Box<dyn std::error::Error>>;

// macro_rules! warn {
//     ($($tokens: tt)*) => {
//         println!("cargo:warning={}", format!($($tokens)*))
//     }
// }

macro_rules! string {
    ($s:literal) => { $s };
    ($i:ident) => { stringify!($i) };
}

macro_rules! env {
    ($name:tt = $value:expr) => {
        println!("cargo:rustc-env={}={}", string!($name), $value);
    }
}

fn main() -> Result {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR")?;
    let profile = env::var("PROFILE")?;
    if profile.as_str() == "debug" {
        env!(IMAGE = format!("{manifest_dir}/res/raspbian.img"));
    }
    Ok(())
}
