mod commands;
mod object;

use clap::{Parser, Subcommand};

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
    HashObject {
        #[clap(short = 'w')]
        write: bool,

        path: String,
    },
    LsTree {
        #[clap(long)]
        name_only: bool,
        tree_hash: String,
    },
}

fn main() {
    let args = Args::parse();

    match args.command {
        Command::Init => commands::init(),
        Command::CatFile {
            pretty_print: _,
            object_hash,
        } => commands::cat_file(object_hash),
        Command::HashObject { write, path } => commands::hash_object(path, write),
        Command::LsTree {
            tree_hash,
            name_only,
        } => commands::ls_tree(tree_hash, name_only),
    }
}
