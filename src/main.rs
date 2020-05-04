#![allow(dead_code)]

extern crate chrono;
extern crate colored;
extern crate users;

mod ls;

use std::env;

fn main()
{
    std::process::exit(
        {
            let args: Vec<String> = env::args().collect(); 
            let mut utility = ls::Utility::new(args);
        
            match utility.execute()
            {
                Ok(_) => {0},
                Err(e) => {eprintln!("custom_ls error: `{}`", e); 1}
            }
        });
}