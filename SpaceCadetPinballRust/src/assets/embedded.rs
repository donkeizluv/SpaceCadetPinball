use std::env;
use std::path::PathBuf;

pub const PINBALL_DAT_ENV: &str = "PINBALL_DAT";
pub const PINBALL_DAT_NAME: &str = "PINBALL.DAT";
pub const PINBALL_MUSIC_NAME: &str = "PINBALL.MID";
pub const PINBALL_FONT_NAME: &str = "PINBALL2.MID";

pub fn locate_dat_path() -> Result<PathBuf, String> {
    if let Ok(value) = env::var(PINBALL_DAT_ENV) {
        let path = PathBuf::from(value);
        if path.is_file() {
            return Ok(path);
        }
    }

    for candidate in dat_search_candidates()? {
        if candidate.is_file() {
            return Ok(candidate);
        }
    }

    Err(format!(
        "could not find {PINBALL_DAT_NAME} (checked {PINBALL_DAT_ENV}, ./{PINBALL_DAT_NAME}, ./Assets/{PINBALL_DAT_NAME})"
    ))
}

pub fn dat_search_candidates() -> Result<[PathBuf; 2], String> {
    let cwd = env::current_dir().map_err(|error| format!("failed to get current dir: {error}"))?;
    Ok([
        cwd.join(PINBALL_DAT_NAME),
        cwd.join("Assets").join(PINBALL_DAT_NAME),
    ])
}

pub fn asset_path_candidates(file_name: &str) -> Result<[PathBuf; 2], String> {
    let cwd = env::current_dir().map_err(|error| format!("failed to get current dir: {error}"))?;
    Ok([cwd.join(file_name), cwd.join("Assets").join(file_name)])
}
