mod curl_helper;
mod entities;
mod mod_downloader;
mod paths;
mod utils;

use mod_downloader::ModDownloader;
use std::{
    collections::HashSet,
    env,
    error::Error,
    fmt,
    io::{self, Write},
    path::PathBuf,
};

#[derive(Debug, Clone)]
pub struct Config {
    pub cache_dir: PathBuf,
    pub config_dir: PathBuf,
    pub data_dir: PathBuf,
}

fn ask(prompt: &str) -> Result<String, Box<dyn Error>> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_owned())
}

fn display_options<T: fmt::Display>(title: &str, items: &[T]) {
    println!(":: {}:", title);
    for (i, item) in items.iter().enumerate() {
        println!("  {}) {}", i + 1, item);
    }
}

const EMUS: &[&str] = &["yuzu", "suyu", "eden", "citron", "torzu", "sudachi"];

fn select_emulator() -> Result<String, Box<dyn Error>> {
    println!();
    display_options("Select emulator", EMUS);

    let input = ask("Enter a number (default=1): ")?;
    let choice = input.parse::<usize>().unwrap_or(0).saturating_sub(1);

    let emu = *EMUS
        .get(choice)
        .ok_or_else(|| format!("invalid option, please choose 1-{}", EMUS.len()))?;

    let (_, config_dir, data_dir) = paths::get_dirs(emu);

    if !data_dir.exists() || !config_dir.exists() {
        eprintln!();
        eprintln!("Expected directories:");
        eprintln!("  Data: {}", data_dir.display());
        eprintln!("  Config: {}", config_dir.display());
        return Err(format!("{} is not installed", emu).into());
    }

    Ok(emu.to_owned())
}

fn try_portable_config() -> Result<Option<Config>, Box<dyn Error>> {
    let user_dir = env::current_exe()?
        .parent()
        .ok_or("cannot get parent directory")?
        .join("user");

    if user_dir.exists() && user_dir.join("config").join("qt-config.ini").exists() {
        return Ok(Some(Config {
            cache_dir: user_dir.join("cache"),
            config_dir: user_dir.join("config"),
            data_dir: user_dir,
        }));
    }

    Ok(None)
}

fn build_config() -> Result<Config, Box<dyn Error>> {
    if let Some(portable_config) = try_portable_config()? {
        return Ok(portable_config);
    }

    let emu = select_emulator()?;
    let (cache_dir, config_dir, data_dir) = paths::get_dirs(&emu);

    Ok(Config {
        cache_dir,
        config_dir,
        data_dir,
    })
}

const REPOS: &[&str] = &[
    "exefer/switch-port-mods",
    "exefer/switch-pchtxt-mods",
    "exefer/Switch-Ultrawide-Mods",
    "exefer/ue4-emuswitch-60fps",
];

fn main() -> Result<(), Box<dyn Error>> {
    println!("=== Mod Downloader ===");

    let config = build_config()?;

    println!();
    display_options("Select repository", REPOS);

    let input = ask("Enter a number (default=1): ")?;
    let choice = input.parse::<usize>().unwrap_or(0).saturating_sub(1);

    let repo = *REPOS
        .get(choice)
        .ok_or_else(|| format!("invalid option, please choose 1-{}", REPOS.len()))?;

    let mut downloader = ModDownloader::new(repo.to_owned(), config);

    let games = downloader.read_game_titles()?;

    println!();
    if games.is_empty() {
        println!("No mod installation folders found on this system.");
        return Ok(());
    }

    let games: Vec<_> = games
        .into_iter()
        .filter(|game| !game.mod_download_entries.is_empty())
        .collect();

    if games.is_empty() {
        println!("No mods available for any installed game.");
        return Ok(());
    }

    println!("Found mods for the following games:");

    for (i, game) in games.iter().enumerate() {
        let mods = game
            .mod_download_entries
            .iter()
            .filter_map(|entry| {
                entry
                    .mod_relative_path
                    .split_once("/")
                    .map(|(first, _)| first)
            })
            .collect::<HashSet<&str>>();
        println!("  {}) {}: {} mods", i + 1, game.title_name, mods.len());
    }

    println!();
    let proceed = ask("Proceed with download? [Y/n]: ")?;
    let proceed = proceed.to_lowercase();

    match proceed.as_str() {
        "y" | "yes" | "" => {
            downloader
                .download_mods(&games)
                .map_err(|e| -> Box<dyn Error> { e })?;
            println!("Operation successful.");
        }
        _ => println!("Operation canceled."),
    }

    Ok(())
}
