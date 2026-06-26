use serde::{Deserialize, Serialize};

/// Configuration for an AI provider
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderConfig {
    pub id: String,
    pub name: String,
    pub description: String,
    pub base_url: String,
    pub api_key_env: String,
    pub api_key_required: bool,
    pub models_endpoint: String,
    pub default_models: Vec<String>,
    pub supports_streaming: bool,
    pub api_format: String, // "openai" | "anthropic"
}

/// Information about a specific model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub provider: String,
}

/// Get all supported providers
pub fn get_all_providers() -> Vec<ProviderConfig> {
    vec![
        ProviderConfig {
            id: "ollama".into(),
            name: "Ollama".into(),
            description: "Run models locally with Ollama".into(),
            base_url: "http://localhost:11434".into(),
            api_key_env: "".into(),
            api_key_required: false,
            models_endpoint: "/api/tags".into(),
            default_models: vec!["llama3".into(), "llama3:8b".into(), "mistral".into(), "codellama".into(), "gemma".into()],
            supports_streaming: true,
            api_format: "openai".into(),
        },
        ProviderConfig {
            id: "openai".into(),
            name: "OpenAI".into(),
            description: "GPT-4o, GPT-4, GPT-3.5 Turbo".into(),
            base_url: "https://api.openai.com/v1".into(),
            api_key_env: "OPENAI_API_KEY".into(),
            api_key_required: true,
            models_endpoint: "/models".into(),
            default_models: vec!["gpt-4o".into(), "gpt-4o-mini".into(), "gpt-4-turbo".into(), "gpt-3.5-turbo".into()],
            supports_streaming: true,
            api_format: "openai".into(),
        },
        ProviderConfig {
            id: "anthropic".into(),
            name: "Anthropic".into(),
            description: "Claude 3.5, Claude 3 models".into(),
            base_url: "https://api.anthropic.com".into(),
            api_key_env: "ANTHROPIC_API_KEY".into(),
            api_key_required: true,
            models_endpoint: "".into(),
            default_models: vec!["claude-sonnet-4-20250514".into(), "claude-3-5-sonnet-20241022".into(), "claude-3-haiku-20240307".into(), "claude-3-opus-20240229".into()],
            supports_streaming: true,
            api_format: "anthropic".into(),
        },
        ProviderConfig {
            id: "groq".into(),
            name: "Groq".into(),
            description: "Ultra-fast inference with Groq".into(),
            base_url: "https://api.groq.com/openai/v1".into(),
            api_key_env: "GROQ_API_KEY".into(),
            api_key_required: true,
            models_endpoint: "/models".into(),
            default_models: vec!["llama-3.1-70b-versatile".into(), "llama-3.1-8b-instant".into(), "mixtral-8x7b-32768".into(), "gemma2-9b-it".into()],
            supports_streaming: true,
            api_format: "openai".into(),
        },
        ProviderConfig {
            id: "mistral".into(),
            name: "Mistral AI".into(),
            description: "Mistral, Mixtral models".into(),
            base_url: "https://api.mistral.ai/v1".into(),
            api_key_env: "MISTRAL_API_KEY".into(),
            api_key_required: true,
            models_endpoint: "/models".into(),
            default_models: vec!["mistral-large-latest".into(), "mistral-medium-latest".into(), "mistral-small-latest".into(), "open-mistral-7b".into()],
            supports_streaming: true,
            api_format: "openai".into(),
        },
        ProviderConfig {
            id: "together".into(),
            name: "Together AI".into(),
            description: "Open-source models on Together".into(),
            base_url: "https://api.together.xyz/v1".into(),
            api_key_env: "TOGETHER_API_KEY".into(),
            api_key_required: true,
            models_endpoint: "/models".into(),
            default_models: vec!["meta-llama/Llama-3-70b-chat-hf".into(), "mistralai/Mixtral-8x7B-Instruct-v0.1".into()],
            supports_streaming: true,
            api_format: "openai".into(),
        },
        ProviderConfig {
            id: "openrouter".into(),
            name: "OpenRouter".into(),
            description: "Access any model via OpenRouter".into(),
            base_url: "https://openrouter.ai/api/v1".into(),
            api_key_env: "OPENROUTER_API_KEY".into(),
            api_key_required: true,
            models_endpoint: "/models".into(),
            default_models: vec!["openai/gpt-4o".into(), "anthropic/claude-3.5-sonnet".into(), "meta-llama/llama-3-70b-instruct".into()],
            supports_streaming: true,
            api_format: "openai".into(),
        },
        ProviderConfig {
            id: "lmstudio".into(),
            name: "LM Studio".into(),
            description: "Run models locally with LM Studio".into(),
            base_url: "http://localhost:1234/v1".into(),
            api_key_env: "".into(),
            api_key_required: false,
            models_endpoint: "/models".into(),
            default_models: vec![],
            supports_streaming: true,
            api_format: "openai".into(),
        },
        ProviderConfig {
            id: "deepseek".into(),
            name: "DeepSeek".into(),
            description: "DeepSeek Coder, DeepSeek Chat".into(),
            base_url: "https://api.deepseek.com/v1".into(),
            api_key_env: "DEEPSEEK_API_KEY".into(),
            api_key_required: true,
            models_endpoint: "/models".into(),
            default_models: vec!["deepseek-chat".into(), "deepseek-coder".into()],
            supports_streaming: true,
            api_format: "openai".into(),
        },
        ProviderConfig {
            id: "nvidia".into(),
            name: "NVIDIA".into(),
            description: "NVIDIA NIM (LLaMA 3, Nemotron, etc.)".into(),
            base_url: "https://integrate.api.nvidia.com/v1".into(),
            api_key_env: "NVIDIA_API_KEY".into(),
            api_key_required: true,
            models_endpoint: "/models".into(),
            default_models: vec!["meta/llama3-70b-instruct".into(), "nvidia/nemotron-4-340b-instruct".into(), "mistralai/mixtral-8x22b-instruct-v0.1".into()],
            supports_streaming: true,
            api_format: "openai".into(),
        },
    ]
}

