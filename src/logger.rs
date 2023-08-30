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

use crate::errors::RemoveFilesError;
use crate::{
    config::dev_mode,
    prelude::{EnhancedExpect, EnhancedUnwrap},
};

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
            tracing_appender::rolling::daily(log_path(None, None), format!("{}.log", bin_name));
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
pub fn cleanup_files_immediately(directory: &str, days: i64) -> Result<(), RemoveFilesError> {
    let paths = fs::read_dir(directory)
        .map_err(|e| RemoveFilesError {
            details: format!("An error occurred in reading the directory and the cleanup file failed: {}", e),
        })?;

    for path in paths.flatten() {
        let path_buf = path.path();
        let modified = fs::metadata(&path_buf)
            .and_then(|metadata| metadata.modified())
            .map_err(|e| RemoveFilesError {
                details: format!("An error occurred in getting file modified time and the cleanup file failed: {}", e),
            })?;
        if (Utc::now() - DateTime::from(modified)).num_days() > days {
            fs::remove_file(&path_buf)
                .map_err(|e| RemoveFilesError {
                    details: format!(
                        "delete file failed, path: {:?}, error: {}",
                        path_buf, e
                    ),
                })?;
        }
    }
    Ok(())
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
/// schedule_cleanup_log_files(None, "/opt/logs/apps/", 30);
/// ```
pub fn schedule_cleanup_log_files(
    directory: &str,
    days: i64,
    cron_expression: Option<String>,
) -> Result<(), RemoveFilesError> {
    // cron_expression sample: 0 15 6,8,10 * Mar,Jun Fri 2017
    // Run at second 0 of the 15th minute of the 6th, 8th, and 10th hour
    // of any day in March and June that is a Friday of the year 2017.
    // more information about `cron_expression` see  https://docs.rs/job_scheduler/latest/job_scheduler/
    let cron_expression = {
        if cron_expression.is_none() {
            "0 0 0 * * * *".to_owned()
        } else {
            cron_expression.unwp()
        }
    };
    let mut sched = JobScheduler::new();
    sched.add(Job::new(cron_expression.parse().unwp(), || {
        debug!("start clean log files");
        cleanup_files_immediately(directory, days).ex("cleanup_files_immediately should work");
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

pub fn log_path(log_path: Option<&str>, env_log_path_key: Option<&str>) -> PathBuf {
    if dev_mode() {
        let dir = env::temp_dir();
        debug!(
            "log will be saved to temporary directory: {}",
            dir.display()
        );
        return dir;
    }

    // log path from param is first if it have been set
    if log_path.is_some() {
        return PathBuf::from(log_path.unwp().trim());
    }

    // default log path
    let log_path = r"/opt/logs/apps/";
    if env_log_path_key.is_some() {
        let env_log_path = env::var(env_log_path_key.unwp());
        match env_log_path {
            Ok(env_log_path) => return PathBuf::from(env_log_path),
            Err(_) => warn!("{} is not set, use default log path: {}", env_log_path_key.unwp(), log_path),
        }
    };
    PathBuf::from(log_path)
}

#[cfg(test)]
mod logger_test {
    use std::env;
    use crate::logger::{cleanup_files_immediately, schedule_cleanup_log_files, log_path};
    use crate::prelude::EnhancedUnwrap;

    #[test]
    fn test_delete_log_files() {
        if let Err(e) = cleanup_files_immediately("/opt/logs/apps/", 30) {
            panic!("test_delete_log_files failed, error: {}", e);
        }
    }

    #[test]
    fn test_schedule_cleanup_log_files() {
        if let Err(e) = schedule_cleanup_log_files("/opt/logs/apps/", 30, None) {
            panic!("test_schedule_cleanup_log_files failed, error: {}", e);
        }
    }

    #[test]
    fn test_get_log_path() {
        let log_path_default = log_path(None, None);
        assert_eq!(log_path_default.to_str().unwp(), "/opt/logs/apps/");

        let log_path_from_param = log_path(Some("/a/b/c"), None);
        assert_eq!(log_path_from_param.to_str().unwp(), "/a/b/c");

        env::set_var("LOG_PATH", "/xx/xx");
        let log_path_from_env = log_path(None, Some("LOG_PATH"));
        assert_eq!(log_path_from_env.to_str().unwp(), "/xx/xx");
    }
}
