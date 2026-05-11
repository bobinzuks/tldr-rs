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
        PathBuf::from("/tmp").join("tldr")
    }
}

fn get_language_code(language: &str) -> String {
    let lang = language.split('.').next().unwrap_or(language);
    match lang {
        "pt_PT" | "pt_BR" | "zh_TW" => lang.to_string(),
        "pt" => "pt_PT".to_string(),
        _ => lang.split('_').next().unwrap_or("C").to_string(),
    }
}

fn get_default_language() -> Option<String> {
    let lang = std::env::var("LANG").unwrap_or_else(|_| "C".to_string());
    let code = get_language_code(&lang);
    if code == "C" || code == "POSIX" {
        None
    } else {
        Some(code)
    }
}

fn get_platform() -> String {
    let os = std::env::consts::OS;
    match os {
        "android" => "android".to_string(),
        "macos" => "osx".to_string(),
        "freebsd" => "freebsd".to_string(),
        "linux" => "linux".to_string(),
        "netbsd" => "netbsd".to_string(),
        "openbsd" => "openbsd".to_string(),
        "sunos" => "sunos".to_string(),
        "windows" => "windows".to_string(),
        _ => "linux".to_string(),
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

fn cache_get(cache_dir: &PathBuf, command: &str, platform: &str, language: Option<&str>) -> Option<String> {
    let pages_dir = get_pages_dir(cache_dir, platform, language);
    let file_path = pages_dir.join(format!("{}.md", command));
    if file_path.exists() {
        fs::read_to_string(&file_path).ok()
    } else {
        None
    }
}

fn cache_set(cache_dir: &PathBuf, command: &str, platform: &str, language: Option<&str>, content: &str) {
    let pages_dir = get_pages_dir(cache_dir, platform, language);
    fs::create_dir_all(&pages_dir).ok();
    let file_path = pages_dir.join(format!("{}.md", command));
    fs::write(&file_path, content).ok();
}

fn update_cache(cache_dir: &PathBuf, client: &Client) -> Result<(), Box<dyn std::error::Error>> {
    println!("{}", "Updating cache...".blue());

    let response = client
        .get(DOWNLOAD_CACHE_LOCATION)
        .header("User-Agent", "tldr-rust-client")
        .send()?
        .bytes()?;

    let mut archive = zip::ZipArchive::new(std::io::Cursor::new(response))?;

    // Remove old cache
    if cache_dir.exists() {
        fs::remove_dir_all(cache_dir)?;
    }

    // Extract new cache
    archive.extract(cache_dir)?;

    println!("{}", "Cache updated successfully!".green());
    Ok(())
}

fn list_pages(cache_dir: &PathBuf, platform: &str, language: Option<&str>) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let pages_dir = get_pages_dir(cache_dir, platform, language);
    if !pages_dir.exists() {
        return Err("Cache directory does not exist. Run 'tldr --update' first.".into());
    }

    let mut pages = Vec::new();
    for entry in fs::read_dir(&pages_dir)? {
        let entry = entry?;
        if entry.path().extension().and_then(|s| s.to_str()) == Some("md") {
            if let Some(name) = entry.path().file_stem().and_then(|s| s.to_str()) {
                pages.push(name.to_string());
            }
        }
    }

    pages.sort();
    Ok(pages)
}

fn fetch_page_online(client: &Client, command: &str, platform: &str, language: Option<&str>) -> Result<String, Box<dyn std::error::Error>> {
    // Try platform-specific first, then common
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
            println!("{}", line[2..].bold().yellow());
        } else if line.starts_with("> ") {
            println!("{}", line[2..].white());
        } else if line.starts_with("- ") || line.starts_with("* ") {
            println!("{}", line[2..].green());
        } else if line.starts_with("`") && line.ends_with("`") {
            println!("{}", line.trim_matches('`').cyan());
        } else if line.contains('`') {
            let parts: Vec<&str> = line.split('`').collect();
            for (i, part) in parts.iter().enumerate() {
                if i % 2 == 1 {
                    print!("{}", part.cyan());
                } else {
                    print!("{}", part);
                }
            }
            println!();
        } else if line.starts_with("{{") && line.ends_with("}}") {
            println!("{}", line[2..line.len()-2].green().italic());
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
                .help("Update the local cache")
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

    let platform = matches
        .get_one::<String>("platform")
        .cloned()
        .unwrap_or_else(get_platform);
    let language = matches
        .get_one::<String>("language")
        .cloned()
        .or_else(get_default_language);

    if matches.get_flag("update") {
        match update_cache(&cache_dir, &client) {
            Ok(_) => std::process::exit(0),
            Err(e) => {
                eprintln!("{}: {}", "Error".red(), e);
                std::process::exit(1);
            }
        }
    }

    if matches.get_flag("list") {
        match list_pages(&cache_dir, &platform, language.as_deref()) {
            Ok(pages) => {
                for page in pages {
                    println!("{}", page);
                }
                std::process::exit(0);
            }
            Err(e) => {
                eprintln!("{}: {}", "Error".red(), e);
                std::process::exit(1);
            }
        }
    }

    let command = match matches.get_one::<String>("command") {
        Some(cmd) => cmd.to_lowercase(),
        None => {
            eprintln!("{}", "No command specified. Use --help for usage.".red());
            std::process::exit(1);
        }
    };

    // Try cache first
    if let Some(content) = cache_get(&cache_dir, &command, &platform, language.as_deref()) {
        render_page(&content);
        std::process::exit(0);
    }

    // Try online
    match fetch_page_online(&client, &command, &platform, language.as_deref()) {
        Ok(content) => {
            cache_set(&cache_dir, &command, &platform, language.as_deref(), &content);
            render_page(&content);
        }
        Err(e) => {
            println!("{}", format!("Could not find page for '{}'.", command).red());
            if let Some(lang) = &language {
                println!("{}", format!("Tried language: {}", lang).yellow());
            }
            println!("{}", format!("Tried platform: {}", platform).yellow());
            eprintln!("{}", e);
            std::process::exit(1);
        }
    }
}
