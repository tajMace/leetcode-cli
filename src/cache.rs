// cache related helper functionality

use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    client::LeetCodeClient,
    error::{LeetCodeError, Result},
    models::ProblemSummary,
};

/*
 * ========== Problem List Cache ==========
 */
pub fn load_cached_problem_list() -> Result<Vec<ProblemSummary>> {
    load_from(&get_cache_filepath()?)
}

pub fn save_cached_problem_list(problems: &[ProblemSummary]) -> Result<()> {
    let path = get_cache_filepath()?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }

    fs::write(path, serde_json::to_string_pretty(problems)?)?;

    Ok(())
}

pub fn download_and_save_problem_list() -> Result<()> {
    let client = LeetCodeClient::new()?;
    let problems = client.fetch_problem_list()?;
    save_cached_problem_list(&problems)?;

    Ok(())
}

// ===== HELPERS =====
fn load_from(path: &Path) -> Result<Vec<ProblemSummary>> {
    let contents = fs::read_to_string(path)?;
    Ok(serde_json::from_str(&contents)?)
}

fn get_cache_filepath() -> Result<PathBuf> {
    let cache_dir = dirs::cache_dir().ok_or_else(|| LeetCodeError::CacheDir)?;
    Ok(cache_dir.join("lc-cli").join("problems.json"))
}
