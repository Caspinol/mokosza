// Crates
extern crate regex;

// Modules

// Standard lib
use self::regex::Regex;

// Project libs
use domain::Domain;

pub struct DomainURL {
    re_full_url: Regex,
    re_domain: Regex,
    re_is_url: Regex,
}

impl DomainURL {

    pub fn new() -> DomainURL {
        DomainURL {
            // This patter should grap entire URL along with its path
            re_full_url: Regex::new(r###"(https?://[^'"?&><]*)"###).unwrap(),
            
            // Extract only the domain part.
            re_domain: Regex::new(r###"(https?://[^/'"?&]*)"###).unwrap(),
            
            // Validate if string is a valid URL i.e http://www.example.com
            re_is_url: Regex::new(r###"^https?://(?:www\.)?[a-z0-9]+[^/'"?&][a-z\.]{2,5}/{0,1}$"###).unwrap(),
        }
    }
    fn get_domain_part<'a>(&self, url: &'a str) -> Option<String> {
        match self.re_domain.captures(url) {
            Some(pattern) => Some(pattern[1].to_owned()),
            _ => None,
        }
    }

    fn get_url<'a>(&self, s: &'a str) -> Option<String> {
        match self.re_full_url.captures(s) {
            Some(pattern) => Some(pattern[1].to_owned()),
            _ => None,
        }
    }

    pub fn find_all_url(&self, page: &str, dom: &mut Domain, other: &mut Vec<String>) {
        println!("Parsing page for links");
        for capture in self.re_full_url.captures_iter(page) {
            if let Some(cap) = capture.at(1) {
                /* Skip all urls ending with
                .jpg, js, .pdf, .css etc.
                there must be a better way to do this
                 */
                if cap.ends_with(".jpg")
                    | cap.ends_with(".js")
                    | cap.ends_with(".css")
                    | cap.ends_with(".png") {
                        println!("URL: {} ends with invalid extension...skipping", cap);
                    } else {
                        if let Some(dpart) = self.get_domain_part(cap) {
                            /* 
                            Also check if URL belongs to the domain we are crawling right now.
                             */
                            if dpart == dom.domain {
                                println!("Adding new URL: \"{}\"", cap);
                                dom.add_to_visit(cap);
                            } else {
                                other.push(cap.to_string());
                            }
                        }
                            
                    }
            }   
        }
    }

    pub fn is_URL(&self, s: &str) -> bool {
        self.re_is_url.is_match(s)
    }
}

#[cfg(test)]
#[test]
fn test_get_domain() {

    let dom = DomainURL::new();
    
    let astring = "<a href='http://www.example.com/dddddd/gggggg?param=3455' >link</a>";
    assert_eq!(dom.get_domain_part(astring), Some("http://www.example.com".to_string()));

    let astring = "<a href='' >link</a>";
    assert_eq!(dom.get_domain_part(astring), None);
}

#[test]
fn test_get_url() {

    let dom = DomainURL::new();
    
    let astring = "<a href='http://www.example.com/dddddd/gggggg?param=3455' >link</a>";
    assert_eq!(dom.get_url(astring),
               Some("http://www.example.com/dddddd/gggggg".to_string()));

    let astring = "<a href='http://www.t.b/dddddd/g(g)gggg?param=3455' >link</a>";
    assert_eq!(dom.get_url(astring),
               Some("http://www.t.b/dddddd/g(g)gggg".to_string()));

    let astring = "<a href='' >link</a>";
    assert_eq!(dom.get_domain_part(astring), None);
}

#[test]
fn test_is_domain() {
    let dom = DomainURL::new();

    let astring = "<a href='http://www.example.com/dddddd/gggggg?param=3455' >link</a>";
    assert_eq!(dom.is_URL(astring), false);

    let astring = "http://www.example.com/dddddd/gggggg?param=3";
    assert_eq!(dom.is_URL(astring), false);

    let astring = "http://www.example.com/";
    assert_eq!(dom.is_URL(astring), true);

    let astring = "http://example.com/";
    assert_eq!(dom.is_URL(astring), true);

    let astring = "https://example.co.uk/";
    assert_eq!(dom.is_URL(astring), true);
}
