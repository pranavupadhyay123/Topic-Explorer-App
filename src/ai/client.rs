use crate::ai::providers;
use crate::db::DbPool;
use log::{error, info};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, OnceLock};

/// Shared HTTP Client for connection pooling (Massive speed boost)
pub fn http_client() -> &'static reqwest::Client {
    static CLIENT: OnceLock<reqwest::Client> = OnceLock::new();
    CLIENT.get_or_init(|| {
        reqwest::Client::builder()
            .pool_idle_timeout(std::time::Duration::from_secs(90))
            .timeout(std::time::Duration::from_secs(300))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new())
    })
}

/// AI configuration loaded from the database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AIConfig {
    pub provider: String,
    pub model: String,
    pub api_key: String,
    pub endpoint: String,
}

/// Load AI configuration from the app_settings table
pub fn get_ai_config(pool: &Arc<DbPool>) -> Result<AIConfig, String> {
    let conn = pool.get().map_err(|e| format!("DB error: {}", e))?;

    let mut stmt = conn
        .prepare("SELECT ai_provider, ai_model, api_key, api_endpoint FROM app_settings WHERE id = 'default'")
        .map_err(|e| format!("Query error: {}", e))?;

    let config = stmt
        .query_row([], |row| {
            Ok(AIConfig {
                provider: row.get::<_, String>(0).unwrap_or_else(|_| "ollama".into()),
                model: row.get::<_, String>(1).unwrap_or_else(|_| "llama3".into()),
                api_key: row.get::<_, String>(2).unwrap_or_default(),
                endpoint: row.get::<_, String>(3).unwrap_or_default(),
            })
        })
        .map_err(|e| format!("Config not found: {}", e))?;

    Ok(config)
}

/// Chat message for AI calls
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: String,
}

/// Call an OpenAI-compatible API (works for OpenAI, Groq, Mistral, Together, OpenRouter, LM Studio, DeepSeek)
pub async fn call_openai_compatible(
    base_url: &str,
    api_key: &str,
    model: &str,
    messages: &[ChatMessage],
) -> Result<String, String> {
    let url = if base_url.contains("/v1") {
        format!("{}/chat/completions", base_url)
    } else {
        format!("{}/v1/chat/completions", base_url)
    };

    let body = serde_json::json!({
        "model": model,
        "messages": messages,
        "temperature": 0.7,
        "max_tokens": 4096,
    });

    let client = http_client(); // Use shared client
    let mut req = client.post(&url).json(&body);
    if !api_key.is_empty() {
        req = req.header("Authorization", format!("Bearer {}", api_key));
    }

    info!("Calling OpenAI-compatible API: {} with model {}", url, model);

    let resp = req
        .send()
        .await
        .map_err(|e| format!("API request failed: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body_text = resp.text().await.unwrap_or_default();
        error!("API error {}: {}", status, body_text);
        return Err(format!("API error {}: {}", status, body_text));
    }

    let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;

    let content = body
        .get("choices")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("message"))
        .and_then(|m| m.get("content"))
        .and_then(|c| c.as_str())
        .unwrap_or("")
        .to_string();

    Ok(content)
}

/// Call the Anthropic API (different format)
pub async fn call_anthropic(
    api_key: &str,
    model: &str,
    messages: &[ChatMessage],
) -> Result<String, String> {
    let url = "https://api.anthropic.com/v1/messages";

    let system_msg = messages
        .iter()
        .find(|m| m.role == "system")
        .map(|m| m.content.clone())
        .unwrap_or_default();

    let filtered_messages: Vec<serde_json::Value> = messages
        .iter()
        .filter(|m| m.role != "system")
        .map(|m| {
            serde_json::json!({
                "role": m.role,
                "content": m.content,
            })
        })
        .collect();

    let mut body = serde_json::json!({
        "model": model,
        "max_tokens": 4096,
        "messages": filtered_messages,
    });

    if !system_msg.is_empty() {
        body["system"] = serde_json::Value::String(system_msg);
    }

    let client = http_client(); // Use shared client
    let resp = client
        .post(url)
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .header("content-type", "application/json")
        .json(&body)
        .send()
        .await
        .map_err(|e| format!("Anthropic request failed: {}", e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body_text = resp.text().await.unwrap_or_default();
        error!("Anthropic error {}: {}", status, body_text);
        return Err(format!("Anthropic error {}: {}", status, body_text));
    }

    let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;

    let content = body
        .get("content")
        .and_then(|c| c.get(0))
        .and_then(|c| c.get("text"))
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .to_string();

    Ok(content)
}

/// Main dispatcher: call the correct AI provider
pub async fn call_ai(config: &AIConfig, messages: &[ChatMessage]) -> Result<String, String> {
    let provider = providers::get_provider(&config.provider)
        .ok_or_else(|| format!("Unknown provider: {}", config.provider))?;

    let base_url = if config.endpoint.is_empty() {
        provider.base_url.clone()
    } else {
        config.endpoint.clone()
    };

    match provider.api_format.as_str() {
        "anthropic" => call_anthropic(&config.api_key, &config.model, messages).await,
        _ => call_openai_compatible(&base_url, &config.api_key, &config.model, messages).await,
    }
}