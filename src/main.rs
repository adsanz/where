use clap::Parser;
use colored::*;
use regex::Regex;
use std::env;
use std::fs::File;
use std::io::Read;
use std::io::Seek;
use std::path::PathBuf;
use std::str::FromStr;
use walkdir::WalkDir;

/// Search type
#[derive(Debug, PartialEq, Clone, Copy)]
enum SearchType {
    Dir,
    File,
    Content,
}

impl FromStr for SearchType {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "dir" => Ok(SearchType::Dir),
            "file" => Ok(SearchType::File),
            "content" => Ok(SearchType::Content),
            _ => Err("no match"),
        }
    }
}

/// CLI tool to find content in your file system. Fast
#[derive(Parser, Debug)]
#[command(version, about, long_about = "A fast file system search tool")]
struct Args {
    /// Type of search to perform. One of "dir", "file", "content"
    #[arg(short, long, default_value = "dir")]
    type_: SearchType,

    /// Regular expression to search for
    #[arg(short, long)]
    expression: Regex,

    /// Max depth to search
    #[arg(short, long, default_value = "10")]
    max_depth: usize,
}

fn dir_finder(where_to_find: &Vec<PathBuf>, expression: Regex, depth: usize) -> Vec<PathBuf> {
    let mut results = Vec::new();
    for path in where_to_find {
        for entry in WalkDir::new(path)
            .max_depth(depth)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let file_name = entry.file_name();
            let path = entry.path();
            if path.is_dir() {
                if expression.is_match(file_name.to_str().unwrap()) {
                    results.push(path.to_path_buf());
                }
            }
        }
    }
    results
}

fn file_finder(where_to_find: &Vec<PathBuf>, expression: Regex, depth: usize) -> Vec<PathBuf> {
    let mut results = Vec::new();
    for path in where_to_find {
        for entry in WalkDir::new(path)
            .max_depth(depth)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let file_name = entry.file_name();
            let path = entry.path();
            if path.is_file() {
                if expression.is_match(file_name.to_str().unwrap()) {
                    results.push(path.to_path_buf());
                }
            }
        }
    }
    results
}

// TODO: Fix the hadouken. Also this is not the best way to check for binary files
fn content_finder(where_to_find: &Vec<PathBuf>, expression: Regex, depth: usize) -> Vec<PathBuf> {
    let mut results = Vec::new();
    for path in where_to_find {
        for entry in WalkDir::new(path)
            .max_depth(depth)
            .into_iter()
            .filter_map(|e| e.ok())
        {
            let path = entry.path();
            if path.is_file() {
                // Check if the file is not binary - kind of weird but avoids reading binary files
                let mut buffer = [0; 1024];
                if let Ok(mut file) = File::open(&path) {
                    if let Ok(n) = file.read(&mut buffer) {
                        if buffer[..n].contains(&0) {
                            continue;
                        } else {
                            if let Ok(_) = file.seek(std::io::SeekFrom::Start(0)) {
                                let mut contents = String::new();
                                if let Ok(_) = file.read_to_string(&mut contents) {
                                    if expression.is_match(&contents) {
                                        results.push(path.to_path_buf());
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
    results
}

fn finder(
    where_to_find: &Vec<PathBuf>,
    expression: Regex,
    search_type: SearchType,
    depth: usize,
) -> Vec<PathBuf> {
    match search_type {
        SearchType::Dir => {
            // search for directories
            dir_finder(&where_to_find, expression, depth)
        }
        SearchType::File => {
            // search for files
            file_finder(&where_to_find, expression, depth)
        }
        SearchType::Content => {
            // search for content
            content_finder(&where_to_find, expression, depth)
        }
    }
}

fn to_absolute_path(path: &str) -> PathBuf {
    let mut p = PathBuf::from(path);
    if p.starts_with("~") {
        if let Some(home) = env::var("HOME")
            .ok()
            .and_then(|h| PathBuf::from(h).canonicalize().ok())
        {
            p = home.join(p.strip_prefix("~").unwrap());
        }
    } else if p.is_relative() {
        if let Ok(cwd) = env::current_dir() {
            p = cwd.join(p);
        }
    }
    p.canonicalize().unwrap_or(p)
}

fn main() {
    let args = Args::parse();
    // Get env variable WHERE_TO_FIND
    let where_to_find: Vec<PathBuf> = env::var("WHERE_TO_FIND")
        .unwrap_or_else(|_| ".".to_string())
        .split(':')
        .map(|p| to_absolute_path(p))
        .collect();

    // search in the file system
    let results = finder(&where_to_find, args.expression, args.type_, args.max_depth);
    for result in results {
        println!("{}", result.display().to_string().cyan().bold());
    }
}
