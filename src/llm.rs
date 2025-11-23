use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::shape::{FeatureDesignInput, FeatureDesignOutput};
use crate::types::{validate, TypeDef, ValidationError};

#[derive(Clone)]
pub struct LlmClient {
    http: Client,
    base_url: String,
    model: String,
    is_ollama: bool,
}

impl LlmClient {
    pub fn new(base_url: String) -> Self {
        Self::new_with_model(base_url, None)
    }

    pub fn new_with_model(base_url: String, model: Option<String>) -> Self {
        // Detect if this is an Ollama endpoint
        let is_ollama = base_url.contains("11434") || base_url.contains("/api/generate");
        
        // Determine the model name
        let model = model.unwrap_or_else(|| {
            std::env::var("OLLAMA_MODEL")
                .unwrap_or_else(|_| "llama3.2:3b".to_string())
        });

        // Create reqwest client with HTTP/1.1 only and no upgrade
        let http = Client::builder()
            .http1_only()
            .no_proxy()
            .build()
            .unwrap_or_else(|_| Client::new());
        
        Self {
            http,
            base_url,
            model,
            is_ollama,
        }
    }

    pub async fn generate_feature_design(
        &self,
        input: &FeatureDesignInput,
        output_schema: &TypeDef,
    ) -> Result<FeatureDesignOutput> {
        let max_retries = 3;
        let mut last_errors: Option<Vec<ValidationError>> = None;

        for _attempt in 0..max_retries {
            let prompt = build_prompt(input, output_schema, last_errors.as_ref());

            let llm_json_text = self.call_llm(&prompt).await?;

            let value: Value = serde_json::from_str(&llm_json_text)
                .map_err(|e| anyhow!("LLM did not return valid JSON: {e}"))?;

            match validate(output_schema, &value) {
                Ok(()) => {
                    let typed: FeatureDesignOutput = serde_json::from_value(value)?;
                    return Ok(typed);
                }
                Err(errors) => {
                    last_errors = Some(errors);
                    continue;
                }
            }
        }

        Err(anyhow!(
            "LLM failed to produce valid output after {} attempts",
            max_retries
        ))
    }

    async fn call_llm(&self, prompt: &str) -> Result<String> {
        if self.is_ollama {
            self.call_ollama(prompt).await
        } else {
            self.call_mock_server(prompt).await
        }
    }

    async fn call_ollama(&self, prompt: &str) -> Result<String> {
        #[derive(Serialize)]
        struct OllamaRequest<'a> {
            model: &'a str,
            prompt: &'a str,
            stream: bool,
        }

        #[derive(Deserialize)]
        struct OllamaResponse {
            response: String,
            #[allow(dead_code)]
            done: bool,
        }

        // Use Ollama's /api/generate endpoint
        let url = if self.base_url.ends_with("/api/generate") {
            self.base_url.clone()
        } else if self.base_url.contains("11434") {
            format!("http://localhost:11434/api/generate")
        } else {
            format!("{}/api/generate", self.base_url.trim_end_matches('/'))
        };

        let resp = self
            .http
            .post(&url)
            .header("Connection", "close")
            .json(&OllamaRequest {
                model: &self.model,
                prompt,
                stream: false,
            })
            .send()
            .await
            .map_err(|e| anyhow!("Ollama HTTP error: {}. URL: {}", e, url))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let error_text = resp.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!("Ollama HTTP error {}: {}", status, error_text));
        }

            let body: OllamaResponse = resp.json().await?;
        // Clean the response - remove markdown code fences if present
        let cleaned = clean_json_response(&body.response);
        Ok(cleaned)
    }

    async fn call_mock_server(&self, prompt: &str) -> Result<String> {
        #[derive(Serialize)]
        struct LlmRequest<'a> {
            prompt: &'a str,
        }

        #[derive(Deserialize)]
        struct LlmResponse {
            output: String,
        }

        // Make request with reqwest (configured for HTTP/1.1 only)
        let resp = self
            .http
            .post(&self.base_url)
            .header("Connection", "close")
            .json(&LlmRequest { prompt })
            .send()
            .await
            .map_err(|e| anyhow!("LLM HTTP error: {}. URL: {}", e, self.base_url))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let error_text = resp.text().await.unwrap_or_else(|_| "Unknown error".to_string());
            return Err(anyhow!("LLM HTTP error {}: {}", status, error_text));
        }

        let body: LlmResponse = resp.json().await?;
        Ok(body.output)
    }
}

/// Clean JSON response from Ollama - removes markdown code fences and extracts JSON
fn clean_json_response(response: &str) -> String {
    let mut cleaned = response.trim();
    
    // Remove markdown code fences (```json ... ``` or ``` ... ```)
    if cleaned.starts_with("```") {
        // Find the first newline after ```
        if let Some(start_idx) = cleaned.find('\n') {
            cleaned = &cleaned[start_idx + 1..];
        } else {
            // No newline, just remove ```
            cleaned = &cleaned[3..];
        }
        
        // Remove trailing ```
        if cleaned.ends_with("```") {
            cleaned = &cleaned[..cleaned.len() - 3];
        }
    }
    
    cleaned.trim().to_string()
}

fn build_prompt(
    input: &FeatureDesignInput,
    output_schema: &TypeDef,
    last_errors: Option<&Vec<ValidationError>>,
) -> String {
    let mut s = String::new();

    s.push_str("You are a system that strictly outputs JSON.\n");
    s.push_str("You must produce a JSON object that matches this schema:\n\n");
    s.push_str(&describe_schema(output_schema, 0));
    s.push_str("\n\nThe JSON must be parseable and not contain comments or explanations.\n");
    s.push_str("Do not wrap it in markdown code fences.\n\n");

    s.push_str("Context:\n");
    s.push_str("- Repo summary: ");
    s.push_str(&input.repo_summary);
    s.push_str("\n- Constraints:\n");
    for c in &input.constraints {
        s.push_str("  - ");
        s.push_str(c);
        s.push('\n');
    }

    if let Some(errors) = last_errors {
        s.push_str("\nYour previous JSON had these problems:\n");
        for e in errors {
            s.push_str("- ");
            s.push_str(&e.to_string());
            s.push('\n');
        }
        s.push_str("\nFix these issues and output ONLY corrected JSON.\n");
    }

    s
}

// Human-readable schema description for the prompt.
fn describe_schema(ty: &TypeDef, indent: usize) -> String {
    use TypeDef::*;
    let mut s = String::new();
    let pad = " ".repeat(indent);

    match ty {
        Text => s.push_str(&format!("{pad}- string\n")),
        Markdown => s.push_str(&format!("{pad}- string (markdown)\n")),
        Number => s.push_str(&format!("{pad}- number\n")),
        Bool => s.push_str(&format!("{pad}- boolean\n")),
        List(inner) => {
            s.push_str(&format!("{pad}- array of:\n"));
            s.push_str(&describe_schema(inner, indent + 2));
        }
        Object(fields) => {
            s.push_str(&format!("{pad}- object with fields:\n"));
            for f in fields {
                s.push_str(&format!("{pad}  - {}: ", f.name));
                match &f.ty {
                    Text => s.push_str("string\n"),
                    Markdown => s.push_str("string (markdown)\n"),
                    Number => s.push_str("number\n"),
                    Bool => s.push_str("boolean\n"),
                    List(inner) => {
                        s.push_str("array of:\n");
                        s.push_str(&describe_schema(inner, indent + 4));
                    }
                    Object(_) => {
                        s.push_str("nested object:\n");
                        s.push_str(&describe_schema(&f.ty, indent + 4));
                    }
                }
            }
        }
    }

    s
}
