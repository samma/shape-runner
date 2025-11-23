use anyhow::{anyhow, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::shape::{FeatureDesignInput, FeatureDesignOutput, FormationInput, FormationOutput};
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
        let mut last_json_error: Option<String> = None;

        for attempt in 0..max_retries {
            eprintln!("[DEMO] Attempt {} of {}", attempt + 1, max_retries);
            if let Some(ref errors) = last_errors {
                eprintln!("[DEMO] Previous validation errors:");
                for err in errors {
                    eprintln!("[DEMO]   - {}", err);
                }
            }
            if let Some(ref json_err) = last_json_error {
                eprintln!("[DEMO] Previous JSON parse error: {}", json_err);
            }
            
            let prompt = build_prompt(input, output_schema, last_errors.as_ref(), last_json_error.as_deref());

            let llm_json_text = self.call_llm(&prompt).await?;
            
            // Log the raw response for debugging (first 500 chars)
            if attempt == 0 {
                let preview = if llm_json_text.len() > 500 {
                    format!("{}...", &llm_json_text[..500])
                } else {
                    llm_json_text.clone()
                };
                eprintln!("[DEMO] LLM raw response (first 500 chars):\n{}", preview);
            }

            // Try to parse JSON - retry if it fails
            let value: Value = match serde_json::from_str(&llm_json_text) {
                Ok(v) => {
                    v
                }
                Err(e) => {
                    let error_msg = format!("{}", e);
                    eprintln!("[DEMO] JSON parse error: {}", error_msg);
                    eprintln!("[DEMO] Response length: {}, First 200 chars: {}", 
                        llm_json_text.len(),
                        if llm_json_text.len() > 200 { &llm_json_text[..200] } else { &llm_json_text }
                    );
                    
                    // If this is the last attempt, return error
                    if attempt == max_retries - 1 {
                        return Err(anyhow!("LLM did not return valid JSON after {} attempts. Last error: {}", max_retries, error_msg));
                    }
                    
                    // Otherwise, save error and retry
                    last_json_error = Some(error_msg);
                    last_errors = None; // Clear validation errors since we didn't get that far
                    if attempt < max_retries - 1 {
                        eprintln!("[DEMO] Retrying with JSON error feedback...\n");
                    }
                    continue;
                }
            };

            match validate(output_schema, &value) {
                Ok(()) => {
                    eprintln!("[DEMO] ✓ Validation passed! Returning result.");
                    let typed: FeatureDesignOutput = serde_json::from_value(value)?;
                    return Ok(typed);
                }
                Err(errors) => {
                    eprintln!("[DEMO] ✗ Validation failed with {} error(s)", errors.len());
                    last_errors = Some(errors);
                    last_json_error = None; // Clear JSON error since JSON was valid
                    if attempt < max_retries - 1 {
                        eprintln!("[DEMO] Retrying...\n");
                    }
                    continue;
                }
            }
        }

        Err(anyhow!(
            "LLM failed to produce valid output after {} attempts",
            max_retries
        ))
    }

    pub async fn generate_formation(
        &self,
        input: &FormationInput,
        output_schema: &TypeDef,
    ) -> Result<FormationOutput> {
        let max_retries = 3;
        let mut last_errors: Option<Vec<ValidationError>> = None;
        let mut last_json_error: Option<String> = None;

        for attempt in 0..max_retries {
            eprintln!("[DEMO] Formation attempt {} of {}", attempt + 1, max_retries);
            if let Some(ref errors) = last_errors {
                eprintln!("[DEMO] Previous validation errors:");
                for err in errors {
                    eprintln!("[DEMO]   - {}", err);
                }
            }
            if let Some(ref json_err) = last_json_error {
                eprintln!("[DEMO] Previous JSON parse error: {}", json_err);
            }
            
            let prompt = build_formation_prompt(input, output_schema, last_errors.as_ref(), last_json_error.as_deref());

            let llm_json_text = self.call_llm(&prompt).await?;
            
            // Try to parse JSON - retry if it fails
            let value: Value = match serde_json::from_str(&llm_json_text) {
                Ok(v) => {
                    v
                }
                Err(e) => {
                    let error_msg = format!("{}", e);
                    eprintln!("[DEMO] JSON parse error: {}", error_msg);
                    
                    // If this is the last attempt, return error
                    if attempt == max_retries - 1 {
                        return Err(anyhow!("LLM did not return valid JSON after {} attempts. Last error: {}", max_retries, error_msg));
                    }
                    
                    // Otherwise, save error and retry
                    last_json_error = Some(error_msg);
                    last_errors = None; // Clear validation errors since we didn't get that far
                    if attempt < max_retries - 1 {
                        eprintln!("[DEMO] Retrying with JSON error feedback...\n");
                    }
                    continue;
                }
            };

            match validate(output_schema, &value) {
                Ok(()) => {
                    eprintln!("[DEMO] ✓ Schema validation passed!");
                    let typed: FormationOutput = serde_json::from_value(value)?;
                    
                    // Validate that we have the correct number of coordinates
                    if typed.coordinates.len() != input.unit_count as usize {
                        eprintln!("[DEMO] ✗ Coordinate count mismatch: expected {}, got {}", 
                            input.unit_count, typed.coordinates.len());
                        // Create a validation-like error to trigger retry
                        let mut errors = Vec::new();
                        errors.push(ValidationError::TypeMismatch {
                            path: "$.coordinates".to_string(),
                            expected: format!("array with exactly {} items", input.unit_count),
                            found: format!("array with {} items", typed.coordinates.len()),
                        });
                        last_errors = Some(errors);
                        last_json_error = None;
                        if attempt < max_retries - 1 {
                            eprintln!("[DEMO] Retrying with coordinate count feedback...\n");
                        }
                        continue;
                    }
                    
                    eprintln!("[DEMO] ✓ All validation passed! Returning result.");
                    return Ok(typed);
                }
                Err(errors) => {
                    eprintln!("[DEMO] ✗ Validation failed with {} error(s)", errors.len());
                    last_errors = Some(errors);
                    last_json_error = None; // Clear JSON error since JSON was valid
                    if attempt < max_retries - 1 {
                        eprintln!("[DEMO] Retrying...\n");
                    }
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
    
    cleaned = cleaned.trim();
    
    // Try to find JSON object boundaries if there's extra text
    // Look for the first { and last } - be more aggressive about finding complete JSON
    if let Some(first_brace) = cleaned.find('{') {
        // Find the matching closing brace by counting braces
        let mut brace_count = 0;
        let mut last_brace = None;
        for (i, c) in cleaned[first_brace..].char_indices() {
            match c {
                '{' => brace_count += 1,
                '}' => {
                    brace_count -= 1;
                    if brace_count == 0 {
                        last_brace = Some(first_brace + i);
                        break;
                    }
                }
                _ => {}
            }
        }
        
        if let Some(end_pos) = last_brace {
            cleaned = &cleaned[first_brace..=end_pos];
        } else if let Some(fallback_brace) = cleaned.rfind('}') {
            // Fallback to simple rfind if brace counting fails
            if fallback_brace > first_brace {
                cleaned = &cleaned[first_brace..=fallback_brace];
            }
        }
    }
    
    // Aggressively filter out control characters that break JSON parsing
    // Control characters in JSON strings must be escaped (like \n, \t), but raw ones break parsing
    let cleaned_str: String = cleaned
        .chars()
        .filter_map(|c| {
            match c {
                // Remove null bytes and other problematic control chars completely
                '\u{0000}'..='\u{001F}' => {
                    // Replace with escaped version if it's a common one, otherwise skip
                    match c {
                        '\n' => Some(' '),  // Replace newline with space
                        '\t' => Some(' '),  // Replace tab with space
                        '\r' => None,       // Remove carriage return
                        _ => None,          // Remove other control chars
                    }
                }
                // Keep all printable characters
                _ => Some(c),
            }
        })
        .collect();
    
    // Remove trailing commas before } or ] (common LLM mistake)
    let mut result = cleaned_str.trim().to_string();
    result = result.replace(",}", "}");
    result = result.replace(",]", "]");
    // Also handle cases with whitespace: ", }" -> "}"
    result = result.replace(", }", "}");
    result = result.replace(", ]", "]");
    
    result.trim().to_string()
}

fn build_prompt(
    input: &FeatureDesignInput,
    output_schema: &TypeDef,
    last_errors: Option<&Vec<ValidationError>>,
    last_json_error: Option<&str>,
) -> String {
    let mut s = String::new();

    s.push_str("You are a system that strictly outputs JSON.\n");
    s.push_str("You must produce a JSON object that matches this schema:\n\n");
    s.push_str(&describe_schema(output_schema, 0));
    s.push_str("\n\nThe JSON must be parseable and not contain comments or explanations.\n");
    s.push_str("Do not wrap it in markdown code fences.\n");
    s.push_str("Do not include control characters (null bytes, etc.) in your output.\n");
    s.push_str("Escape special characters properly in JSON strings (use \\n for newlines, etc.).\n\n");

    s.push_str("Context:\n");
    s.push_str("- Repo summary: ");
    s.push_str(&input.repo_summary);
    s.push_str("\n- Constraints:\n");
    for c in &input.constraints {
        s.push_str("  - ");
        s.push_str(c);
        s.push('\n');
    }

    if let Some(json_err) = last_json_error {
        s.push_str("\nYour previous response was not valid JSON. The error was:\n");
        s.push_str(json_err);
        s.push_str("\n\nPlease output ONLY valid, parseable JSON without any control characters or formatting issues.\n");
    }

    if let Some(errors) = last_errors {
        s.push_str("\nYour previous JSON had these validation problems:\n");
        for e in errors {
            s.push_str("- ");
            s.push_str(&e.to_string());
            s.push('\n');
        }
        s.push_str("\nFix these issues and output ONLY corrected JSON.\n");
    }

    s
}

fn build_formation_prompt(
    input: &FormationInput,
    output_schema: &TypeDef,
    last_errors: Option<&Vec<ValidationError>>,
    last_json_error: Option<&str>,
) -> String {
    let mut s = String::new();

    s.push_str("You are a system that strictly outputs JSON.\n");
    s.push_str("You must produce a JSON object that matches this schema:\n\n");
    s.push_str(&describe_schema(output_schema, 0));
    s.push_str("\n\nThe JSON must be parseable and not contain comments or explanations.\n");
    s.push_str("Do not wrap it in markdown code fences.\n");
    s.push_str("Do not include control characters (null bytes, etc.) in your output.\n");
    s.push_str("Escape special characters properly in JSON strings (use \\n for newlines, etc.).\n\n");

    s.push_str("Task: Generate 2D coordinates for unit formation.\n");
    s.push_str(&format!("- Formation description: {}\n", input.formation_description));
    s.push_str(&format!("- Number of units: {}\n", input.unit_count));
    s.push_str("\n");
    s.push_str("CRITICAL: You MUST generate EXACTLY ");
    s.push_str(&input.unit_count.to_string());
    s.push_str(" coordinates (x, y pairs), no more, no less.\n");
    s.push_str("The coordinates array must contain exactly ");
    s.push_str(&input.unit_count.to_string());
    s.push_str(" items.\n");
    s.push_str("Coordinates should be reasonable 2D positions (typically between 0-100 for x and y).\n");
    s.push_str("The formation should be visually recognizable as the requested shape.\n");
    s.push_str("\n");
    s.push_str("Example output format (for 3 units):\n");
    s.push_str("{\"coordinates\":[{\"x\":0.0,\"y\":0.0},{\"x\":10.0,\"y\":0.0},{\"x\":5.0,\"y\":10.0}]}\n");
    s.push_str("\n");
    s.push_str("CRITICAL: Output ONLY the JSON object, nothing else. No text before or after. No markdown. No explanations.\n");
    s.push_str("The JSON must be valid and parseable. Do NOT include:\n");
    s.push_str("- Control characters (null bytes, etc.)\n");
    s.push_str("- Unescaped newlines or tabs inside JSON strings\n");
    s.push_str("- Any characters outside the JSON structure\n");
    s.push_str("- Trailing commas\n");

    if let Some(json_err) = last_json_error {
        s.push_str("\nYour previous response was not valid JSON. The error was:\n");
        s.push_str(json_err);
        s.push_str("\n\nPlease output ONLY valid, parseable JSON without any control characters or formatting issues.\n");
    }

    if let Some(errors) = last_errors {
        s.push_str("\nYour previous JSON had these validation problems:\n");
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
