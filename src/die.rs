macro_rules! die(
    ($fmt:expr) => (
        match writeln!(&mut ::std::io::stderr(), "{}", format!("{}", $fmt).trim()) {
            Ok(_) => ::std::process::exit(1),
            Err(e) => panic!("Unable to write to stderr: {}", e),
        });
    ($($arg:tt)*) => (die!(format_args!($($arg)*)));
    );
