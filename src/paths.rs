use std::path::PathBuf;

/// Get all directories (cache, config, data) for the given emulator name
pub fn get_dirs(emu: &str) -> (PathBuf, PathBuf, PathBuf) {
    #[cfg(target_os = "macos")]
    {
        let home = dirs::home_dir().unwrap();
        (
            home.join(".cache").join(emu),
            home.join(".config").join(emu),
            home.join(".local").join("share").join(emu),
        )
    }
    #[cfg(not(target_os = "macos"))]
    {
        (
            dirs::cache_dir().unwrap().join(emu),
            dirs::config_dir().unwrap().join(emu),
            dirs::data_dir().unwrap().join(emu),
        )
    }
}
