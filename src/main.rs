use clap::Parser;
use colored::*;
use rayon::prelude::*;
use regex::Regex;
use std::env;
use std::fs::File;
use std::io::Read;
use std::io::Seek;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use walkdir::WalkDir;

/// ENUNMS AND DEFFINITIONS

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

#[derive(Debug, PartialEq, Clone)]
enum SearchResult {
    Verbose { path: PathBuf, line: String },
    Simple(PathBuf),
}

/// CLI tool to find content in your file system. Fast
#[derive(Parser, Debug)]
#[command(
    version,
    about,
    long_about = "A fast file system search tool. Remember to set WHERE_TO_FIND env variable to the directories you want to search in."
)]
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

    /// Verbose mode - allows first line of content matched by regex to be displayed. May decrease performance
    /// Only works with content search
    #[arg(short, long, default_value = "false")]
    verbose: bool,
}

/// HELPER FUNCTIONS

fn entry_builder(where_to_find: &Vec<PathBuf>, depth: usize) -> Vec<walkdir::DirEntry> {
    let results = Arc::new(Mutex::new(Vec::new()));
    for path in where_to_find {
        let entries: Vec<_> = WalkDir::new(path)
            .max_depth(depth)
            .into_iter()
            .filter_map(|e| e.ok())
            .collect();
        results.lock().unwrap().extend(entries);
    }
    Arc::try_unwrap(results).unwrap().into_inner().unwrap()
}

fn first_line_matched(mut file: &File, expression: &Regex, verbose: bool) -> Option<String> {
    let mut buffer = Vec::new();
    if let Ok(_) = file.seek(std::io::SeekFrom::Start(0)) {
        if file.read_to_end(&mut buffer).is_ok() {
            let content = String::from_utf8_lossy(&buffer);
            if verbose {
                for line in content.lines() {
                    if expression.is_match(line) {
                        return Some(line.to_string());
                    }
                }
            } else {
                if expression.is_match(&content) {
                    return Some("Matched".to_string());
                }
            }
        }
    }
    None
}

fn binary_checker(mut file: &File) -> bool {
    // Check if the file is not binary - kind of weird but avoids reading binary files
    let mut buffer = [0; 1024];
    if let Ok(n) = file.read(&mut buffer) {
        if buffer[..n].contains(&0) {
            return true;
        }
    }
    false
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

/// MAIN FUNCTIONS

fn dir_finder(where_to_find: &Vec<PathBuf>, expression: Regex, depth: usize) -> Vec<SearchResult> {
    let results = Arc::new(Mutex::new(Vec::new()));
    let entries = entry_builder(where_to_find, depth);
    entries.iter().for_each(|entry| {
        let file_name = entry.file_name();
        let path = entry.path();
        if path.is_dir() {
            if expression.is_match(file_name.to_str().unwrap()) {
                let mut results = results.lock().unwrap();
                results.push(SearchResult::Simple(path.to_path_buf()));
            }
        }
    });

    Arc::try_unwrap(results).unwrap().into_inner().unwrap()
}

fn file_finder(where_to_find: &Vec<PathBuf>, expression: Regex, depth: usize) -> Vec<SearchResult> {
    let results = Arc::new(Mutex::new(Vec::new()));
    let entries = entry_builder(where_to_find, depth);
    entries.iter().for_each(|entry| {
        let file_name = entry.file_name();
        let path = entry.path();
        if path.is_file() {
            if expression.is_match(file_name.to_str().unwrap()) {
                let mut results = results.lock().unwrap();
                results.push(SearchResult::Simple(path.to_path_buf()));
            }
        }
    });

    Arc::try_unwrap(results).unwrap().into_inner().unwrap()
}

fn content_finder(
    where_to_find: &Vec<PathBuf>,
    expression: Regex,
    depth: usize,
    verbose: bool,
) -> Vec<SearchResult> {
    let results = Arc::new(Mutex::new(Vec::new()));
    let entries = entry_builder(where_to_find, depth);
    entries.par_iter().for_each(|entry| {
        let path = entry.path();
        if path.is_file() {
            if let Ok(file) = File::open(&path) {
                if binary_checker(&file) {
                    return;
                } else {
                    let mut results = results.lock().unwrap();
                    let matched_line = first_line_matched(&file, &expression, verbose);
                    if matched_line.is_some() {
                        if verbose {
                            results.push(SearchResult::Verbose {
                                path: path.to_path_buf(),
                                line: matched_line.unwrap(),
                            });
                        } else {
                            results.push(SearchResult::Simple(path.to_path_buf()));
                        }
                    }
                }
            }
        }
    });
    Arc::try_unwrap(results).unwrap().into_inner().unwrap()
}

fn finder(
    where_to_find: &Vec<PathBuf>,
    expression: Regex,
    search_type: SearchType,
    depth: usize,
    verbose: bool,
) -> Vec<SearchResult> {
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
            content_finder(&where_to_find, expression, depth, verbose)
        }
    }
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
    let results = finder(
        &where_to_find,
        args.expression,
        args.type_,
        args.max_depth,
        args.verbose,
    );
    for result in results {
        match result {
            SearchResult::Simple(path) => {
                println!("{}", path.display().to_string().green().bold());
            }
            SearchResult::Verbose { path, line } => {
                println!(
                    "{} -> {}",
                    path.display().to_string().green().bold(),
                    line.cyan().bold()
                );
            }
        }
    }
}
