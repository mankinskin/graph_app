use anyhow::{
    Context,
    Result,
};
use reqwest::Client;
use serde::{
    Deserialize,
    Serialize,
};
use std::env;

#[cfg(feature = "embedded-llm")]
use super::candle_server::CandleServer;

/// Trait for AI clients that can analyze code similarity
#[async_trait::async_trait]
pub trait AiClient {
    async fn analyze_code_similarity(
        &self,
        code_snippets: &[CodeSnippet],
        analysis_prompt: &str,
    ) -> Result<SimilarityAnalysis>;

    async fn suggest_refactoring(
        &self,
        code_context: &str,
        analysis_prompt: &str,
    ) -> Result<RefactoringAnalysis>;
}

/// Represents a code snippet for analysis
#[derive(Debug, Clone, Serialize)]
pub struct CodeSnippet {
    pub content: String,
    pub file_path: String,
    pub function_name: String,
    pub line_number: usize,
    pub context: String,
}

/// AI analysis result for code similarity
#[derive(Debug, Deserialize)]
pub struct SimilarityAnalysis {
    pub similar_groups: Vec<SimilarCodeGroup>,
    pub confidence_score: f32,
    pub reasoning: String,
    pub suggested_refactoring: Option<String>,
}

/// Group of similar code snippets identified by AI
#[derive(Debug, Deserialize)]
pub struct SimilarCodeGroup {
    pub snippet_indices: Vec<usize>,
    pub similarity_score: f32,
    pub common_patterns: Vec<String>,
    pub differences: Vec<String>,
}

/// AI-suggested refactoring analysis
#[derive(Debug, Deserialize)]
pub struct RefactoringAnalysis {
    pub suggestions: Vec<RefactoringSuggestion>,
    pub confidence_score: f32,
    pub reasoning: String,
}

#[derive(Debug, Deserialize)]
pub struct RefactoringSuggestion {
    pub suggestion_type: String,
    pub description: String,
    pub affected_functions: Vec<String>,
    pub estimated_benefit: String,
    pub implementation_notes: String,
}

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

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("OpenAI API error: {}", error_text));
        }

        let response_json: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse OpenAI API response")?;

        let content = response_json["choices"][0]["message"]["content"]
            .as_str()
            .context("Invalid response format from OpenAI API")?;

        // Extract JSON from the response (it might have markdown formatting)
        let json_start = content.find('{').unwrap_or(0);
        let json_end =
            content.rfind('}').map(|i| i + 1).unwrap_or(content.len());
        let json_content = &content[json_start..json_end];

        serde_json::from_str(json_content)
            .context("Failed to parse similarity analysis from AI response")
    }

    async fn suggest_refactoring(
        &self,
        code_context: &str,
        analysis_prompt: &str,
    ) -> Result<RefactoringAnalysis> {
        let prompt = format!(
            r#"{analysis_prompt}

Please analyze the following Rust code for refactoring opportunities:

```rust
{code_context}
```

Provide your analysis in the following JSON format:
{{
    "suggestions": [
        {{
            "suggestion_type": "extract_function",
            "description": "Extract common functionality into a shared function",
            "affected_functions": ["func1", "func2"],
            "estimated_benefit": "Reduces code duplication by 30 lines",
            "implementation_notes": "Create a generic function with parameters"
        }}
    ],
    "confidence_score": 0.85,
    "reasoning": "Detailed explanation of the refactoring analysis"
}}
"#
        );

        let request_body = serde_json::json!({
            "model": self.model,
            "messages": [
                {
                    "role": "system",
                    "content": "You are an expert Rust developer specializing in code refactoring. Analyze code for refactoring opportunities and provide structured responses in JSON format."
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

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("OpenAI API error: {}", error_text));
        }

        let response_json: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse OpenAI API response")?;

        let content = response_json["choices"][0]["message"]["content"]
            .as_str()
            .context("Invalid response format from OpenAI API")?;

        // Extract JSON from the response
        let json_start = content.find('{').unwrap_or(0);
        let json_end =
            content.rfind('}').map(|i| i + 1).unwrap_or(content.len());
        let json_content = &content[json_start..json_end];

        serde_json::from_str(json_content)
            .context("Failed to parse refactoring analysis from AI response")
    }
}

/// Anthropic Claude API client for code analysis
pub struct ClaudeClient {
    client: Client,
    api_key: String,
    model: String,
    base_url: String,
}

impl ClaudeClient {
    pub fn new() -> Result<Self> {
        let api_key = env::var("ANTHROPIC_API_KEY").context(
            "Anthropic API key not found in environment. Set ANTHROPIC_API_KEY",
        )?;

        let model = env::var("ANTHROPIC_MODEL")
            .unwrap_or_else(|_| "claude-3-5-sonnet-20241022".to_string());
        let base_url = env::var("ANTHROPIC_BASE_URL")
            .unwrap_or_else(|_| "https://api.anthropic.com/v1".to_string());

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
                .unwrap_or("https://api.anthropic.com/v1".to_string()),
        }
    }
}

