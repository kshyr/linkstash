use ansi_term::{
    Color::{Blue, Cyan, Green, Red, RGB},
    Style,
};
use clap::{Args, Parser, Subcommand};
use directories_next::{self, ProjectDirs};
use open;
use opengraph;
use serde::{Deserialize, Serialize};
use serde_json;
use std::io::Write;
use std::path::PathBuf;
use std::{
    fs::{create_dir_all, File},
    process::Command,
};

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
#[clap(author, version, about = format!("{}", RGB(170, 170, 240).bold().paint(LOGO)))]
pub struct CLI {
    #[clap(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Add URL to store
    Add(LinkArg),
    /// Delete URL by index
    Delete(IndexArg),
    /// List all links
    List,
    /// Open link with default browser or app of choice
    Open(OpenArgs),
}

#[derive(Debug, Args)]
pub struct LinkArg {
    /// URL to store
    pub url: String,
}

#[derive(Debug, Args)]
pub struct IndexArg {
    /// Index
    pub index: usize,
}

#[derive(Debug, Args)]
pub struct OpenArgs {
    /// Index
    pub index: usize,
    /// Program to open link with
    pub program: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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
        Some(Commands::Open(OpenArgs { index, program })) => open_link(*index, program),
        None => (),
    }
}

fn list_all() {
    let urls = read_urls();

    println!("");
    for (i, link) in urls.iter().rev().enumerate() {
        println!(
            "    {}. {}",
            Style::new().bold().paint((i + 1).to_string()),
            Blue.bold().paint(&link.title)
        );
        println!("       {}", Style::new().italic().paint(&link.url));
        println!("");
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
        println!("\nAdded {} to stash.\n", Style::new().italic().paint(url));
        list_all();
    } else {
        println!("{}", RGB(250, 100, 100).bold().paint("Error reading link."));
    }
}

fn delete_link(index: usize) {
    let mut urls = read_urls();

    // 1-indexed
    if index <= urls.len() {
        let url = urls.get(urls.len() - index).unwrap().url.clone();
        urls.remove(urls.len() - index);
        write_urls(&urls);
        println!(
            "\n{} is removed from stash.\n",
            Style::new().italic().paint(url)
        );
        list_all();
    } else {
        println!("{}", RGB(250, 100, 100).bold().paint("Invalid index!"));
    }
}

fn open_link(index: usize, program: &Option<String>) {
    let urls = read_urls();
    let url = &urls.get(urls.len() - index).unwrap().url;
    if let Some(prog) = program {
        Command::new(prog).arg(url).spawn().unwrap();
    } else {
        match open::that(url) {
            Ok(()) => println!("\nOpened '{}'\n", Style::new().italic().paint(url)),
            Err(err) => println!(
                "Error when opening '{}': {}",
                Style::new().italic().paint(url),
                RGB(250, 100, 100).bold().paint(err.to_string())
            ),
        }
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
