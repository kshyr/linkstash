use clap::{Args, Parser, Subcommand};
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs::{File, OpenOptions};
use std::io::Write;

const STASH_PATH: &str = "stash.json";

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
    if let Ok(file) = File::open(&STASH_PATH) {
        if let Ok(urls) = serde_json::from_reader::<File, Vec<Link>>(file) {
            return urls;
        }
    }

    Vec::new()
}

fn write_urls(urls: &Vec<Link>) {
    let json = serde_json::to_string_pretty(urls).expect("Failed to serialize URLs to JSON");

    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(STASH_PATH)
        .expect("Failed to open file");

    writeln!(file, "{}", json).expect("Failed to write to file");
}
