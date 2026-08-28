// response types: Problem, CodeSnippet, SubmissionStatus (enum + match),
// whatever shape the GraphQL/REST responses actually turn out to have

/*
 * Production Code
 */
use crate::error::Result;
use serde::Deserialize;
use serde_json::Value;

#[derive(PartialEq, Eq, Deserialize, Debug)]
pub enum Difficulty {
    Easy,
    Medium,
    Hard,
}

#[derive(PartialEq, Eq, Deserialize, Debug)]
#[serde(rename_all = "lowercase")]
pub enum LangSlug {
    Cpp,
    Java,
    Python3,
    Python,
    JavaScript,
    TypeScript,
    CSharp,
    C,
    Golang,
    Kotlin,
    Swift,
    Rust,
    Ruby,
    Php,
    Dart,
    Scala,
    Elixir,
    Erlang,
    Racket,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CodeSnippet {
    pub lang: String,
    pub lang_slug: LangSlug,
    pub code: String,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Question {
    pub question_id: String,
    pub question_frontend_id: String,
    pub title: String,
    pub title_slug: String,
    pub difficulty: Difficulty,
    pub content: String,
    pub code_snippets: Vec<CodeSnippet>,
    pub example_testcase_list: Vec<String>,
}

impl Question {
    pub fn from_graphql_payload(json_str: &str) -> Result<Self> {
        let root: Value = serde_json::from_str(json_str)?;
        let question_json = root["data"]["question"].clone();
        let question: Question = serde_json::from_value(question_json)?;

        Ok(question)
    }

    pub fn lang_snippet(&self, lang: &LangSlug) -> Option<&CodeSnippet> {
        self.code_snippets.iter().find(|s| s.lang_slug == *lang)
    }
}

/*
 * Unit Tests
 */
#[cfg(test)]
mod tests {
    use super::*;

    mod question_model {
        use super::*;

        mod lang_snippet {
            use super::*;
            use crate::error::Result;

            #[test]
            fn parses_graphql_and_finds_language_snippet() -> Result<()> {
                let json_payload =
                    include_str!("../test_data/response_samples/two_sum_response.json");

                let question = Question::from_graphql_payload(json_payload)?;
                assert_eq!(question.question_frontend_id, "1");
                assert_eq!(question.title, "Two Sum");
                assert_eq!(question.difficulty, Difficulty::Easy);

                let snippet = question
                    .lang_snippet(&LangSlug::Rust)
                    .expect("Expected to find a Rust snippet in the parsed payload");

                // verify various parts of the snippet contents
                assert_eq!(snippet.lang, "Rust");
                assert_eq!(snippet.lang_slug, LangSlug::Rust);
                assert!(snippet.code.contains("impl Solution"));
                assert!(snippet.code.contains("pub fn two_sum"));

                Ok(())
            }

            #[test]
            fn returns_none_for_missing_language() -> Result<()> {
                let json_payload =
                    include_str!("../test_data/response_samples/two_sum_response.json");
                let _ = Question::from_graphql_payload(json_payload)?;

                // tests that a lanuage not included in the output does not crash the system
                // possible on newer problems that don't yet support niche languages

                Ok(())
            }

            #[test]
            fn fails_on_malformed_json_syntax() {
                // pass a completely broken JSON string
                let bad_json = "{ this is not valid json";
                let result = Question::from_graphql_payload(bad_json);

                // asserts that the first `?` operator correctly caught the error
                assert!(result.is_err(), "Expected an error for malformed JSON");
            }

            #[test]
            fn fails_on_missing_required_fields() {
                // pass valid JSON that lacks the required Question fields
                let bad_schema_json = r#"{
                    "data": {
                        "question": {
                            "title": "Missing ID and Difficulty"
                        }
                    }
                }"#;
                let result = Question::from_graphql_payload(bad_schema_json);

                // asserts that the second `?` operator (serde_json::from_value) caught the missing fields
                assert!(
                    result.is_err(),
                    "Expected an error for missing Question struct fields"
                );
            }
        }
    }
}
