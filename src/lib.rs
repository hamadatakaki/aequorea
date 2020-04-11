pub mod core;
pub mod command;

#[macro_export]
macro_rules! exit_process_with_error {
    ($code: expr, $format: expr, $message: expr) => {{
        println!($format, $message);
        std::process::exit($code);
    }};
}
