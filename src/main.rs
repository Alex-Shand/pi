fn main() -> std::process::ExitCode {
    match pi::main() {
        Ok(code) => code,
        Err(e) => {
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}
