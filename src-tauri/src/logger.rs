/**
 * 日志模块
 * 提供带文件名和行号的日志宏，并支持文件记录
 */

use tracing_subscriber::{FmtSubscriber, EnvFilter, prelude::*};
use std::path::Path;
use tracing_appender::non_blocking::WorkerGuard;
use crate::config::LogConfig;

// 初始化日志系统，返回 guard 以保持非阻塞日志写入器运行
pub fn init_logger(data_dir: &Path, config: &LogConfig) -> Option<WorkerGuard> {
    if !config.enabled {
        // 如果未启用文件日志，仅初始化控制台日志（或者干脆不初始化 tracing，
        // 但为了开发方便，我们至少保留基本的控制台输出）
        let subscriber = FmtSubscriber::builder()
            .with_env_filter(EnvFilter::new(&config.level))
            .with_file(true)
            .with_line_number(true)
            .with_target(false)
            .finish();
        
        tracing::subscriber::set_global_default(subscriber).ok();
        return None;
    }

    let log_dir = data_dir.join("logs");
    
    // 创建每日滚动的日志记录器
    // 命名格式：YYYY-MM-DD.log (tracing-appender 默认行为是 prefix.YYYY-MM-DD)
    // 用户要求以年月日时间命名（20251228），我们可以通过自定义文件名前缀和日期格式来实现
    let file_appender = tracing_appender::rolling::Builder::new()
        .rotation(tracing_appender::rolling::Rotation::DAILY)
        .max_log_files(config.max_days as usize)
        .filename_prefix("") // 无前缀
        .filename_suffix("log")
        .build(log_dir)
        .expect("Failed to create log appender");

    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

    let filter = EnvFilter::new(&config.level);

    // 文件输出层
    let file_layer = tracing_subscriber::fmt::layer()
        .with_writer(non_blocking)
        .with_ansi(false) // 文件日志不需要 ANSI 颜色
        .with_file(true)
        .with_line_number(true)
        .with_target(false);

    // 控制台输出层
    let console_layer = if config.console_output {
        Some(tracing_subscriber::fmt::layer()
            .with_file(true)
            .with_line_number(true)
            .with_target(false))
    } else {
        None
    };

    tracing_subscriber::registry()
        .with(filter)
        .with(file_layer)
        .with(console_layer)
        .init();

    Some(guard)
}

// 自定义日志宏，支持文件名和行号
#[macro_export]
macro_rules! log_debug {
    ($($arg:tt)*) => {
        tracing::debug!($($arg)*)
    };
}

#[macro_export]
macro_rules! log_info {
    ($($arg:tt)*) => {
        tracing::info!($($arg)*)
    };
}

#[macro_export]
macro_rules! log_warn {
    ($($arg:tt)*) => {
        tracing::warn!($($arg)*)
    };
}

#[macro_export]
macro_rules! log_error {
    ($($arg:tt)*) => {
        tracing::error!($($arg)*)
    };
}