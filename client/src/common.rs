use std::time::{SystemTime, UNIX_EPOCH};

pub fn get_current_unix() -> u128 {
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Cannot get unix timestamp");
    since_the_epoch.as_millis()
}
