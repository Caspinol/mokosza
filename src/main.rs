// Crates
extern crate nix;

// Module imports
mod domain;
mod db;
mod crawler;
mod log;

// Standard libs
//use std::env;
//use std::str;

//Extern libs
use nix::unistd::{ fork, chdir, ForkResult };
use nix::sys::stat::{ umask, Mode };


// Project libs
use log::{log_info, log_err, log_warn};

//use domain::Domain;


fn main() {
    
    //let _ = crawler::crawl_domain("http://www.example.com");
    /*
    let mut threads = Vec::new();
    for i in 0..3 {
    let child = thread::spawn(move || {
    println!("{} Hail Satan!", i);
});
    threads.push(child);
}
    
    for t in threads {
    t.join().unwrap();
}
     */
    
    match fork() {
        Err(err) => {
            println!("Error while  forking process: {}", err);
            return ::std::process::exit(1);
        },
        Ok(ForkResult::Parent {child} ) => {
            println!("Child runing with PID: {}", child);
            log_info("Parent exiting...");
            ::std::process::exit(0);
        },
        Ok(ForkResult::Child) => {
            // Prepare deamon environment
            umask(Mode::from_bits(0).unwrap());
            // TODO:
            // We need to change this to some more meaningful path
            // possibly home_dir of mokosza user
            if let Err(_) = chdir("/") {
                log_err("Failed to set working directory. Exiting");
                ::std::process::exit(1);
            }

            
            loop {
                log_info("deamonizing success");
            }
        },
    }
    
}
