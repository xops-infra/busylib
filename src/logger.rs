// #![allow(unused)]

use std::{env, fs, path::PathBuf, thread};

use chrono::{DateTime, Utc};
use job_scheduler::{Job, JobScheduler};
use log::{debug, warn};
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

/// Immediately clean up files in the specified `directory` that have been modified more than
/// a specified number of `days` ago.
/// Typically used to clean up log files with.
///
/// ```rust,ignore
///
/// cleanup_files_immediately("/opt/logs/apps/", 30);
/// ```
pub fn cleanup_files_immediately(directory: &str, days: i64) {
    if let Ok(paths) = fs::read_dir(directory) {
        for path in paths.flatten() {
            let path_buf = path.path();
            if let Ok(modified) = fs::metadata(&path_buf).and_then(|metadata| metadata.modified()) {
                if (Utc::now() - DateTime::from(modified)).num_days() > days {
                    if fs::remove_file(path_buf).is_ok() {
                    } else {
                        warn!("remove log file failed")
                    }
                }
            } else {
                warn!("try to access log file metadata but failed")
            }
        }
    } else {
        warn!("read files from {} failed", directory)
    }
}

/// Clean up files in the specified `directory` that have been modified more than
/// a specified number of `days` ago.
///
/// ```rust,ignore
/// // The parameter `cron_expression` default is `0 0 0 * * * *`.
/// // The parameter `cron_expression` sample: 0 15 6,8,10 * Mar,Jun Fri 2017
/// // means Run at second 0 of the 15th minute of the 6th, 8th, and 10th hour of any day in March
/// // and June that is a Friday of the year 2017.
/// // More information about `cron_expression` parameter see
/// // https://docs.rs/job_scheduler/latest/job_scheduler/
///
/// schedule_cleanup_log_files("", "/opt/logs/apps/", 30);
/// ```
pub fn schedule_cleanup_log_files(cron_expression: &str, directory: &str, days: i64) {
    // cron_expression sample: 0 15 6,8,10 * Mar,Jun Fri 2017
    // Run at second 0 of the 15th minute of the 6th, 8th, and 10th hour
    // of any day in March and June that is a Friday of the year 2017.
    // more information about `cron_expression` see  https://docs.rs/job_scheduler/latest/job_scheduler/
    let mut default_cron_expression = "0 0 0 * * * *";
    if !cron_expression.trim().is_empty() {
        default_cron_expression = cron_expression;
    }
    let mut sched = JobScheduler::new();
    sched.add(Job::new(default_cron_expression.parse().unwrap(), || {
        debug!("start clean log files");
        cleanup_files_immediately(directory, days);
    }));

    loop {
        sched.tick();
        thread::sleep(sched.time_till_next_job());
    }
}

#[allow(unused, unreachable_code)]
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
