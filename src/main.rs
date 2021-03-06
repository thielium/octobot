extern crate env_logger;
extern crate octobot;
extern crate time;
extern crate thread_id;
extern crate log;

use octobot::config;
use octobot::server;

use env_logger::LogBuilder;
use log::{LogLevelFilter, LogRecord};

use octobot::errors::*;

fn main() {
    if let Err(ref e) = run() {
        use std::io::Write;
        let stderr = &mut ::std::io::stderr();
        let errmsg = "Error writing to stderr";

        writeln!(stderr, "error: {}", e).expect(errmsg);

        for e in e.iter().skip(1) {
            writeln!(stderr, "caused by: {}", e).expect(errmsg);
        }

        // The backtrace is not always generated. Try to run this example
        // with `RUST_BACKTRACE=1`.
        if let Some(backtrace) = e.backtrace() {
            writeln!(stderr, "backtrace: {:?}", backtrace).expect(errmsg);
        }

        ::std::process::exit(1);
    }
}

fn run() -> Result<()> {
    if std::env::args().len() < 2 {
        return Err("Usage: octobot <config-file>".into());
    }

    let config_file = std::env::args().nth(1).unwrap();

    setup_logging();

    let config = config::new(config_file.into()).chain_err(|| "Error parsing config")?;

    server::main::start(config).chain_err(|| "Failed to start server")
}

fn setup_logging() {
    let formatter = |record: &LogRecord| {
        let t = time::now();
        format!(
            "[{},{:03}][{}:{}] - {} - {}",
            time::strftime("%Y-%m-%d %H:%M:%S", &t).unwrap(),
            t.tm_nsec / 1000_000,
            thread_id::get(),
            std::thread::current().name().unwrap_or(""),
            record.level(),
            record.args()
        )
    };

    let mut builder = LogBuilder::new();
    builder.format(formatter).filter(None, LogLevelFilter::Info);

    let is_info;
    if let Ok(ref env_log) = std::env::var("RUST_LOG") {
        builder.parse(env_log);
        is_info = env_log.is_empty() || env_log.to_lowercase() == "info";
    } else {
        is_info = true;
    }

    if is_info {
        builder.filter(Some("rustls"), LogLevelFilter::Warn);
    }

    builder.init().unwrap();
}
