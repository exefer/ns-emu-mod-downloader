mod curl_helper;
mod entities;
mod mod_downloader;
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

fn get_input(prompt: &str) -> Result<String, Box<dyn Error>> {
    print!("{}", prompt);
    io::stdout().flush()?;
    let mut input = String::new();
    io::stdin().read_line(&mut input)?;
    Ok(input.trim().to_owned())
}

fn display_options<T: fmt::Display>(title: &str, items: &[T]) {
    println!("{}:", title);
    for (i, item) in items.iter().enumerate() {
        println!("  {}) {}", i + 1, item);
    }
}

const EMUS: &[&[u8]] = &[
    &[121, 117, 122, 117],
    &[115, 117, 121, 117],
    &[101, 100, 101, 110],
    &[99, 105, 116, 114, 111, 110],
    &[116, 111, 114, 122, 117],
    &[115, 117, 100, 97, 99, 104, 105],
];

fn select_emulator() -> Result<String, Box<dyn Error>> {
    let emus: Vec<String> = EMUS
        .iter()
        .map(|slice| String::from_utf8(slice.to_vec()).unwrap())
        .collect();

    display_options("\nSelect an emulator to download mods for", &emus);

    let input = get_input(&format!("\nEnter your choice [1-{}]: ", emus.len()))?;
    let choice = input.parse::<usize>().unwrap_or(0).saturating_sub(1);

    let emu = emus.get(choice).ok_or_else(|| {
        format!(
            "Invalid option '{input}'. Please choose a value from 1 to {}.",
            emus.len()
        )
    })?;

    let data_dir = dirs::data_dir().unwrap().join(emu);
    let config_dir = dirs::config_dir().unwrap().join(emu);

    if !data_dir.exists() || !config_dir.exists() {
        println!(
            "\nPlease install {emu} first or verify it's properly configured.\n\
             Expected directories:\n  Data: {}\n  Config: {}\n",
            data_dir.display(),
            config_dir.display()
        );
        return Err(format!("Emulator '{emu}' is not installed on this system.").into());
    }

    Ok(emu.clone())
}

fn try_portable_config() -> Result<Option<Config>, Box<dyn Error>> {
    let user_dir = env::current_exe()?
        .parent()
        .ok_or("Cannot get parent directory")?
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

    Ok(Config {
        config_dir: dirs::config_dir().unwrap().join(&emu),
        data_dir: dirs::data_dir().unwrap().join(&emu),
        cache_dir: dirs::cache_dir().unwrap().join(&emu),
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

    display_options("\nSelect a repository to download mods from", REPOS);

    let input = get_input(&format!("\nEnter your choice [1-{}]: ", REPOS.len()))?;
    let choice = input.parse::<usize>().unwrap_or(0).saturating_sub(1);

    let repo = *REPOS.get(choice).ok_or_else(|| {
        format!(
            "Invalid option '{}'. Please choose 1 to {}.",
            input,
            REPOS.len()
        )
    })?;

    let mut downloader = ModDownloader::new(repo.to_owned(), config);

    let games = downloader.read_game_titles()?;

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

    println!("\nFound mods for the following games:");

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

    let proceed = get_input("\nDo you want to proceed to the download [Y/n]: ")?;

    match proceed.as_str() {
        "Y" => {
            downloader
                .download_mods(&games)
                .map_err(|e| -> Box<dyn Error> { e })?;
            println!("Operation successfull.");
        }
        _ => println!("Operation canceled."),
    }

    Ok(())
}
