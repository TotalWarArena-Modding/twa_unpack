extern crate getopts;
extern crate glob;
extern crate tw_pack_lib;

use std::env;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

use getopts::Options;
use glob::glob;

static VERSION: &str = env!("CARGO_PKG_VERSION");

struct Config {
    verbose: bool
}

fn unpack_pack(path: &PathBuf, output_directory: &PathBuf, config: &Config) {
    match File::open(&path) {
        Ok(pack_filename) => {
            let pack = tw_pack_lib::parse_pack(pack_filename).unwrap();
            println!("unpacking {}: {}", &path.display(), &pack);

            for item in pack.into_iter() {
                if config.verbose {
                    println!("{}", &item);
                }
                let target_directory = output_directory.join(&Path::new(&item.path).parent().unwrap());
                let target_path = output_directory.join(&item.path);
                std::fs::create_dir_all(target_directory).unwrap();
                let mut file = OpenOptions::new().write(true).create(true).open(&target_path).unwrap();
                file.write(&item.get_data().unwrap()).unwrap();
            }
        },
        Err(e) => panic!("Could not open file {} ({})", &path.display(), e)
    }
}

fn print_usage(program: &str, opts: Options) {
    let brief = format!("tw_unpack version {}\nUsage: {} FILE", VERSION, program);
    println!("{}", opts.usage(&brief));
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let program = args[0].clone();
    let mut opts = Options::new();
    opts.optopt("o", "", "the output directory for the extracted files. If no output directory is specified, tw_unpack will save the files in the current directory", "OUTPUT");
    opts.optflag("v", "", "enable verbose logging");

    let matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(f) => {
            println!("failed to parse arguments ({})", f);
            return;
        }
    };

    let output_directory_param = matches.opt_str("o");
    let pack_filename_param = if !matches.free.is_empty() {
        matches.free[0].clone()
    } else {
        print_usage(&program, opts);
        return;
    };

    let verbose = if matches.opt_present("v") {
        true
    } else {
        false
    };

    let output_directory = match output_directory_param {
        Some(p) => {
            let path = PathBuf::from(&p);
            if path.exists() {
                path
            } else {
                println!("output directory does not exist");
                return;
            }
        },
        None => {
            match env::current_dir() {
                Ok(curr_dir) => curr_dir,
                Err(e) => {
                    println!("invalid current directory ({})", e);
                    return;
                }
            }
        }
    };

    let config = Config {
        verbose: verbose
    };

    match glob(&pack_filename_param) {
        Ok(glob) => {
            for entry in glob {
                match entry {
                    Ok(path) => {
                        unpack_pack(&path, &output_directory, &config)
                    }
                    Err(e) => println!("failed to handle glob entry ({})", e),
                }
            }
        },
        Err(e) => {
            println!("invalid glob pattern ({})", e);
            return;
        }
    }
}
