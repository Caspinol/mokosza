// Crates
#[macro_use]
extern crate chan;
extern crate chan_signal;
extern crate nix;

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
use chan_signal::{ notify, Signal };

// Project libs
use log::{log_info, log_err, log_warn};


const THREADS: usize = 5;

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

            // Register a signal handler
            let signal = notify(&[Signal::INT, Signal::KILL]);
            let (tx, rx) = chan::sync(THREADS);


            // Create the crawler threads
            log_info("Spawning threads");
            for _ in 0..THREADS {
                let rx = rx.clone();
                thread::spawn(move || {
                    run(rx);
                });
            }

            // Create URL feeder thread
            thread::spawn(move || {
                let db = db::DB::new("postgresql://mokosza:mokoszamokosza@\
                                      catdamnit.chs4hglw5opg.eu-west-1.rds.amazonaws.com\
                                      :5432/mokosza");
                if let Ok(conn) = db {
                    while let Some(url) = conn.next_domain() {

                        tx.send(url)
                    }
                }
            });
            
            // Handle signal when received
            chan_select! {
                signal.recv() -> sig => {
                    log_warn(&format!("Received signal: {:?}", sig.unwrap()));
                },
            }
        },
    }
}

fn run(rx: chan::Receiver<String>) {
    let db = db::DB::new("postgresql://mokosza:mokoszamokosza@\
                          catdamnit.chs4hglw5opg.eu-west-1.rds.amazonaws.com:5432/mokosza");
    log_info("Succesfully connected to database...");
    if let Ok(conn) = db {
        loop {
            log_info("Fetching new domain");
            match rx.recv() {
                None => break,
                Some(url) => {
                    log_info(&format!("Gonna crawl [{}]", url));
                    let crawl_result = crawler::crawl_domain(&url, |page, other| {
                        // Here we can do something with the page
                        // i.e. store it, send it etc.
                        // Now jus print it
                        println!("{}",*page);
                        conn.store_domains(&other);
                    });
                    
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
