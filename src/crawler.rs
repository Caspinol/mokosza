// Crates

// Modules

// Standard libs
use std::error::Error;

// Project libs
use domain::{ Domain };
use domain::domain_url::DomainURL;
use domain::domain_error::DomainError;
use db::DB;

pub fn crawl_domain(domain_url: &str) -> Result<(), DomainError> {

    let db = try!(DB::new("postgresql://mokosza:mokoszamokosza@\
                           catdamnit.chs4hglw5opg.eu-west-1.rds.amazonaws.com:5432/mokosza")
                  .map_err(DomainError::DBError));
    
    let durl = DomainURL::new();

    // Lets validate the url syntax
    // We want to start from the root URL i.e. www.example.com
    if durl.is_url(domain_url) == false {
        return Err(DomainError::InvalidURL);
    }

    let mut domain = Domain::new(domain_url);

    loop {
        match domain.paths_to_visit.pop() {
            Some(url) => {
                println!("Downloading content from {}", url);
                let page_result = domain.get_webpage(&url);
                let mut other_domains: Vec<String> = Vec::new();
                match page_result {
                    Ok(p) => {
                        println!("Downloading {} succesful", url);
                        durl.find_all_url(&*p, &mut domain, &mut other_domains);
                        println!("Found {} links for current domain and {} pointing to different domain", domain.paths_to_visit.len(), other_domains.len());
                        // Mark url as processed
                        domain.add_visited(&url);
                        /*
                        Write all links that point out of domain to db
                        */
                        db.store_domains(&other_domains);
                        
                        // Here page can be send to other system for keywork analysys
                        // analyze(*p)    
                    },
                    Err(err) => {
                        println!("Error: {}", err.description());
                        return Err(err);
                    },
                }
            },
            None => {
                println!("No more links to crawl on {}", domain.domain);
                return Ok(())
            },
        }
    }
}


#[test]
fn test_crawl_domain() {
    let www = "http://example.com";
    let res = crawl_domain(www);
    assert!(res.is_ok());

    let www = "http://www.example.com";
    let res = crawl_domain(www);
    assert!(res.is_ok());

    let www = "http://excom";
    let res = crawl_domain(www);
    assert!(res.is_err());
}
