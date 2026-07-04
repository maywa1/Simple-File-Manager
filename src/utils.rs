use std::fs::OpenOptions;
use std::io::Write;

#[allow(dead_code)]
pub fn dbg_log(msg: &str) {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open("debug.log")
        .unwrap();

    writeln!(file, "{}", msg).unwrap();
}
