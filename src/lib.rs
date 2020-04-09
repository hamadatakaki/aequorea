pub mod core;

#[macro_export]
macro_rules! exit_process {
    ($code: expr, $format: expr, $message: expr) => {{
        println!($format, $message);
        std::process::exit($code);
    }};
}
