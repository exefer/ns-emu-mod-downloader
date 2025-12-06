use crate::{
    EMU_NAME,
    curl_helper::BodyExt,
    entities::game::{Game, ModDownloadEntry},
    entities::github::GitTree,
    utils::read_lines,
};
use curl::easy::Easy;
use rayon::prelude::*;
use std::{
    error::Error,
    ffi::OsString,
    fs::{File, create_dir_all},
    io::{self, Write},
    path::PathBuf,
};

struct ModPathInfo {
    title_name: String,
    title_id: String,
    title_version: String,
    relative_path: String,
}

pub struct ModDownloader {
    client: Easy,
    repository: String,
}

const MOD_SUB_DIRS: &[&str] = &["exefs", "romfs", "cheats"];
const MOD_BASE_VERSIONS: &[&str] = &["1.0", "1.0.0"];

impl ModDownloader {
    pub fn new(repository: String) -> Self {
        Self {
            repository,
            client: Easy::new(),
        }
    }

    fn get_git_tree(&mut self) -> Result<GitTree, Box<dyn Error>> {
        self.client.get(true)?;
        self.client.useragent(env!("CARGO_PKG_NAME"))?;
        self.client.url(&format!(
            "https://api.github.com/repos/{}/git/trees/master?recursive=1",
            self.repository
        ))?;
        Ok(self.client.without_body().send_with_response()?)
    }

    pub fn read_game_titles(&mut self) -> Result<Vec<Game>, Box<dyn Error>> {
        let load_directory = self.get_load_directory_path()?;
        let git_tree = self.get_git_tree()?;

        let games = self
            .get_mod_directories()?
            .iter()
            .filter_map(|mod_dir_name| {
                let title_id = mod_dir_name.to_string_lossy();
                let title_version = self.get_title_version(&title_id).ok()?;
                let mut title_name = String::new();

                let mod_download_entries: Vec<_> = git_tree
                    .tree
                    .iter()
                    .filter(|e| {
                        e.type_field == "blob"
                            && MOD_SUB_DIRS
                                .iter()
                                .any(|s| e.path.contains(&format!("/{s}/")))
                    })
                    .filter_map(|entry| {
                        let info = self.parse_mod_path(&entry.path)?;

                        if OsString::from(&info.title_id) != *mod_dir_name {
                            return None;
                        }

                        if title_name.is_empty() {
                            title_name = info.title_name;
                        }

                        let version_matches = title_version.as_deref() == Some(&info.title_version)
                            || info.title_version == "x.x.x"
                            || (title_version.is_none()
                                && MOD_BASE_VERSIONS.contains(&info.title_version.as_str()));

                        version_matches.then(|| ModDownloadEntry {
                            download_url: format!(
                                "https://raw.githubusercontent.com/{}/refs/heads/master/{}",
                                self.repository, entry.path
                            ),
                            mod_relative_path: info.relative_path,
                        })
                    })
                    .collect();

                (!title_name.is_empty()).then(|| Game {
                    title_name,
                    title_version,
                    mod_download_entries,
                    mod_data_location: load_directory.join(mod_dir_name),
                    title_id: title_id.to_string(),
                })
            })
            .collect();

        Ok(games)
    }

    pub fn download_mods(&self, games: &[Game]) -> Result<(), Box<dyn Error + Send + Sync>> {
        games
            .iter()
            .flat_map(|game| {
                game.mod_download_entries.iter().map(move |entry| {
                    (
                        &entry.download_url,
                        game.mod_data_location.join(&entry.mod_relative_path),
                    )
                })
            })
            .collect::<Vec<_>>()
            .par_iter()
            .try_for_each(|(url, path)| {
                create_dir_all(path.parent().unwrap())?;

                let mut file = File::create(path)?;
                let mut easy = Easy::new();

                easy.get(true)?;
                easy.url(&url.replace(' ', "%20"))?;

                let mut transfer = easy.transfer();

                transfer.write_function(|data| {
                    file.write_all(data)
                        .expect("Failed to write during download");
                    Ok(data.len())
                })?;

                transfer.perform()?;

                Ok(())
            })
    }

    fn get_load_directory_path(&self) -> Result<PathBuf, Box<dyn Error>> {
        let emu = EMU_NAME.get().unwrap();
        let config_path = dirs::config_dir().unwrap().join(emu).join("qt-config.ini");

        for line in read_lines(&config_path)? {
            if let Some(load_dir) = line?.strip_prefix("load_directory=") {
                return Ok(if load_dir.is_empty() {
                    dirs::data_dir().unwrap().join(emu).join("nand")
                } else {
                    load_dir.into()
                });
            }
        }

        Err("Could not find 'load_directory' in config file".into())
    }

    fn get_title_version(&self, title_id: &str) -> io::Result<Option<String>> {
        let emu = EMU_NAME.get().unwrap();
        let pv_path = dirs::cache_dir()
            .unwrap()
            .join(emu)
            .join("game_list")
            .join(format!("{title_id}.pv.txt"));

        if !pv_path.exists() {
            return Ok(None);
        }

        for line in read_lines(pv_path)? {
            if let Some(version) = line?
                .strip_prefix("Update (")
                .and_then(|s| s.strip_suffix(')'))
            {
                return Ok(Some(version.to_owned()));
            }
        }

        Ok(None)
    }

    fn get_mod_directories(&self) -> Result<Vec<OsString>, Box<dyn Error>> {
        Ok(self
            .get_load_directory_path()?
            .read_dir()?
            .collect::<Result<Vec<_>, _>>()?
            .into_iter()
            .map(|e| e.file_name())
            .collect())
    }

    fn parse_mod_path(&self, path: &str) -> Option<ModPathInfo> {
        let mut parts = path.splitn(5, '/');
        parts.next()?;

        Some(ModPathInfo {
            title_name: parts.next()?.to_owned(),
            title_id: parts.next()?.replace(['[', ']'], ""),
            title_version: parts.next()?.to_owned(),
            relative_path: parts.next()?.to_owned(),
        })
    }
}
