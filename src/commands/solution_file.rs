use std::{fs, path::PathBuf};

use crate::{
    config::Config,
    error::{LeetCodeError, Result},
    models::{LangSlug, Question, SOLUTION_MARKER},
};

/*
 * helper function to transform a pulled object into a file
 */
pub fn generate_problem_file(question: &Question, lang: &LangSlug) -> Result<String> {
    let snippet = question
        .lang_snippet(lang)
        .ok_or_else(|| LeetCodeError::UnsupportedLanguage(lang.as_str().to_string()))?;

    Ok(format!(
        "// {title} ({difficulty})\n\
          // https://leetcode.com/problems/{slug}/\n\
          // question_id: {question_id}\n\n\
          {SOLUTION_MARKER}\n\n\
          {code}\n\n",
        title = question.title,
        difficulty = question.difficulty,
        slug = question.title_slug,
        question_id = question.question_id,
        code = snippet.code,
    ))
}

/*
 * helper object and functions to transform a solution file into an object
 */
pub struct ParsedSolution {
    pub question_id: String,
    pub lang: LangSlug,
    pub typed_code: String,
}

pub fn read_and_parse_solution_file(slug: &str, lang: &LangSlug) -> Result<ParsedSolution> {
    let filepath = get_challenge_filepath(slug, lang)?;
    let contents = fs::read_to_string(filepath)?;
    Ok(parse_solution_file(&contents, lang))
}

fn parse_solution_file(contents: &str, lang: &LangSlug) -> ParsedSolution {
    ParsedSolution {
        question_id: get_question_id(contents),
        lang: lang.clone(),
        typed_code: get_solution_code(contents),
    }
}

fn get_question_id(contents: &str) -> String {
    let (_, id_and_code) = contents
        .split_once("// question_id: ")
        .expect("solution file must always contain the marker");

    let (id, _) = id_and_code
        .split_once("\n")
        .expect("question id line must be followed by a new line");

    id.trim().to_string()
}

fn get_solution_code(contents: &str) -> String {
    let (_, code) = contents
        .split_once(SOLUTION_MARKER)
        .expect("solution file must always contain the marker");
    code.trim().to_string()
}

/*
 * question filepath helper
 */

pub fn get_challenge_dir(slug: &str) -> Result<PathBuf> {
    let config = Config::load()?;
    let base_path = config.require_storage_dir()?;
    Ok(base_path.join("src/problems").join(slug))
}

pub fn get_challenge_filepath(slug: &str, lang: &LangSlug) -> Result<PathBuf> {
    let challenge_dir = get_challenge_dir(slug)?;
    let challenge = format!("q.{ext}", ext = lang.file_extension());
    Ok(challenge_dir.join(challenge))
}
