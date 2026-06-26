use log::warn;
use serde_json::Value;

/// Extract a JSON object from AI response text (handles markdown fences, extra text)
pub fn extract_json(text: &str) -> Option<Value> {
    // Try parsing directly first
    if let Ok(val) = serde_json::from_str::<Value>(text) {
        if val.is_object() {
            return Some(val);
        }
    }

    // Try extracting from markdown code fences
    let cleaned = strip_code_fences(text);
    if let Ok(val) = serde_json::from_str::<Value>(&cleaned) {
        if val.is_object() {
            return Some(val);
        }
    }

    let sanitized = sanitize_ai_json(&cleaned);
    if let Ok(val) = serde_json::from_str::<Value>(&sanitized) {
        if val.is_object() {
            return Some(val);
        }
    }

    // Try finding JSON object between { and }
    if let Some(start) = text.find('{') {
        if let Some(end) = text.rfind('}') {
            let candidate = &text[start..=end];
            let sanitized_candidate = sanitize_ai_json(candidate);
            if let Ok(val) = serde_json::from_str::<Value>(&sanitized_candidate) {
                if val.is_object() {
                    return Some(val);
                }
            }
        }
    }

    warn!("Failed to extract JSON object from AI response");
    None
}

/// Extract a JSON array from AI response text
pub fn extract_json_array(text: &str) -> Option<Value> {
    // Try parsing directly
    if let Ok(val) = serde_json::from_str::<Value>(text) {
        if val.is_array() {
            return Some(val);
        }
    }

    let cleaned = strip_code_fences(text);
    if let Ok(val) = serde_json::from_str::<Value>(&cleaned) {
        if val.is_array() {
            return Some(val);
        }
    }

    let sanitized = sanitize_ai_json(&cleaned);
    if let Ok(val) = serde_json::from_str::<Value>(&sanitized) {
        if val.is_array() {
            return Some(val);
        }
    }

    // Try finding JSON array between [ and ]
    if let Some(start) = text.find('[') {
        if let Some(end) = text.rfind(']') {
            let candidate = &text[start..=end];
            let sanitized_candidate = sanitize_ai_json(candidate);
            if let Ok(val) = serde_json::from_str::<Value>(&sanitized_candidate) {
                if val.is_array() {
                    return Some(val);
                }
            }
        }
    }

    warn!("Failed to extract JSON array from AI response");
    None
}

/// Strip markdown code fences from text
fn strip_code_fences(text: &str) -> String {
    let cleaned = text.trim();

    // Look for ``` block and extract everything inside it
    if let Some(start_idx) = cleaned.find("```") {
        let after_start = &cleaned[start_idx + 3..];
        let content_start = if let Some(newline_pos) = after_start.find('\n') {
            newline_pos + 1
        } else {
            0
        };
        let inner = &after_start[content_start..];
        
        // Find the matching end fence
        if let Some(end_idx) = inner.find("```") {
            return inner[..end_idx].trim().to_string();
        }
    }

    cleaned.to_string()
}

/// Sanitize common JSON errors from AI responses
pub fn sanitize_ai_json(text: &str) -> String {
    let mut result = text.to_string();

    // Remove trailing commas before } or ]
    loop {
        let prev = result.clone();
        
        let mut new_result = String::with_capacity(result.len());
        let chars: Vec<char> = result.chars().collect();
        let mut i = 0;
        while i < chars.len() {
            if chars[i] == ',' {
                let mut j = i + 1;
                while j < chars.len() && chars[j].is_whitespace() {
                    j += 1;
                }
                if j < chars.len() && (chars[j] == '}' || chars[j] == ']') {
                    // Skip the comma by not pushing it
                    i += 1;
                    continue;
                }
            }
            new_result.push(chars[i]);
            i += 1;
        }
        result = new_result;

        if result == prev {
            break;
        }
    }

    if serde_json::from_str::<Value>(&result).is_err() {
        result = attempt_fix_quotes(&result);
    }

    result
}

/// Attempt to fix single quotes in JSON keys/values
fn attempt_fix_quotes(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let chars: Vec<char> = text.chars().collect();
    let mut in_string = false;
    let mut string_char = '"';

    for (i, &ch) in chars.iter().enumerate() {
        if !in_string {
            // Only convert single quotes to double quotes if they look like they wrap a value/key
            if ch == '\'' || ch == '"' {
                in_string = true;
                string_char = ch;
                result.push('"');
            } else {
                result.push(ch);
            }
        } else {
            if ch == string_char && (i == 0 || chars[i - 1] != '\\') {
                in_string = false;
                result.push('"');
            } else if ch == '"' && string_char == '\'' {
                result.push_str("\\\"");
            } else {
                result.push(ch);
            }
        }
    }

    result
}

/// Safely parse JSON with sanitization fallback
pub fn safe_parse_json<T: serde::de::DeserializeOwned>(text: &str) -> Result<T, String> {
    if let Ok(val) = serde_json::from_str::<T>(text) {
        return Ok(val);
    }

    let sanitized = sanitize_ai_json(text);
    if let Ok(val) = serde_json::from_str::<T>(&sanitized) {
        return Ok(val);
    }

    if let Some(json_val) = extract_json(text) {
        if let Ok(val) = serde_json::from_value::<T>(json_val) {
            return Ok(val);
        }
    }

    Err(format!("Failed to parse JSON: {}", &text[..text.len().min(200)]))
}
