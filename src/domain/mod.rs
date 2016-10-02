// Crates
extern crate curl;

// extern libs
use self::curl::easy::Easy;

// Modules
pub mod domain_url;

// stdlib
use std::fmt;
use std::ops::Deref;
use std::error::{self, Error};

/* 
Page struct definition
 */
pub struct Page {
    pub page: String,
}

impl fmt::Display for Page {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.page)
    }
}

impl Deref for Page {
    type Target = String;

    fn deref(&self) -> &String {
        &self.page
    }
}

pub struct Domain<'a> {
    pub domain: &'a str,
    robots: Vec<String>,
    pub paths_visited: Vec<String>,
    pub paths_to_visit: Vec<String>,
}

impl<'a> PartialEq for Domain<'a> {
    fn eq(&self, other: &Domain) -> bool {
        self.domain == other.domain
    }
}

impl<'a> Domain<'a> {
    pub fn new(domain: &str) -> Domain {
        let mut dom = Domain {
            domain: domain,
            robots: Vec::new(),
            paths_visited: Vec::new(),
            paths_to_visit: Vec::new(),
        };

        let _ = dom.check_robots()
            .map_err(|err| println!("Error: {}", err.description()))
            .and_then(|()| {
                println!("Succesfully fetched robots.txt from: {} ", domain);
                println!("We have {} disallowed paths", dom.robots.len());
                Ok(())
            });
        dom
    }

    pub fn get_webpage(&self, url: &str) -> Result<Page, DomainError> {
        if self.is_url_in_robots(url) {
            println!("{} skipped. Forbidden by robots.txt", url);
            return Err(DomainError::SkippedURL);
        }

        match self.page_curl(url) {
            Ok(page) => Ok(page),
            Err(err) => Err(err),
        }
    }

    pub fn add_visited(&mut self, url: &str) -> usize {
        self.paths_visited.push(url.to_owned());
        self.paths_visited.len()
    }

    pub fn add_to_visit(&mut self, url: &str) -> usize {
        // We dont want any duplicates here
        let s = url.to_owned();
        if !self.paths_to_visit.contains(&s) {
            self.paths_to_visit.push(s);
        }
        self.paths_to_visit.len()
    }

    pub fn get_next_to_visit(&mut self) -> Option<String> {
        self.paths_to_visit.pop()
    }

    fn is_url_in_robots(&self, url: &str) -> bool {
        let isit = &self.robots.clone().into_iter()
            .filter(|path| url.contains(path))
            .collect::<String>();
        
        return isit.len() > 0;
    }

    fn get_robots_url(&self) -> String {
        // Just return domain and robots path stitched together
        let mut robots_url = String::from(self.domain.clone());
        robots_url.push_str("/robots.txt");
        robots_url
    }

    fn page_curl(&self, url: &str) -> Result<Page, DomainError> {
        let mut easy = Easy::new();
        let mut dst = Vec::new();
        {
            try!(easy.url(url).map_err(DomainError::FetchError));
            
            let mut transfer = easy.transfer();
            try!(transfer.write_function(|data| {
                dst.extend_from_slice(data);
                Ok(data.len())
            }).map_err(DomainError::FetchError));
            
            try!(transfer.perform().map_err(DomainError::FetchError));
        }
        /* Its unsafe because sometimes the captured data from webpage 
        is not correctly utf-8 formrmated
        I can live with a odd letter malformed here and there
         */
        unsafe {
            return Ok(Page {
                page: String::from_utf8_unchecked(dst)
            })
        }
    }

    /*
    Tries to fetch and parse the robots file
    Upon success the robots vector will be populated with forbidded 
    paths. 
     */
    fn check_robots(&mut self) -> Result<(), DomainError>{

        let robots_url = self.get_robots_url();
        println!("Fetching robots.txt from {}", robots_url);
        match self.page_curl(&robots_url) {
            Ok(res) => {
                // Extract forbidden URLs from robots
                for robots_line in res.page.lines() {
                    let line: Vec<&str> = robots_line.split_terminator(':').collect();
                    if line.len() == 2 && line[0].contains("Disallow") {
                        println!("Disallow: {}", line[1]);
                        self.robots.push(String::from(line[1]));
                    } else {
                        return Err(DomainError::RobotsError);
                    }
                }
                Ok(())
            },
            Err(err) => Err(err) ,
        }
    }   
}

/* 
Fetching error stuct definition
 */

#[derive(Debug)]
pub enum DomainError {
    SkippedURL,
    RobotsError,
    InvalidURL,
    FetchError(curl::Error),
}

impl fmt::Display for DomainError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            DomainError::SkippedURL => write!(f, "URL skipped"),
            DomainError::RobotsError => write!(f, "robot.txt missing or malformed"),
            DomainError::InvalidURL => write!(f, "Malformed url"),
            DomainError::FetchError(ref err) => write!(f, "Failed to grab domain: {}", err),
        }
    }
}

impl error::Error for DomainError {
    fn description(&self) -> &str {
        match *self {
            DomainError::SkippedURL => "URL was skipped due to robots.txt policy.",
            DomainError::RobotsError => "robots.txt file missing or malformed.",
            DomainError::InvalidURL => "Specified URL is invalid/malformed",
            DomainError::FetchError(ref err) => err.description(),
        }
    }

    fn cause(&self) -> Option<&error::Error> {
        match *self {
            DomainError::SkippedURL => None,
            DomainError::RobotsError => None,
            DomainError::InvalidURL => None,
            DomainError::FetchError(ref err) => err.cause(),
        }
    }
}

impl From<curl::Error> for DomainError {
    fn from(err: curl::Error) -> DomainError {
        DomainError::FetchError(err)
    }
}

#[cfg(test)]
#[test]
fn test_is_in_robots() {
    let mut dom = Domain::new("http://exampdssdsdle.com");
    dom.robots.push("/aaa".to_string());
    dom.robots.push("/bbb".to_string());
    dom.robots.push("/ccc".to_string());

    assert!(dom.is_url_in_robots("http://example.com/fff/aaa"));
    assert!(!dom.is_url_in_robots("http://example.com/fff/ggg"));
    assert!(dom.is_url_in_robots("http://example.com/fff/bbb/zzz"));
}


#[test]
fn test_add_to_visit() {

    // Test if list of to be visited URL takes in duplicates
    let mut dom = Domain::new("");
    dom.add_to_visit("http://example.com/fff");
    assert_eq!(dom.paths_to_visit.len(), 1);

    dom.add_to_visit("http://example.com/fff");
    assert_eq!(dom.paths_to_visit.len(), 1);

    dom.add_to_visit("http://example.com/ggg");
    assert_eq!(dom.paths_to_visit.len(), 2);
}


#[test]
fn test_get_webpage() {

    let mut dom = Domain::new("http://example.com");
    let curl_result = dom.get_webpage(dom.domain);

    assert!(curl_result.is_ok());
    dom.robots.push("domains".to_owned());
    let curl_result = dom.get_webpage("http://www.iana.org/domains/reserved");
    assert!(curl_result.is_err());
    //assert_eq!(curl_result.unwrap(), DomainError::SkippedURL);
    
    let curl_result = dom.get_webpage("");
    assert!(curl_result.is_err());
}
