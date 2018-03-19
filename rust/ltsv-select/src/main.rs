extern crate getopts;

use std::env;
use std::io;
use std::io::BufRead;
use std::process;
use std::fs::File;
use std::collections::{HashSet, LinkedList};

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();

    let mut options = getopts::Options::new();
    options.optmulti("l", "label", "Select this label", "LABEL");
    options.optflag("h", "help", "Print help");
    let matches = match options.parse(&args[1..]) {
        Ok(m) => m,
        Err(msg) => {
            println!("{}", msg);
            print_usage(&program, &options);
            process::exit(1);
        }
    };

    if matches.opt_present("h") {
        print_usage(&program, &options);
        process::exit(0);
    }

    let mut labels = HashSet::new();
    for label in matches.opt_strs("l") {
        labels.insert(label);
    }
    match ltsv_select(&labels, matches.free.get(0)) {
        Ok(_) => {}
        Err(msg) => {
            println!("{}", msg);
            process::exit(2);
        }
    }
}

fn print_usage(program: &str, options: &getopts::Options) {
    println!("{}", options.short_usage(program));
    println!("{}", options.usage("Filter LTSV records."));
}

fn ltsv_select(labels: &HashSet<String>, path: Option<&String>) -> io::Result<()> {
    match path {
        None => {
            let stdin = io::stdin();
            let lock = stdin.lock();
            ltsv_select2(labels, lock)
        }
        Some(path) => {
            let file = try!(File::open(path));
            ltsv_select2(labels, io::BufReader::new(file))
        }
    }
}

fn ltsv_select2<R: BufRead>(labels: &HashSet<String>, reader: R) -> io::Result<()> {
    for line in reader.lines() {
        let line = try!(line);
        let mut record = LinkedList::new();
        for label_and_value in line.split('\t') {
            let xs: Vec<&str> = label_and_value.split(':').collect();
            if !xs.is_empty() {
                let label = xs[0];
                if labels.is_empty() || labels.contains(label) {
                    record.push_back(label_and_value);
                }
            }
        }

        let mut first = true;
        for label_and_value in record {
            if first {
                print!("{}", label_and_value);
            } else {
                print!("\t{}", label_and_value);
            }
            first = false;
        }
        println!();
    }

    Ok(())
}