#[async_trait::async_trait]
impl AiClient for ClaudeClient {
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
            "max_tokens": 2000,
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ]
        });

        let response = self
            .client
            .post(&format!("{}/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .context("Failed to send request to Anthropic API")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Anthropic API error: {}", error_text));
        }

        let response_json: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse Anthropic API response")?;

        let content = response_json["content"][0]["text"]
            .as_str()
            .context("Invalid response format from Anthropic API")?;

        // Extract JSON from the response
        let json_start = content.find('{').unwrap_or(0);
        let json_end =
            content.rfind('}').map(|i| i + 1).unwrap_or(content.len());
        let json_content = &content[json_start..json_end];

        serde_json::from_str(json_content)
            .context("Failed to parse similarity analysis from AI response")
    }

    async fn suggest_refactoring(
        &self,
        code_context: &str,
        analysis_prompt: &str,
    ) -> Result<RefactoringAnalysis> {
        let prompt = format!(
            r#"{analysis_prompt}

Please analyze the following Rust code for refactoring opportunities:

```rust
{code_context}
```

Provide your analysis in the following JSON format:
{{
    "suggestions": [
        {{
            "suggestion_type": "extract_function",
            "description": "Extract common functionality into a shared function",
            "affected_functions": ["func1", "func2"],
            "estimated_benefit": "Reduces code duplication by 30 lines",
            "implementation_notes": "Create a generic function with parameters"
        }}
    ],
    "confidence_score": 0.85,
    "reasoning": "Detailed explanation of the refactoring analysis"
}}
"#
        );

        let request_body = serde_json::json!({
            "model": self.model,
            "max_tokens": 2000,
            "messages": [
                {
                    "role": "user",
                    "content": prompt
                }
            ]
        });

        let response = self
            .client
            .post(&format!("{}/messages", self.base_url))
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .context("Failed to send request to Anthropic API")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Anthropic API error: {}", error_text));
        }

        let response_json: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse Anthropic API response")?;

        let content = response_json["content"][0]["text"]
            .as_str()
            .context("Invalid response format from Anthropic API")?;

        // Extract JSON from the response
        let json_start = content.find('{').unwrap_or(0);
        let json_end =
            content.rfind('}').map(|i| i + 1).unwrap_or(content.len());
        let json_content = &content[json_start..json_end];

        serde_json::from_str(json_content)
            .context("Failed to parse refactoring analysis from AI response")
    }
}

/// Ollama local LLM client for code analysis
pub struct OllamaClient {
    client: Client,
    model: String,
    base_url: String,
}

impl OllamaClient {
    pub fn new() -> Result<Self> {
        let model = env::var("OLLAMA_MODEL")
            .unwrap_or_else(|_| "codellama:13b".to_string());
        let base_url = env::var("OLLAMA_BASE_URL")
            .unwrap_or_else(|_| "http://localhost:11434".to_string());

        Ok(Self {
            client: Client::new(),
            model,
            base_url,
        })
    }

