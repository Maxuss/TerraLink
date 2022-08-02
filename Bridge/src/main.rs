use std::path::Path;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::append::rolling_file::policy::compound::CompoundPolicy;
use log4rs::append::rolling_file::policy::compound::roll::fixed_window::FixedWindowRoller;
use log4rs::append::rolling_file::policy::compound::trigger::size::SizeTrigger;
use log4rs::append::rolling_file::RollingFileAppender;
use log4rs::{Config, init_config};
use log4rs::config::{Appender, Logger, Root};
use log4rs::encode::pattern::PatternEncoder;
use log::{info, LevelFilter};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use crate::opt::BridgeOptions;
use crate::server::bind;

pub mod proto;
pub mod server;
pub mod opt;
pub mod client;

#[tokio::main]
async fn main() {
    configure_logging().await;

    let opts = File::open("terralink.toml").await;
    if let Err(_) = opts {
        println!("Looks like it's your first time using TerraLink bridge! Options file was created in terralink.toml");
        println!("Restart the server to apply changes!");

        let mut file = File::create("terralink.toml").await.unwrap();
        file.write_all(&toml::ser::to_string(&BridgeOptions::default()).unwrap().into_bytes()).await.unwrap();
        return;
    }
    let mut strbuf = String::new();
    opts.unwrap().read_to_string(&mut strbuf).await.unwrap();
    let opts: BridgeOptions = toml::de::from_str(&strbuf).unwrap();

    info!("Starting bridge...");
    bind(opts).await.unwrap()
}

async fn configure_logging() {
    let path = Path::new("./logs/latest.log");
    if path.exists() {}

    let pattern = "[{d(%Y-%m-%d %H:%M:%S)}] <{M}> {h([{l}])}: {m}\n";
    let stdout = ConsoleAppender::builder()
        .encoder(Box::new(PatternEncoder::new(pattern)))
        .build();

    let logfile = RollingFileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(pattern)))
        .build(
            "logs/latest.log",
            Box::new(CompoundPolicy::new(
                Box::new(SizeTrigger::new(4 * 1024)),
                Box::new(
                    FixedWindowRoller::builder()
                        .build("logs/log_{}.old.gz", 4)
                        .expect("Could not initialize logger roller."),
                ),
            )),
        )
        .expect("Could not initialize file logging");

    let config = Config::builder()
        .appender(Appender::builder().build("stdout", Box::new(stdout)))
        .appender(Appender::builder().build("logfile", Box::new(logfile)))
        .logger(Logger::builder().build("soulflame::general", LevelFilter::Debug))
        .build(
            Root::builder()
                .appender("stdout")
                .appender("logfile")
                .build(LevelFilter::Info),
        )
        .expect("Could not build logger config");

    init_config(config).expect("Could not initialize logger config");
}