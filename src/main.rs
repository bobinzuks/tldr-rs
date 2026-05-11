use clap::{Arg, Command};
use colored::*;
use reqwest::blocking::Client;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const VERSION: &str = "3.4.4";
const CLIENT_SPECIFICATION: &str = "2.3";
const PAGES_SOURCE_LOCATION: &str =
    "https://raw.githubusercontent.com/tldr-pages/tldr/main/pages";
const DOWNLOAD_CACHE_LOCATION: &str =
    "https://github.com/tldr-pages/tldr/releases/latest/download/tldr.zip";

fn get_cache_dir() -> PathBuf {
    if let Ok(xdg_cache) = std::env::var("XDG_CACHE_HOME") {
        PathBuf::from(xdg_cache).join("tldr")
    } else if let Ok(home) = std::env::var("HOME") {
        PathBuf::from(home).join(".cache").join("tldr")
    } else {
        PathBuf::from("/tmp").join(".cache").join("tldr")
    }
}

fn get_pages_dir(base_dir: &PathBuf, platform: &str, language: Option<&str>) -> PathBuf {
    if let Some(lang) = language {
        let dir = base_dir.join("pages").join(lang);
        if dir.exists() {
            return dir;
        }
    }
    base_dir.join("pages").join(platform)
}

fn get_cached_page(cache_dir: &PathBuf, command: &str, platform: &str, language: Option<&str>) -> Option<String> {
    let pages_dir = get_pages_dir(cache_dir, platform, language);
    let file_path = pages_dir.join(format!("{}.md", command));
    fs::read_to_string(file_path).ok()
}

fn set_cached_page(cache_dir: &PathBuf, command: &str, platform: &str, language: Option<&str>, content: &str) {
    let pages_dir = get_pages_dir(cache_dir, platform, language);
    fs::create_dir_all(&pages_dir).ok();
    let file_path = pages_dir.join(format!("{}.md", command));
    fs::write(file_path, content).ok();
}

fn update_cache(cache_dir: &PathBuf, client: &Client) {
    println!("{}", "Updating cache...".blue());
    let resp = client
        .get(DOWNLOAD_CACHE_LOCATION)
        .header("User-Agent", "tldr-rust-client")
        .send()
        .expect("Failed to download cache");
    let bytes = resp.bytes().expect("Failed to read response");
    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(bytes))
        .expect("Failed to open zip archive");
    let pages_dir = cache_dir.join("pages");
    if pages_dir.exists() {
        fs::remove_dir_all(&pages_dir).ok();
    }
    archive
        .extract(cache_dir)
        .expect("Failed to extract cache");
    println!("{}", "Cache updated successfully!".green());
}

#[allow(dead_code)]
fn list_pages(cache_dir: &PathBuf) {
    let pages_dir = cache_dir.join("pages");
    if !pages_dir.exists() {
        eprintln!("No cached pages found. Run 'tldr --update' first.");
        return;
    }
    let entries = fs::read_dir(&pages_dir).unwrap();
    for entry in entries.flatten() {
        if entry.file_type().map(|t| t.is_dir()).unwrap_or(false) {
            let platform = entry.file_name();
            if let Ok(platform_entries) = fs::read_dir(entry.path()) {
                for pe in platform_entries.flatten() {
                    if let Some(name) = pe.path().file_stem() {
                        if let Some(name_str) = name.to_str() {
                            println!("{}/{}", platform.to_string_lossy(), name_str);
                        }
                    }
                }
            }
        }
    }
}

#[derive(Deserialize)]
struct TldrPage {
    name: String,
    description: String,
    platform: Option<String>,
    examples: Vec<Example>,
}

#[derive(Deserialize)]
struct Example {
    description: String,
    command: String,
}

fn fetch_page_online(client: &Client, command: &str, platform: &str, language: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
    let urls = vec![
        format!("{}/{}/{}.md", PAGES_SOURCE_LOCATION, platform, command),
        format!("{}/common/{}.md", PAGES_SOURCE_LOCATION, command),
    ];
    for url in &urls {
        let resp = client.get(url).send();
        if let Ok(r) = resp {
            if r.status().is_success() {
                return Ok(r.text()?);
            }
        }
    }
    let url = urls[0].clone();

    let response = client
        .get(&url)
        .header("User-Agent", "tldr-rust-client")
        .send()?
        .error_for_status()?
        .text()?;

    Ok(response)
}

