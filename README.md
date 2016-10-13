[![Build Status](https://travis-ci.org/Caspinol/mokosza.svg?branch=master)](https://travis-ci.org/Caspinol/mokosza)

# mokosza

## A simple webcrawler written in Rust.

It tries to be polite and respect the robots.txt. It tries to (but needs more work)
to not overload the domain its crawling. It can send the webpage content to external system
for processing/analisys.

The idea is that multiple instances of it can be easilly and quickly deployed on AWS.