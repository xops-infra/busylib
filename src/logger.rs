// #![allow(unused)]

use std::{env, path::PathBuf};

use log::debug;
use time::UtcOffset;
use tracing_appender::non_blocking::WorkerGuard;
use tracing_subscriber::{
    filter,
    filter::Targets,
    fmt::{time::OffsetTime, MakeWriter},
    layer::SubscriberExt,
    reload,
    reload::Handle,
    util::SubscriberInitExt,
    Layer, Registry,
};

use crate::{config::dev_mode, prelude::EnhancedExpect};

pub type LogHandle = Handle<Targets, Registry>;

pub fn init_logger(
    bin_name: &str,
    crates_to_log: &[&str],
    debug: bool,
) -> (Option<WorkerGuard>, Option<LogHandle>) {
    let timer = OffsetTime::new(
        UtcOffset::from_hms(8, 0, 0).ex("UtcOffset::from_hms should work"),
        time::format_description::well_known::Rfc3339,
    );
    let stdout_log = tracing_subscriber::fmt::layer().with_timer(timer.clone());
    let reg = tracing_subscriber::registry();
    let mut base_filter = Targets::new().with_target(bin_name, filter::LevelFilter::DEBUG);
    for crate_name in crates_to_log {
        base_filter = base_filter.with_target(*crate_name, filter::LevelFilter::DEBUG);
    }
    let (filter, reload_handle) = reload::Layer::new(base_filter.clone());

    if debug {
        let file_appender =
            tracing_appender::rolling::daily(log_path(), format!("{}.log", bin_name));
        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
        let file_filter = tracing_subscriber::fmt::layer()
            .with_timer(timer)
            .with_writer(non_blocking.make_writer())
            .with_filter(base_filter);

        reg.with(stdout_log.with_filter(filter).and_then(file_filter))
            .init();
        debug!("Debug logging is on");
        return (Some(guard), Some(reload_handle));
    } else {
        reg.with(stdout_log.with_filter(filter::LevelFilter::INFO))
            .init();
    }

    (None, None)
}

#[allow(unused)]
pub fn change_debug(handle: &LogHandle, debug: &str) -> bool {
    // TODO: change_debug
    panic!("TODO: ");
    let base_filter = filter::Targets::new().with_target("foo", filter::LevelFilter::DEBUG);
    handle.modify(|filter| *filter = base_filter);
    true
}

fn log_path() -> PathBuf {
    if dev_mode() {
        let dir = env::temp_dir();
        println!(
            "log will be saved to temporary directory: {}",
            dir.display()
        );
        return dir;
    }
    // TODO: log_path read from env
    PathBuf::from(r"/opt/logs/apps/")
}
