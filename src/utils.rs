use std::time::{SystemTime, UNIX_EPOCH};

// 获取当前时间戳
pub fn current_timestamp() -> u64 {
    let start = SystemTime::now();
    let since_epoch = start.duration_since(UNIX_EPOCH).expect("时间戳生成失败！");
    since_epoch.as_secs()
}
