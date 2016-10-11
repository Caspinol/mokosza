// Crates
extern crate nix;
#[macro_use]
extern crate chan;
extern crate chan_signal;

// Module imports
mod domain;
mod db;
mod crawler;
mod log;

// Standard libs
//use std::str;
use std::thread;

//Extern libs
use nix::unistd::{ fork, chdir, ForkResult };
use nix::sys::stat::{ umask, Mode };
use chan_signal::{Signal, notify};

// Project libs
use log::{log_info, log_err, log_warn};

//use domain::Domain;


fn main() {
    
    //let _ = crawler::crawl_domain("http://www.example.com");
    
    // Daemonize the process
    match fork() {
        Err(err) => {
            println!("Error while  forking process: {}", err);
            return ::std::process::exit(1);
        },
        Ok(ForkResult::Parent {child} ) => {
            println!("Child PID: {0}\nTo shut it down use: \"kill -9 {0}\"", child);
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

            //let mut threads = Vec::new();
            //log_info("Spawning threads");
            //for _ in 0..4 {
                let _ = thread::spawn(move || {
                    run();
                });
            //    threads.push(child);
           // }
            
            //for t in threads {
            //    t.join().unwrap();
            //}
            
            // Register a signal handler
            let signal = notify(&[Signal::INT]);
            // Handle signal when received
            chan_select! {
                signal.recv() -> sig => {
                    log_warn(&format!("Received INT signal: {:?}", sig.unwrap()));
                },
            }
        },
    }
}

fn run() {
    let db = db::DB::new("postgresql://mokosza:mokoszamokosza@\
                          catdamnit.chs4hglw5opg.eu-west-1.rds.amazonaws.com:5432/mokosza");
    log_info("Succesfully connected to database...");
    if let Ok(conn) = db {
        loop {
            log_info("Fetching new domain");
            match conn.next_domain() {
                None => break,
                Some(url) => {
                    log_info(&format!("Gonna crawl [{}]", url));
                    let crawl_result = crawler::crawl_domain(&url, |page, other| {
                        println!("{}",*page);
                        conn.store_domains(&other);
                    });
                    // Regardless of result mark domain as processed
                    
                    match crawl_result {
                        Ok(_) => {
                            let _ = conn.domain_done(&url);
                        },
                        Err(err) => {
                            log_err(&format!("Failed to crawl {}. error: {}",
                                             url, err));
                            let _ = conn.domain_err(&url);
                        }
                    }
                }
            }
        }
    }
}
