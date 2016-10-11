// Crates

// Modules

// Standard libs
use std::error::Error;

// Project libs
use domain::{ Domain, Page };
use domain::domain_url::DomainURL;
use domain::domain_error::DomainError;
use log::*;

pub fn crawl_domain<F>(domain_url: &str, handle_page: F)
                       -> Result<(), DomainError>
    where F: Fn(Page, Vec<String>)
{
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
                log_info(&format!("Downloading content from {}", url));
                let page_result = domain.get_webpage(&url);
                match page_result {
                    Ok(p) => {
                        let mut other_domains: Vec<String> = Vec::new();
                        log_info(&format!("Downloading {} succesful", url));
                        durl.find_all_url(&*p, &mut domain, &mut other_domains);
                        log_info(&format!("Found {} links for current domain and {} \
                                  pointing to different domain",
                                 domain.paths_to_visit.len(), other_domains.len()));
                        // Mark url as visited
                        domain.add_visited(&url);
                        
                        // Here page can be send to other system for keywork analysys
                        handle_page(p, other_domains)    
                    },
                    Err(err) => {
                        log_err(&format!("Error: {}", err.description()));
                        return Err(err);
                    },
                }
            },
            None => {
                log_info(&format!("No more links to crawl on {}", domain.domain));
                return Ok(());
            },
        }
        // Lets not overload their webserver with to frequent queries
        ::std::thread::sleep(::std::time::Duration::from_millis(5000));
    }
}
