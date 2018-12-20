#![forbid(unsafe_code)]
extern crate clap;
extern crate rust_htslib;

use regex::Regex;
use std::path::Path;
use clap::{App, Arg, ArgGroup};

fn main() {
    let matches = App::new("bpr")
                          .version("0.1.0")
                          .author("Matt Lawlor <matt.a.lawlor@gmail.com>")
                          .about("Create psuedoreplicates from bam files.")
                          .arg(Arg::with_name("ibam")
                               .help("bam file holding reads")
                               .required(true)
                               .index(1)
						   	   .validator(bam_seems_ok))
                          .arg(Arg::with_name("basename")
                               .help("basename for output bams")
                               .required(true)
                               .index(2)
							   .validator(dir_exists))
					      .arg(Arg::with_name("single_end")
					  		    .long("single-end"))
                          .arg(Arg::with_name("threads")
                          	   .help("threads to use")
                          	   .short("p")
                          	   .long("threads")
                          	   .takes_value(true))
                          .get_matches();

    let bam_file: &str = matches.value_of("ibam").unwrap();
    let basename: &str = matches.value_of("basename").unwrap();
    let threads: usize = matches.value_of("threads").unwrap_or("1").parse().unwrap();
    let single_end: bool = matches.is_present("single_end");

    run();

}

// Checks that the output bam path is writeable.
fn dir_exists(a: String) -> Result<(), String> {
    let p = Path::new(&a);

    match p.parent().unwrap().exists() {
        true => Ok(()),
        false => Err(String::from(
            "Your intended result directory doesn't exist yet.",
        )),
    }
}

// Checks that the input bam seems to exist.
fn bam_seems_ok(a: String) -> Result<(), String> {
    let p = Path::new(&a);

    match p.is_file() {
        true => match p.extension().unwrap().to_str().unwrap() == "bam" {
            true => Ok(()),
            false => Err(String::from("Input bam does not appear to be a bam.")),
        },
        false => Err(String::from("Input bam doesn't seem to exist.")),
    }
}


// actually run
fn run() {
    println!("running");
}
