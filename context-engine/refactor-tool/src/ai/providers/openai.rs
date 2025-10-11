use anyhow::{Context, Result};
use reqwest::Client;
use serde_json;
use std::env;

use crate::ai::client::{AiClient, CodeSnippet, SimilarityAnalysis, RefactoringAnalysis};

/// OpenAI API client for code analysis
pub struct OpenAiClient {
    client: Client,
    api_key: String,
    model: String,
    base_url: String,
}

impl OpenAiClient {
    pub fn new() -> Result<Self> {
        let api_key = env::var("OPENAI_API_KEY")
            .or_else(|_| env::var("COPILOT_API_KEY"))
            .context("OpenAI/Copilot API key not found in environment. Set OPENAI_API_KEY or COPILOT_API_KEY")?;

        let model =
            env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4".to_string());
        let base_url = env::var("OPENAI_BASE_URL")
            .unwrap_or_else(|_| "https://api.openai.com/v1".to_string());

        Ok(Self {
            client: Client::new(),
            api_key,
            model,
            base_url,
        })
    }

    pub fn with_config(
        api_key: String,
        model: String,
        base_url: Option<String>,
    ) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model,
            base_url: base_url
                .unwrap_or("https://api.openai.com/v1".to_string()),
        }
    }
}

#[async_trait::async_trait]
impl AiClient for OpenAiClient {
    async fn analyze_code_similarity(
        &self,
        code_snippets: &[CodeSnippet],
        analysis_prompt: &str,
    ) -> Result<SimilarityAnalysis> {
        let prompt = format!(
            r#"{analysis_prompt}

Please analyze the following Rust code snippets for similarity and duplication patterns:

{}

Provide your analysis in the following JSON format:
{{
    "similar_groups": [
        {{
            "snippet_indices": [0, 1],
            "similarity_score": 0.8,
            "common_patterns": ["similar control flow", "shared functionality"],
            "differences": ["parameter types", "return values"]
        }}
    ],
    "confidence_score": 0.9,
    "reasoning": "Detailed explanation of the analysis",
    "suggested_refactoring": "Optional refactoring suggestion"
}}
"#,
            code_snippets
                .iter()
                .enumerate()
                .map(|(i, snippet)| format!(
                    "## Snippet {} ({}:{})\n```rust\n{}\n```\n",
                    i, snippet.file_path, snippet.line_number, snippet.content
                ))
                .collect::<Vec<_>>()
                .join("\n")
        );

        let request_body = serde_json::json!({
            "model": self.model,
            "messages": [
                {
                    "role": "system",
                    "content": "You are an expert Rust developer specializing in code analysis and refactoring. Analyze code for similarity patterns and provide structured responses in JSON format."
                },
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "temperature": 0.1,
            "max_tokens": 2000
        });

        let response = self
            .client
            .post(&format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .context("Failed to send request to OpenAI API")?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .context("Failed to read OpenAI API response")?;

        if !status.is_success() {
            return Err(anyhow::anyhow!(
                "OpenAI API request failed with status {}: {}",
                status,
                response_text
            ));
        }

        let api_response: serde_json::Value = serde_json::from_str(&response_text)
            .context("Failed to parse OpenAI API response as JSON")?;

        let content = api_response["choices"][0]["message"]["content"]
            .as_str()
            .context("No content in OpenAI API response")?;

        // Try to extract JSON from the response
        let json_start = content.find('{').unwrap_or(0);
        let json_end = content.rfind('}').map(|i| i + 1).unwrap_or(content.len());
        let json_content = &content[json_start..json_end];

        serde_json::from_str(json_content)
            .context("Failed to parse AI analysis response as JSON")
    }

    async fn suggest_refactoring(
        &self,
        code_context: &str,
        analysis_prompt: &str,
    ) -> Result<RefactoringAnalysis> {
        let prompt = format!(
            r#"{analysis_prompt}

Code context:
```rust
{}
```

Please suggest specific refactoring improvements for this Rust code. Provide your analysis in JSON format:
{{
    "suggestions": [
        {{
            "suggestion_type": "extract_function",
            "description": "Extract common logic into a shared function",
            "affected_functions": ["func1", "func2"],
            "estimated_benefit": "Reduced duplication, improved maintainability",
            "implementation_notes": "Create new function with common parameters"
        }}
    ],
    "overall_assessment": "General code quality assessment",
    "reasoning": "Detailed reasoning for suggestions"
}}
"#,
            code_context
        );

        let request_body = serde_json::json!({
            "model": self.model,
            "messages": [
                {
                    "role": "system",
                    "content": "You are an expert Rust developer. Analyze code and suggest specific refactoring improvements. Focus on reducing duplication, improving readability, and following Rust best practices."
                },
                {
                    "role": "user",
                    "content": prompt
                }
            ],
            "temperature": 0.1,
            "max_tokens": 1500
        });

        let response = self
            .client
            .post(&format!("{}/chat/completions", self.base_url))
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .context("Failed to send refactoring request to OpenAI API")?;

        let status = response.status();
        let response_text = response
            .text()
            .await
            .context("Failed to read OpenAI refactoring response")?;

        if !status.is_success() {
            return Err(anyhow::anyhow!(
                "OpenAI refactoring request failed with status {}: {}",
                status,
                response_text
            ));
        }

        let api_response: serde_json::Value = serde_json::from_str(&response_text)
            .context("Failed to parse OpenAI refactoring response as JSON")?;

        let content = api_response["choices"][0]["message"]["content"]
            .as_str()
            .context("No content in OpenAI refactoring response")?;

        // Try to extract JSON from the response
        let json_start = content.find('{').unwrap_or(0);
        let json_end = content.rfind('}').map(|i| i + 1).unwrap_or(content.len());
        let json_content = &content[json_start..json_end];

        serde_json::from_str(json_content)
            .context("Failed to parse refactoring analysis response as JSON")
    }
}