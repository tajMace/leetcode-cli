// LeetCodeClient: wraps an HTTP client, talks to leetcode.com/graphql
// (unauthenticated: fetch problem) and the REST run/submit/check endpoints
// (authenticated: needs the session cookie from config.rs)

use crate::commands::ParsedSolution;
use crate::config::Config;
use crate::error::{LeetCodeError, Result};
use crate::models::{Question, RunResult};

const LEETCODE_GRAPHQL_ENDPOINT: &str = "https://leetcode.com/graphql/";
const FETCH_QUESTION_QUERY: &str = "query fetchProblem($titleSlug: String!) {
  question(titleSlug: $titleSlug) {
    questionId
    questionFrontendId
    title
    titleSlug
    difficulty
    content
    codeSnippets { lang langSlug code }
    exampleTestcaseList
  }
}";

pub struct LeetCodeClient {
    http: reqwest::blocking::Client,
    config: Config,
}

impl LeetCodeClient {
    pub fn new() -> Result<LeetCodeClient> {
        Ok(LeetCodeClient {
            http: reqwest::blocking::Client::new(),
            config: Config::load()?,
        })
    }

    /* HTTP FETCH HELPERS */
    fn query_graphql(
        &self,
        query: &str,
        variables: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let body = serde_json::json!({
            "query": query,
            "variables": variables,
        });

        Ok(self
            .http
            .post(LEETCODE_GRAPHQL_ENDPOINT)
            .json(&body)
            .send()?
            .json()?)
    }

    pub fn fetch_question(&self, slug: &str) -> Result<Question> {
        let query = FETCH_QUESTION_QUERY;
        let variables = serde_json::json!({
            "titleSlug": slug
        });
        let ret = self.query_graphql(query, variables)?;
        Ok(Question::from_graphql_value(&ret, slug)?)
    }

    /// Runs a solution against a problem's visible example testcases via
    /// LeetCode's `interpret_solution/` endpoint (the "Run" button, not a
    /// real submission)
    pub fn run_testcases(
        &self,
        question: &Question,
        solution: &ParsedSolution,
    ) -> Result<RunResult> {
        let slug = &question.title_slug;
        let referer = problem_referer(slug);
        let url = format!("{referer}/interpret_solution/");

        let body = serde_json::json!({
            "lang": solution.lang.as_str(),
            "question_id": &solution.question_id,
            "typed_code": solution.typed_code,
            "data_input": question.example_testcase_list.join("\n")
        });

        let response_text = self
            .with_auth_headers(self.http.post(&url), &referer)?
            .json(&body)
            .send()?
            .text()?;

        let response_json: serde_json::Value = serde_json::from_str(&response_text)?;
        let interpret_id = response_json
            .get("interpret_id")
            .and_then(|v| v.as_str())
            .expect("response was not in the expected shape: possibly a bot check");

        self.get_run_status(interpret_id, slug)
    }

    /* ----- private helpers ----- */
    fn with_auth_headers(
        &self,
        builder: reqwest::blocking::RequestBuilder,
        referer: &str,
    ) -> Result<reqwest::blocking::RequestBuilder> {
        let (session, csrf) = self.config.require_session()?;
        Ok(builder
            .header("Referer", referer)
            .header("x-csrftoken", csrf)
            .header(
                "Cookie",
                format!("LEETCODE_SESSION={session}; csrftoken={csrf}"),
            ))
    }

    /// polls `/submissions/detail/<id>/check/` until the run finishes,
    /// then deserializes the final response into `RunResult`.
    fn get_run_status(&self, interpret_id: &str, slug: &str) -> Result<RunResult> {
        let url = format!("https://leetcode.com/submissions/detail/{interpret_id}/check/");

        for _ in 0..20 {
            let response: serde_json::Value = self
                .with_auth_headers(self.http.get(&url), &problem_referer(slug))?
                .header("Content-Type", "application/json")
                .send()?
                .json()?;

            let state = response.get("state").and_then(|v| v.as_str()).unwrap_or("");

            if state == "SUCCESS" {
                eprintln!("{response:?}");
                let result: RunResult = serde_json::from_value(response)?;
                return Ok(result);
            }

            // limit polling rate to avoid blockout
            std::thread::sleep(std::time::Duration::from_millis(500));
        }

        // shouldn't get here: likely a hanging server issue
        Err(LeetCodeError::TestingTooLong)
    }
}

