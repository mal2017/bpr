#![forbid(unsafe_code)]
extern crate clap;
extern crate mbulib;
extern crate rust_htslib;
#[macro_use]
extern crate quick_error;
extern crate itertools;
extern crate rand;

use clap::{App, Arg, ArgGroup};
use regex::Regex;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use itertools::Itertools;

fn main() {
    let matches = App::new("bpr")
        .version("0.1.0")
        .author("Matt Lawlor <matt.a.lawlor@gmail.com>")
        .about("Create psuedoreplicates from bam files.")
        .arg(
            Arg::with_name("ibam")
                .help("bam file holding reads")
                .required(true)
                .index(1)
                .validator(bam_seems_ok),
        )
        .arg(
            Arg::with_name("basename")
                .help("basename for output bams")
                .required(true)
                .index(2)
                .validator(dir_exists),
        )
        .arg(Arg::with_name("single_end").long("single-end"))
        .arg(
            Arg::with_name("threads")
                .help("threads to use")
                .short("p")
                .long("threads")
                .takes_value(true),
        )
        .get_matches();

    let bam_file: &str = matches.value_of("ibam").unwrap();
    let basename: &str = matches.value_of("basename").unwrap();
    let threads: usize = matches.value_of("threads").unwrap_or("1").parse().unwrap();
    let single_end: bool = matches.is_present("single_end");

    run(bam_file, basename, threads);
}

// actually run
fn run(b: &str, o: &str, p: usize) {
    use mbulib::bam::header::*;
    use mbulib::bam::sort::*;
    use rust_htslib::bam::*;
    use rand::{thread_rng, Rng};
    use rand::distributions::Uniform;

    let bampath = Path::new(b);
    let opaths = make_output_names(o).unwrap();

    let mut bam = Reader::from_path(bampath).unwrap();
    bam.set_threads(p);

    // Random number generation.
    let mut rng = thread_rng();

    // Distribution of possible outbams.
    let dist = Uniform::new(0usize, 3);

    let header = mbulib::bam::header::edit_hdr_srt_tag(bam.header(), "unknown");

    let mut obams: Vec<Writer> = opaths.into_iter()
                      .map(|a| Writer::from_path(a, &header).unwrap())
                      .collect();

    let rec_it = bam.records()
       .map(|a| a.unwrap())
       .group_by(|a| String::from_utf8(a.qname().to_vec()).unwrap());

     let mut bucket;

     for (x,y) in rec_it.into_iter() {
         bucket = rng.sample(dist);

         y.into_iter()
          .map(|a| obams[bucket].write(&a).unwrap())
          .for_each(drop);
     }


}

// TODO this function is horrible
fn make_output_names(a: &str) -> Result<Vec<String>, FilenameGenerationError> {
    let basepath = Path::new(a);

    let dir = basepath.parent().unwrap();
    let basename = match basepath.file_stem() {
        Some(b) => b.to_str().unwrap().to_owned(),
        None => return Err(FilenameGenerationError::NoneSuchFileStem),
    };

    let exts = vec!["rep0.bam", "rep1.bam", "rep2.bam"];
    let filenames: Vec<String> = std::iter::repeat(basename)
        .take(3)
        .map(|a| Path::new(&a).to_owned())
        .zip(exts.iter())
        .map(|(a, b)| a.with_extension(b))
        .map(|a| dir.join(a))
        .map(|a| a.to_str().unwrap().to_owned())
        .collect();
    Ok(filenames)
}

quick_error! {
    /// Error in creation of file names.
    #[derive(Debug, Clone)]
    pub enum FilenameGenerationError {
        NoneSuchFileStem {
            description("couldn't extract the basename")
        }
    }
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
