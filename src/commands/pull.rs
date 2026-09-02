// fetch a problem, write its starter code to src/problems/<slug>/q.<fe>

use std::fs;

use crate::{
    client::LeetCodeClient,
    commands::solution_file::{
        generate_problem_file, generate_spec_file, get_challenge_dir, get_challenge_filepath,
        get_spec_filepath,
    },
    error::Result,
    manifest::add_bin_entry,
    models::LangSlug,
};

pub fn pull(slug: String, lang: LangSlug) -> Result<()> {
    let dirpath = get_challenge_dir(&slug)?;
    let spec_filepath = get_spec_filepath(&slug)?;
    let challenge_filepath = get_challenge_filepath(&slug, &lang)?;

    // don't repull existing challenge
    if fs::exists(&challenge_filepath)? {
        return Ok(());
    };

    let client = LeetCodeClient::new()?;
    let question = client.fetch_question(&slug)?;

    fs::create_dir_all(&dirpath)?;
    if !fs::exists(&spec_filepath)? {
        fs::write(&spec_filepath, generate_spec_file(&question)?)?;
    }

    fs::write(
        &challenge_filepath,
        generate_problem_file(&question, &lang)?,
    )?;
    if lang == LangSlug::Rust {
        add_bin_entry(&slug)?;
    }

    Ok(())
}