// referer is always the same: the associated problem
fn problem_referer(slug: &str) -> String {
    format!("https://leetcode.com/problems/{slug}")
}

/*
 * ========== UNIT TESTS ==========
 */
#[cfg(test)]
mod client_tests {
    use super::*;
    use crate::models::LangSlug;

    fn live_client() -> LeetCodeClient {
        LeetCodeClient::new().expect("client should construct")
    }

    fn two_sum_question(client: &LeetCodeClient) -> Question {
        client
            .fetch_question("two-sum")
            .expect("should fetch two-sum")
    }

    #[test]
    #[ignore = "hits the real LeetCode API — run manually with `cargo test -- --ignored --nocapture`"]
    fn run_testcases_reports_accepted_for_correct_solution() {
        let client = live_client();
        let question = two_sum_question(&client);

        let solution = ParsedSolution {
            lang: LangSlug::Rust,
            question_id: question.question_id.clone(),
            typed_code: r#"
 impl Solution {
     pub fn two_sum(nums: Vec<i32>, target: i32) -> Vec<i32> {
         use std::collections::HashMap;
         let mut seen = HashMap::new();
         for (i, &n) in nums.iter().enumerate() {
             if let Some(&j) = seen.get(&(target - n)) {
                 return vec![j as i32, i as i32];
             }
             seen.insert(n, i);
         }
         vec![]
     }
 }
 "#
            .to_string(),
        };

        let result = client
            .run_testcases(&question, &solution)
            .expect("run_testcases should succeed");

        assert_eq!(result.status_code, 10);
        assert_eq!(result.status_msg, "Accepted");
        assert_eq!(result.correct_answer, Some(true));
        assert_eq!(result.total_correct, result.total_testcases);
    }

    #[test]
    #[ignore = "hits the real LeetCode API — run manually with `cargo test -- --ignored --nocapture`"]
    fn run_testcases_reports_wrong_answer_for_bad_solution() {
        let client = live_client();
        let question = two_sum_question(&client);

        let solution = ParsedSolution {
            lang: LangSlug::Rust,
            question_id: question.question_id.clone(),
            typed_code: r#"
 impl Solution {
     pub fn two_sum(nums: Vec<i32>, target: i32) -> Vec<i32> {
         vec![0, 0]
     }
 }
 "#
            .to_string(),
        };

        let result = client
             .run_testcases(&question, &solution)
             .expect("run_testcases should succeed (as an API call — the *solution* is wrong, not the request)");

        assert_eq!(result.correct_answer, Some(false));
    }

    #[test]
    #[ignore = "hits the real LeetCode API — run manually with `cargo test -- --ignored --nocapture`"]
    fn run_testcases_reports_compile_error_for_invalid_syntax() {
        let client = live_client();
        let question = two_sum_question(&client);

        let solution = ParsedSolution {
            lang: LangSlug::Rust,
            question_id: question.question_id.clone(),
            typed_code: r#"
 impl Solution {
     pub fn two_sum(nums: Vec<i32>, target: i32) -> Vec<i32> {
         shouldn'tcompile
     }
 }
 "#
            .to_string(),
        };

        let result = client
             .run_testcases(&question, &solution)
             .expect("run_testcases should succeed (as an API call — the *code* fails to compile, not the request)");

        assert_eq!(result.status_msg, "Compile Error");
        assert!(result.compile_error.is_some());
        assert_eq!(result.correct_answer, None);
    }
}
