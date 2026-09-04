// command to initialise a leetcode directory at the current level

use crate::{
    cache::{self},
    config::Config,
    error::{LeetCodeError::CargoInitFailed, Result},
};
use std::{
    fs::create_dir_all,
    path::{Path, PathBuf},
    process::Command,
};

/// initialises a storage directory for leetcode problems + solutions
/// will happily reinitialise to a new dir: renders the old directory dead
pub fn init(path: PathBuf) -> Result<()> {
    let relative_path = path.join("leetcode");
    create_storage_directory(&relative_path)?;

    let storage_path = relative_path.canonicalize()?;
    initialise_cargo_package(&storage_path)?;
    save_storage_path_to_config(&storage_path)?;

    cache::download_and_save_problem_list()?;

    Ok(())
}

fn create_storage_directory(storage_path: &Path) -> Result<()> {
    Ok(create_dir_all(storage_path)?)
}

fn initialise_cargo_package(storage_path: &Path) -> Result<()> {
    let output = Command::new("cargo")
        .arg("init")
        .current_dir(storage_path)
        .output()?;

    if output.status.success() {
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        Err(CargoInitFailed(stderr))
    }
}

fn save_storage_path_to_config(path: &Path) -> Result<()> {
    let mut config = Config::load().unwrap_or_default();
    config.storage_path = Some(path.to_owned());
    config.save()
}
