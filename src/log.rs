extern crate time;

use std::env::home_dir;
use std::fs;
use std::path::PathBuf;
use std::io::{ Write, Error };



pub fn log_info(message: &str) {
    let _ = _log("INFO", message);
}

pub fn log_err(message: &str) {
    let _ = _log("ERROR", message);
}

pub fn log_warn(message: &str) {
    let _ = _log("WARNING", message);
}

fn _log(level: &str, message: &str) -> Result<(), Error> {
    let mut logdir = PathBuf::new();
    
    match home_dir() {
        Some(p) => logdir.push(p),
        None => logdir.push("home/ubuntu")
    };

    logdir.push("mokosza.log");
    
    let mut file = try! (fs::OpenOptions::new()
                         .create(true)
                         .write(true)
                         .append(true)
                         .open(logdir.as_path()));
    let t = time::now();
    let msg = format!("{} - [{}]: {}", t.ctime(), level, message);
    try! (file.write(msg.as_bytes()));
    try! (file.write(b"\n"));
    Ok(())
}
