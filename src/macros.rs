macro_rules! prompt {
    ($($args:tt)*) => {
        { print!($($args)*); ::std::io::Write::flush(&mut ::std::io::stdout())? }
    }
}