/// Find a provider by ID
pub fn get_provider(provider_id: &str) -> Option<ProviderConfig> {
    get_all_providers().into_iter().find(|p| p.id == provider_id)
}

/// Fetch available models from a provider
pub async fn fetch_models_from_provider(
    provider_id: &str,
    api_key: &str,
    custom_endpoint: &str,
) -> Result<Vec<ModelInfo>, String> {
    let provider = get_provider(provider_id).ok_or_else(|| format!("Unknown provider: {}", provider_id))?;

    if provider_id == "ollama" {
        return fetch_ollama_models(custom_endpoint).await;
    }

    if provider_id == "anthropic" {
        // Anthropic doesn't have a models endpoint; return defaults
        return Ok(provider
            .default_models
            .iter()
            .map(|m| ModelInfo {
                id: m.clone(),
                name: m.clone(),
                provider: provider_id.into(),
            })
            .collect());
    }

    let base = if custom_endpoint.is_empty() {
        &provider.base_url
    } else {
        custom_endpoint
    };

    let url = format!("{}{}", base, provider.models_endpoint);

    let client = reqwest::Client::new();
    let mut req = client.get(&url);
    if !api_key.is_empty() {
        req = req.header("Authorization", format!("Bearer {}", api_key));
    }

    let resp = req.send().await.map_err(|e| format!("Request failed: {}", e))?;

    if !resp.status().is_success() {
        // Fall back to default models
        return Ok(provider
            .default_models
            .iter()
            .map(|m| ModelInfo {
                id: m.clone(),
                name: m.clone(),
                provider: provider_id.into(),
            })
            .collect());
    }

    let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;

    let models = if let Some(data) = body.get("data").and_then(|d| d.as_array()) {
        data.iter()
            .filter_map(|m| {
                let id = m.get("id")?.as_str()?.to_string();
                Some(ModelInfo {
                    name: id.clone(),
                    id,
                    provider: provider_id.into(),
                })
            })
            .collect()
    } else {
        provider
            .default_models
            .iter()
            .map(|m| ModelInfo {
                id: m.clone(),
                name: m.clone(),
                provider: provider_id.into(),
            })
            .collect()
    };

    Ok(models)
}

/// Fetch models from Ollama
async fn fetch_ollama_models(custom_endpoint: &str) -> Result<Vec<ModelInfo>, String> {
    let base = if custom_endpoint.is_empty() {
        "http://localhost:11434"
    } else {
        custom_endpoint
    };

    let url = format!("{}/api/tags", base);

    let client = reqwest::Client::new();
    let resp = client
        .get(&url)
        .timeout(std::time::Duration::from_secs(5))
        .send()
        .await
        .map_err(|e| format!("Ollama not reachable: {}", e))?;

    let body: serde_json::Value = resp.json().await.map_err(|e| format!("Parse error: {}", e))?;

    let models = if let Some(models_arr) = body.get("models").and_then(|m| m.as_array()) {
        models_arr
            .iter()
            .filter_map(|m| {
                let name = m.get("name")?.as_str()?.to_string();
                Some(ModelInfo {
                    id: name.clone(),
                    name,
                    provider: "ollama".into(),
                })
            })
            .collect()
    } else {
        vec![]
    };

    Ok(models)
}

/// Test connection to a provider
pub async fn test_provider_connection(
    provider_id: &str,
    api_key: &str,
    custom_endpoint: &str,
) -> Result<bool, String> {
    match fetch_models_from_provider(provider_id, api_key, custom_endpoint).await {
        Ok(models) => Ok(!models.is_empty()),
        Err(e) => Err(e),
    }
}
