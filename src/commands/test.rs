// run local src/problems/<slug>/q.<fe> solution against the example testcases
use crate::{
    client::LeetCodeClient,
    commands::solution_file::read_and_parse_solution_file,
    error::Result,
    models::{LangSlug, RunResult},
};

pub fn test(slug: &str, lang: &LangSlug) -> Result<()> {
    let solution = read_and_parse_solution_file(slug, lang)?;

    let client = LeetCodeClient::new()?;
    let question = client.fetch_question(slug)?;
    let result = client.run_testcases(&question, &solution)?;

    print_run_result(&result);

    Ok(())
}

const RESET: &str = "\x1B[0m";
const BOLD: &str = "\x1B[1m";
const GREEN: &str = "\x1B[1;32m";
const RED: &str = "\x1B[1;31m";
const DIM: &str = "\x1B[2m";
const CLEAR_SCREEN: &str = "\x1B[2J\x1B[1;1H";

fn print_run_result(result: &RunResult) {
    print!("{CLEAR_SCREEN}");

    // check for compiler error first: don't contain the 'correct_answer' field
    if let Some(full_compile_error) = &result.full_compile_error {
        println!("{RED}{BOLD}╔═══════════════════════════════════════╗{RESET}");
        println!("{RED}{BOLD}║           ✗  COMPILE ERROR            ║{RESET}");
        println!("{RED}{BOLD}╚═══════════════════════════════════════╝{RESET}");
        println!();
        println!("{full_compile_error}");
        return;
    }

    if result.correct_answer.unwrap_or(false) {
        println!("{GREEN}{BOLD}╔═══════════════════════════════════════╗{RESET}");
        println!("{GREEN}{BOLD}║              ✓  ACCEPTED              ║{RESET}");
        println!("{GREEN}{BOLD}╚═══════════════════════════════════════╝{RESET}");
        println!();
        println!(
            "  {}/{} testcases passed",
            result.total_correct.unwrap_or(0),
            result.total_testcases.unwrap_or(0),
        );
        println!(
            "  Runtime: {}  ({})",
            result.status_runtime,
            percentile_str(result.runtime_percentile, "faster")
        );
        println!(
            "  Memory:  {}  ({})",
            result.status_memory,
            percentile_str(result.memory_percentile, "less memory")
        );
        return;
    }

    println!("{RED}{BOLD}╔═══════════════════════════════════════╗{RESET}");
    println!("{RED}{BOLD}║               ✗  FAILED               ║{RESET}");
    println!("{RED}{BOLD}╚═══════════════════════════════════════╝{RESET}");
    println!();
    println!(
        "  {}/{} testcases passed",
        result.total_correct.unwrap_or(0),
        result.total_testcases.unwrap_or(0),
    );
    println!();

    let compare_result = result.compare_result.as_deref().unwrap_or("");
    for (i, passed) in compare_result.chars().enumerate() {
        if passed == '0' {
            let got = result
                .code_answer
                .as_ref()
                .and_then(|v| v.get(i))
                .map(String::as_str)
                .unwrap_or("?");
            let expected = result
                .expected_code_answer
                .as_ref()
                .and_then(|v| v.get(i))
                .map(String::as_str)
                .unwrap_or("?");
            println!("{DIM}  testcase {}:{RESET}", i + 1);
            println!("    expected: {expected}");
            println!("    got:      {got}");
        }
    }
}

fn percentile_str(percentile: Option<f32>, comparison: &str) -> String {
    match percentile {
        Some(p) => format!("beats {p:.1}% of submissions on {comparison}"),
        None => "percentile unavailable".to_string(),
    }
}