fn render_page(content: &str) {
    for line in content.lines() {
        if line.starts_with("# ") {
            // Command name — bold green
            println!("{}", line[2..].bold().green());
        } else if line.starts_with("> ") {
            // Description — white
            println!("{}", line[2..].white());
        } else if line.starts_with("- ") || line.starts_with("* ") {
            // Example description — yellow
            let example_text = &line[2..];
            if example_text.contains('`') {
                let parts: Vec<&str> = example_text.split('`').collect();
                for (i, part) in parts.iter().enumerate() {
                    if i % 2 == 1 {
                        print!("{}", part.cyan());
                    } else {
                        print!("{}", part.yellow());
                    }
                }
                println!();
            } else {
                println!("{}", example_text.yellow());
            }
        } else if line.starts_with("`") && line.ends_with("`") {
            // Inline code — cyan
            println!("{}", line.trim_matches('`').cyan());
        } else if line.contains('`') {
            let parts: Vec<&str> = line.split('`').collect();
            for (i, part) in parts.iter().enumerate() {
                if i % 2 == 1 {
                    print!("{}", part.cyan());
                } else {
                    // Check for placeholders in backtick content
                    let rendered = part
                        .replace("{{", "")
                        .replace("}}", "");
                    print!("{}", rendered);
                }
            }
            println!();
        } else if line.contains("{{") && line.contains("}}") {
            // Placeholder lines — cyan
            let text = line
                .replace("{{", "")
                .replace("}}", "");
            println!("{}", text.cyan());
        } else {
            println!("{}", line);
        }
    }
}

fn main() {
    let matches = Command::new("tldr")
        .version(VERSION)
        .about("Simplified and community-driven man pages")
        .arg(
            Arg::new("command")
                .help("Command to look up")
                .index(1)
                .required(false),
        )
        .arg(
            Arg::new("update")
                .long("update")
                .help("Update the local cache of tldr pages")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("list")
                .long("list")
                .help("List all cached commands")
                .action(clap::ArgAction::SetTrue),
        )
        .arg(
            Arg::new("platform")
                .long("platform")
                .help("Override the operating system [linux, osx, windows, ...]")
                .action(clap::ArgAction::Set),
        )
        .arg(
            Arg::new("language")
                .long("language")
                .help("Override the language")
                .action(clap::ArgAction::Set),
        )
        .get_matches();

    let cache_dir = get_cache_dir();
    let client = Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .expect("Failed to create HTTP client");

    if matches.get_flag("update") {
        update_cache(&cache_dir, &client);
        return;
    }

    if matches.get_flag("list") {
        list_pages(&cache_dir);
        return;
    }

    let platform = matches
        .get_one::<String>("platform")
        .map(|s| s.as_str())
        .unwrap_or(if cfg!(target_os = "linux") {
            "linux"
        } else if cfg!(target_os = "windows") {
            "windows"
        } else if cfg!(target_os = "macos") {
            "osx"
        } else {
            "linux"
        });

    let language = matches.get_one::<String>("language").map(|s| s.as_str());

    let command = match matches.get_one::<String>("command") {
        Some(cmd) => cmd.clone(),
        None => {
            eprintln!("No command specified. Usage: tldr <command>");
            std::process::exit(1);
        }
    };

    if let Some(cached) = get_cached_page(&cache_dir, &command, platform, language) {
        render_page(&cached);
    } else {
        match fetch_page_online(&client, &command, platform, language) {
            Ok(content) => {
                set_cached_page(&cache_dir, &command, platform, language, &content);
                render_page(&content);
            }
            Err(e) => {
                eprintln!(
                    "Error: Could not find page for '{}': {}",
                    command,
                    e.to_string().red()
                );
                std::process::exit(1);
            }
        }
    }
}
