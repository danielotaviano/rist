use clap::{Parser, Subcommand};
use flate2::read::ZlibDecoder;
use std::ffi::CStr;
use std::fs;
use std::io::prelude::*;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, Subcommand, Clone)]
enum Command {
    Init,
    CatFile {
        #[clap(short = 'p')]
        pretty_print: bool,

        object_hash: String,
    },
}

fn main() {
    let args = Args::parse();

    match args.command {
        Command::Init => {
            fs::create_dir(".git").unwrap();
            fs::create_dir(".git/objects").unwrap();
            fs::create_dir(".git/refs").unwrap();
            fs::write(".git/HEAD", "ref: refs/heads/main\n").unwrap();
            println!("Initialized git directory")
        }
        Command::CatFile {
            pretty_print: _,
            object_hash,
        } => {
            let f = fs::File::open(format!(
                ".git/objects/{}/{}",
                &object_hash[..2],
                &object_hash[2..]
            ))
            .expect("Can't read the file");

            let mut z = ZlibDecoder::new(f);
            let mut s = String::new();
            z.read_to_string(&mut s).expect("Invalid zlib content");

            let cstr = CStr::from_bytes_until_nul(s.as_bytes())
                .expect("Error when try to extract the bytes until null")
                .to_str()
                .expect("Error when try to convert CStr to a string");

            let file_type = cstr
                .split_whitespace()
                .nth(0)
                .expect("Error when try to extract file type");

            if file_type != "blob" {
                panic!("Unknow file type: {}", file_type);
            }

            let file_size = cstr
                .split_whitespace()
                .nth(1)
                .expect("Error when try to extract file size")
                .parse::<usize>()
                .expect("Error when try to convert file size to number");

            let content = &s[cstr.len() + 1..file_size];
            print!("{}", content);
        }
    }
}
