// Module imports
mod domain;
mod db;
mod crawler;

// Standard libs
//use std::env;
//use std::str;

// Project libs
//use domain::Domain;

fn main() {

    // Capture all strings passed in as arguments
    //let mut query: Vec<String> = env::args().collect();
    //assert!(!(query.len() < 2));

    let _ = crawler::crawl_domain("http://www.example.com");
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
}
