/**
 * 日志模块
 * 提供带文件名和行号的日志宏
 */

use tracing_subscriber::{FmtSubscriber, EnvFilter};

// 初始化日志系统
pub fn init_logger() {
    // 创建格式化订阅者
    let subscriber = FmtSubscriber::builder()
        .with_env_filter(EnvFilter::from_default_env())
        .with_file(true)        // 显示文件名
        .with_line_number(true) // 显示行号
        .with_target(false)     // 隐藏目标（模块路径）
        .finish();

    // 设置全局默认订阅者
    tracing::subscriber::set_global_default(subscriber)
        .expect("Failed to set global default subscriber");
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