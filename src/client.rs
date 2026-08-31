// LeetCodeClient: wraps an HTTP client, talks to leetcode.com/graphql
// (unauthenticated: fetch problem) and the REST run/submit/check endpoints
// (authenticated: needs the session cookie from config.rs)

use crate::config::Config;
use crate::error::{LeetCodeError, Result};
use crate::models::Question;

const LEETCODE_GRAPHQL_ENDPOINT: &str = "https://leetcode.com/graphql/";

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

    pub fn fetch_problem(&self, slug: &str) -> Result<Question> {
        let query = "query fetchProblem($titleSlug: String!) {
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
        let variables = serde_json::json!({
            "titleSlug": slug
        });
        let ret = self.query_graphql(query, variables)?;
        Ok(Question::from_graphql_value(&ret, slug)?)
    }
}

/*
 * ========== UNIT TESTS ==========
 */

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Difficulty, LangSlug};

    fn test_client() -> LeetCodeClient {
        LeetCodeClient::new().expect("client should construct")
    }

    #[test]
    #[ignore = "hits the real LeetCode API -- run manually with `cargo test -- --ignored`"]
    fn fetches_two_sum_from_live_api() {
        let client = test_client();
        let question = client
            .fetch_problem("two-sum")
            .expect("should fetch two-sum");

        assert_eq!(question.question_frontend_id, "1");
        assert_eq!(question.title, "Two Sum");
        assert_eq!(question.difficulty, Difficulty::Easy);
        assert!(question.lang_snippet(&LangSlug::Rust).is_some());
    }

    #[test]
    #[ignore = "hits the real LeetCode API -- run manually with `cargo test -- --ignored`"]
    fn fetches_a_second_known_problem() {
        let client = test_client();
        let question = client
            .fetch_problem("valid-parentheses")
            .expect("should fetch valid-parentheses");

        assert_eq!(question.question_frontend_id, "20");
        assert_eq!(question.title, "Valid Parentheses");
    }

    #[test]
    #[ignore = "hits the real LeetCode API -- run manually with `cargo test -- --ignored`"]
    fn fetching_nonexistent_slug_returns_problem_not_found() {
        let client = test_client();
        let result = client.fetch_problem("this-problem-definitely-does-not-exist-xyz");

        match result {
            Err(LeetCodeError::ProblemNotFound(slug)) => {
                assert_eq!(slug, "this-problem-definitely-does-not-exist-xyz");
            }
            other => panic!("expected ProblemNotFound, got {other:?}"),
        }
    }
}