    pub fn with_config(
        model: String,
        base_url: Option<String>,
    ) -> Self {
        Self {
            client: Client::new(),
            model,
            base_url: base_url.unwrap_or("http://localhost:11434".to_string()),
        }
    }

    async fn test_connection(&self) -> Result<()> {
        // Test if Ollama is running and model is available
        let response = self
            .client
            .get(&format!("{}/api/tags", self.base_url))
            .send()
            .await
            .context("Failed to connect to Ollama server. Make sure Ollama is running.")?;

        if !response.status().is_success() {
            return Err(anyhow::anyhow!(
                "Ollama server returned error: {}",
                response.status()
            ));
        }

        let tags_response: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse Ollama tags response")?;

        // Check if our model is available
        if let Some(models) = tags_response["models"].as_array() {
            let model_available = models.iter().any(|model| {
                model["name"]
                    .as_str()
                    .map(|name| name.starts_with(&self.model))
                    .unwrap_or(false)
            });

            if !model_available {
                return Err(anyhow::anyhow!(
                    "Model '{}' not found in Ollama. Available models: {}",
                    self.model,
                    models
                        .iter()
                        .filter_map(|m| m["name"].as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
        }

        Ok(())
    }
}

#[async_trait::async_trait]
impl AiClient for OllamaClient {
    async fn analyze_code_similarity(
        &self,
        code_snippets: &[CodeSnippet],
        analysis_prompt: &str,
    ) -> Result<SimilarityAnalysis> {
        // Test connection first
        self.test_connection().await?;

        let prompt = format!(
            r#"{analysis_prompt}

Please analyze the following Rust code snippets for similarity and duplication patterns:

{}

Provide your analysis in the following JSON format (respond ONLY with valid JSON):
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
            "prompt": prompt,
            "stream": false,
            "options": {
                "temperature": 0.1,
                "num_predict": 2000
            }
        });

        let response = self
            .client
            .post(&format!("{}/api/generate", self.base_url))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .context("Failed to send request to Ollama API")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Ollama API error: {}", error_text));
        }

        let response_json: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse Ollama API response")?;

        let content = response_json["response"]
            .as_str()
            .context("Invalid response format from Ollama API")?;

        // Extract JSON from the response (it might have extra text)
        let json_start = content.find('{').unwrap_or(0);
        let json_end =
            content.rfind('}').map(|i| i + 1).unwrap_or(content.len());
        let json_content = &content[json_start..json_end];

        serde_json::from_str(json_content)
            .context("Failed to parse similarity analysis from Ollama response")
    }

    async fn suggest_refactoring(
        &self,
        code_context: &str,
        analysis_prompt: &str,
    ) -> Result<RefactoringAnalysis> {
        // Test connection first
        self.test_connection().await?;

        let prompt = format!(
            r#"{analysis_prompt}

Please analyze the following Rust code for refactoring opportunities:

```rust
{code_context}
```

Provide your analysis in the following JSON format (respond ONLY with valid JSON):
{{
    "suggestions": [
        {{
            "suggestion_type": "extract_function",
            "description": "Extract common functionality into a shared function",
            "affected_functions": ["func1", "func2"],
            "estimated_benefit": "Reduces code duplication by 30 lines",
            "implementation_notes": "Create a generic function with parameters"
        }}
    ],
    "confidence_score": 0.85,
    "reasoning": "Detailed explanation of the refactoring analysis"
}}
"#
        );

        let request_body = serde_json::json!({
            "model": self.model,
            "prompt": prompt,
            "stream": false,
            "options": {
                "temperature": 0.1,
                "num_predict": 2000
            }
        });

        let response = self
            .client
            .post(&format!("{}/api/generate", self.base_url))
            .header("Content-Type", "application/json")
            .json(&request_body)
            .send()
            .await
            .context("Failed to send request to Ollama API")?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(anyhow::anyhow!("Ollama API error: {}", error_text));
        }

        let response_json: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse Ollama API response")?;

        let content = response_json["response"]
            .as_str()
            .context("Invalid response format from Ollama API")?;

        // Extract JSON from the response
        let json_start = content.find('{').unwrap_or(0);
        let json_end =
            content.rfind('}').map(|i| i + 1).unwrap_or(content.len());
        let json_content = &content[json_start..json_end];

        serde_json::from_str(json_content).context(
            "Failed to parse refactoring analysis from Ollama response",
        )
    }
}

/// Factory for creating AI clients based on configuration
pub struct AiClientFactory;

impl AiClientFactory {
    pub fn create_openai_client() -> Result<Box<dyn AiClient + Send + Sync>> {
        Ok(Box::new(OpenAiClient::new()?))
    }

