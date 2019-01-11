#![forbid(unsafe_code)]
extern crate clap;
extern crate rust_htslib;
#[macro_use]
extern crate quick_error;
extern crate itertools;
extern crate rand;
extern crate rand_xorshift;

use clap::{App, Arg, ArgGroup};
use regex::Regex;
use std::ffi::OsStr;
use std::path::{Path, PathBuf};
use itertools::Itertools;
use rust_htslib::bam::*;
use rand::{thread_rng, Rng, SeedableRng};
use rand::distributions::Uniform;


fn main() {
    let matches = App::new("bpr")
        .version("0.3.0a")
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
        .arg(
            Arg::with_name("n_reps")
                .help("number of replicates to produce")
                .short("n")
                .long("n-pseudoreps")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("seed")
                .help("seed for random assignment to pseudoreplicate")
                .required(true)
                .short("s")
                .long("seed")
                .takes_value(true)
        )
        .arg(
            Arg::with_name("threads")
                .help("threads to use")
                .short("p")
                .long("threads")
                .takes_value(true),
        )
        .get_matches();

    let bam_file: &str  = matches.value_of("ibam").unwrap();
    let basename: &str  = matches.value_of("basename").unwrap();
    let threads:  usize = matches.value_of("threads").unwrap_or("1").parse().unwrap();
    let seed:     &str  = matches.value_of("seed").unwrap();
    let n_reps:   usize  = matches.value_of("n_reps").unwrap_or("1").parse().unwrap();

    run(bam_file, basename, threads, seed, n_reps);
}

// actually run
fn run(b: &str, o: &str, p: usize, seed: &str, n: usize) {

    let mut seedu8: [u8; 16] = make_seed_from_str(seed);

    let bampath = Path::new(b);
    let opaths = make_output_names(o, n).unwrap();

    let mut bam = Reader::from_path(bampath).unwrap();
    bam.set_threads(p);

    // Random number generation.
    let mut rng = rand_xorshift::XorShiftRng::from_seed(seedu8);

    // Distribution of possible outbams.
    let dist = Uniform::new(0usize, n);


    let header = Header::from_template(bam.header());

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


fn make_seed_from_str(seed: &str) -> [u8; 16] {
    let bytes: Vec<u8> = seed.as_bytes()
                             .to_owned()
                             .into_iter()
                             .cycle()
                             .take(16).collect();

    let mut array = [0; 16];
    array.copy_from_slice(&bytes);
    array
}

// TODO this function is horrible
fn make_output_names(a: &str, n: usize) -> Result<Vec<String>, FilenameGenerationError> {
    let basepath = Path::new(a);

    let dir = basepath.parent().unwrap();
    let basename = match basepath.file_stem() {
        Some(b) => b.to_str().unwrap().to_owned(),
        None => return Err(FilenameGenerationError::NoneSuchFileStem),
    };

    let dist = 0..n;
    //let exts = vec!["rep0.bam", "rep1.bam"];
    let exts: Vec<String> = dist.into_iter()
                   .map(|a| format!("rep{}.bam",a)).
                   collect();

    let filenames: Vec<String> = std::iter::repeat(basename)
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

    let ext = p.extension().unwrap().to_str().unwrap();

    match p.is_file() {
        true => match  (ext == "bam" || ext == "cram") {
            true => Ok(()),
            false => Err(String::from("Input bam/cram does not appear to be a bam/cram.")),
        },
        false => Err(String::from("Input bam/cram doesn't seem to exist.")),
    }
}
