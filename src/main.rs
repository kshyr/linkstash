use clap::{Args, Parser, Subcommand};
use directories_next::{self, ProjectDirs};
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::PathBuf;

const STASH_FILENAME: &str = "stash.json";

#[derive(Debug, Parser)]
#[command(arg_required_else_help = true)]
#[clap(author, version, about)]
pub struct CLI {
    #[clap(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// add URL to store
    Add(Link),
    /// delete URL by index
    Delete(Index),
}

#[derive(Debug, Args, Serialize, Deserialize)]
pub struct Link {
    /// URL to store
    pub url: String,
}

#[derive(Debug, Args)]
pub struct Index {
    /// index
    pub index: usize,
}

fn main() {
    let args = CLI::parse();

    match &args.command {
        Some(Commands::Add(Link { url })) => stash_link(url),
        Some(Commands::Delete(Index { index })) => delete_link(*index),
        None => (),
    }
}

fn stash_link(url: &str) {
    let mut urls = read_urls();

    urls.push(Link {
        url: url.to_owned(),
    });

    write_urls(&urls);

    println!("Added {} to stash.", url);
    println!("{:?}", urls);
}

fn delete_link(index: usize) {
    let mut urls = read_urls();

    if index < urls.len() {
        urls.remove(index);
        write_urls(&urls);
        println!("URL at index {} deleted from urls.json", index);
    } else {
        println!("Invalid index!");
    }
}

fn read_urls() -> Vec<Link> {
    if let Ok(file) = File::open(get_stash_path()) {
        if let Ok(urls) = serde_json::from_reader::<File, Vec<Link>>(file) {
            return urls;
        }
    }

    Vec::new()
}

fn write_urls(urls: &Vec<Link>) {
    let json = serde_json::to_string_pretty(urls).expect("Failed to serialize URLs to JSON");

    if let Ok(mut stash) = File::create(get_stash_path()) {
        writeln!(stash, "{}", json).expect("Failed to write to file");
    }
}

fn get_stash_path() -> PathBuf {
    let proj_dirs = ProjectDirs::from("com", "kshyr", "linkstash").expect("Proj dirs error.");
    let config_dir = proj_dirs.config_dir();

    if !config_dir.is_dir() {
        create_dir_all(config_dir).expect("Failed to create config directory.");
    }

    config_dir.join(STASH_FILENAME)
}
