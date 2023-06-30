use clap::{Args, Parser, Subcommand};
use directories_next::{self, ProjectDirs};
use open;
use opengraph;
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs::{create_dir_all, File};
use std::io::Write;
use std::path::PathBuf;

const STASH_FILENAME: &str = "stash.json";

const LOGO: &str = r"
by kshyr    __    _       __   _____ __             __  
           / /   (_)___  / /__/ ___// /_____ ______/ /_ 
          / /   / / __ \/ //_/\__ \/ __/ __ `/ ___/ __ \
         / /___/ / / / / ,<  ___/ / /_/ /_/ (__  ) / / /
        /_____/_/_/ /_/_/|_|/____/\__/\__,_/____/_/ /_/ 
";

#[derive(Debug, Parser)]
#[command(arg_required_else_help = true)]
#[clap(author, version, about = LOGO)]
pub struct CLI {
    #[clap(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// add URL to store
    Add(LinkArg),
    /// delete URL by index
    Delete(IndexArg),
    /// list all links
    List,
    Open(IndexArg),
}

#[derive(Debug, Args)]
pub struct LinkArg {
    /// URL to store
    pub url: String,
}

#[derive(Debug, Args)]
pub struct IndexArg {
    /// index
    pub index: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Link {
    pub url: String,
    pub title: String,
}

fn main() {
    let args = CLI::parse();

    match &args.command {
        Some(Commands::Add(LinkArg { url })) => stash_link(url),
        Some(Commands::Delete(IndexArg { index })) => delete_link(*index),
        Some(Commands::List) => list_all(),
        Some(Commands::Open(IndexArg { index })) => open_link(*index),
        None => (),
    }
}

fn list_all() {
    let urls = read_urls();

    for (i, link) in urls.iter().rev().enumerate() {
        println!("\n{}. {}", i + 1, link.title);
        println!("{}", link.url);
    }
}

fn stash_link(url: &str) {
    if let Ok(obj) = opengraph::scrape(&url, Default::default()) {
        let link = Link {
            url: url.to_string(),
            title: obj.title,
        };

        let mut urls = read_urls();
        urls.push(link);
        write_urls(&urls);
        println!("Added {} to stash.", url);
        list_all();
    } else {
        println!("Error reading link.");
    }
}

fn delete_link(index: usize) {
    let mut urls = read_urls();

    // 1-indexed
    if index <= urls.len() {
        urls.remove(urls.len() - index);
        write_urls(&urls);
        println!("URL at index {} deleted from urls.json", index);
        list_all();
    } else {
        println!("Invalid index!");
    }
}

fn open_link(index: usize) {
    let urls = read_urls();
    let url = &urls.get(urls.len() - index).unwrap().url;
    match open::that(url) {
        Ok(()) => println!("Opened '{}'", url),
        Err(err) => println!("Error when opening '{}': {}", url, err),
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
