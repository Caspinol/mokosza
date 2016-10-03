// Crates

// Modules

// Standard libs
use std::error::Error;

// Project libs
use domain::Domain;
use domain::domain_url::DomainURL;
use domain::domain_error::DomainError;

pub fn crawl_domain(url: &str) -> Result<(), DomainError> {

    let durl = DomainURL::new();

    // Lets validate the url syntax
    // We want to start from the root URL i.e. www.example.com
    if durl.is_URL(url) == false {
        return Err(DomainError::InvalidURL);
    }

    let mut domain = Domain::new(url);

    println!("Downloading content from {}", url);
    let page_result = domain.get_webpage(url);

    let mut other_domains: Vec<String> = Vec::new();
    match page_result {
        Ok(p) => {
            println!("Downloading {} succesful", url);
            durl.find_all_url(&*p, &mut domain, &mut other_domains);
            println!("Found {} links in total. {} point to different domain",
                     domain.paths_to_visit.len(), other_domains.len());
            Ok(())
        },
        Err(err) => {
            println!("Error: {}", err.description());
            Err(err)
        },
    }
}


#[test]
fn test_crawl_domain() {
    let www = "http://example.com";

    let res = crawl_domain(www);

    assert!(res.is_ok());
}
