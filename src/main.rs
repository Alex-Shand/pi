fn main() -> std::process::ExitCode {
    // SAFETY: This is the first thing the program does, there is nothing
    // interesting happening at this point
    unsafe { std::env::set_var("RUST_BACKTRACE", "1") };
    match pi::main(argh::from_env()) {
        Ok(code) => code,
        Err(e) => {
            eprintln!("{e:?}");
            std::process::exit(1);
        }
    }
}