    pub fn create_claude_client() -> Result<Box<dyn AiClient + Send + Sync>> {
        Ok(Box::new(ClaudeClient::new()?))
    }

    pub fn create_ollama_client() -> Result<Box<dyn AiClient + Send + Sync>> {
        Ok(Box::new(OllamaClient::new()?))
    }

    pub fn create_ollama_client_with_config(
        base_url: String,
        model: Option<String>,
    ) -> Result<Box<dyn AiClient + Send + Sync>> {
        let model = model.unwrap_or_else(|| "codellama:7b".to_string());
        Ok(Box::new(OllamaClient::with_config(model, Some(base_url))))
    }

    #[cfg(feature = "embedded-llm")]
    pub async fn create_embedded_client() -> Result<Box<dyn AiClient + Send + Sync>> {
        Ok(Box::new(CandleServer::new().await?))
    }

    #[cfg(not(feature = "embedded-llm"))]
    pub fn create_embedded_client() -> Result<Box<dyn AiClient + Send + Sync>> {
        Err(anyhow::anyhow!(
            "Embedded LLM support not compiled. Rebuild with --features embedded-llm to enable."
        ))
    }

    pub fn create_client_from_env() -> Result<Box<dyn AiClient + Send + Sync>> {
        // Try OpenAI/Copilot first, then Claude, then Ollama, finally embedded
        if env::var("OPENAI_API_KEY").is_ok()
            || env::var("COPILOT_API_KEY").is_ok()
        {
            Self::create_openai_client()
        } else if env::var("ANTHROPIC_API_KEY").is_ok() {
            Self::create_claude_client()
        } else {
            // Try Ollama as fallback (no API key needed, just check if server is running)
            match Self::create_ollama_client() {
                Ok(client) => Ok(client),
                Err(_) => {
                    // If Ollama fails, try embedded as last resort
                    #[cfg(feature = "embedded-llm")]
                    {
                        // Since this is not an async function but create_embedded_client is async,
                        // we need to use a different approach or make this function async
                        Err(anyhow::anyhow!(
                            "No AI providers available. Ollama failed and embedded client requires async initialization. Please set up OpenAI or Claude API keys."
                        ))
                    }
                    #[cfg(not(feature = "embedded-llm"))]
                    {
                        Err(anyhow::anyhow!(
                            "No AI providers available. Please set up OpenAI, Claude, Ollama, or compile with embedded LLM support."
                        ))
                    }
                },
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_openai_client_creation() {
        // This will fail unless API key is set, but tests the creation logic
        let result = OpenAiClient::new();
        match result {
            Ok(_client) => {
                // API key was available
                assert!(true);
            },
            Err(e) => {
                // Expected if no API key is set
                assert!(e.to_string().contains("API key not found"));
            },
        }
    }

    #[tokio::test]
    async fn test_claude_client_creation() {
        let result = ClaudeClient::new();
        match result {
            Ok(_client) => {
                // API key was available
                assert!(true);
            },
            Err(e) => {
                // Expected if no API key is set
                assert!(e.to_string().contains("API key not found"));
            },
        }
    }

    #[test]
    fn test_code_snippet_creation() {
        let snippet = CodeSnippet {
            content: "fn test() {}".to_string(),
            file_path: "src/test.rs".to_string(),
            function_name: "test".to_string(),
            line_number: 10,
            context: "Test function".to_string(),
        };

        assert_eq!(snippet.function_name, "test");
        assert_eq!(snippet.line_number, 10);
    }
}
