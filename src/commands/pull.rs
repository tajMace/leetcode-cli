// fetch a problem, write its starter code to src/problems/<slug>/q.<fe>

use std::fs;

use crate::{client::LeetCodeClient, error::Result, manifest::add_bin_entry, models::LangSlug};

pub fn pull(slug: String, lang: LangSlug) -> Result<()> {
    let dirpath = format!("src/problems/{slug}");
    let filepath = format!("{dirpath}/q.{ext}", ext = lang.file_extension());

    if fs::exists(&filepath)? {
        return Ok(());
    };

    let client = LeetCodeClient::new()?;
    let problem = client.fetch_problem(&slug)?;

    fs::create_dir_all(&dirpath)?;
    fs::write(&filepath, problem.generate_problem_file(&lang)?)?;
    if lang == LangSlug::Rust {
        add_bin_entry(&slug)?;
    }

    Ok(())
}
