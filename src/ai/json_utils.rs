use log::warn;
use serde_json::Value;

/// Extract a JSON object from AI response text (handles markdown fences, extra text)
pub fn extract_json(text: &str) -> Option<Value> {
    if let Ok(val) = serde_json::from_str::<Value>(text) {
        if val.is_object() {
            return Some(val);
        }
    }

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
    let check_val = |val: &Value| -> Option<Value> {
        if val.is_array() {
            return Some(val.clone());
        }
        if let Some(obj) = val.as_object() {
            for (_, v) in obj {
                if v.is_array() {
                    return Some(v.clone());
                }
            }
        }
        None
    };

    if let Ok(val) = serde_json::from_str::<Value>(text) {
        if let Some(res) = check_val(&val) {
            return Some(res);
        }
    }

    let cleaned = strip_code_fences(text);
    if let Ok(val) = serde_json::from_str::<Value>(&cleaned) {
        if let Some(res) = check_val(&val) {
            return Some(res);
        }
    }

    let sanitized = sanitize_ai_json(&cleaned);
    if let Ok(val) = serde_json::from_str::<Value>(&sanitized) {
        if let Some(res) = check_val(&val) {
            return Some(res);
        }
    }

    if let Some(start) = text.find('[') {
        if let Some(end) = text.rfind(']') {
            let candidate = &text[start..=end];
            let sanitized_candidate = sanitize_ai_json(candidate);
            if let Ok(val) = serde_json::from_str::<Value>(&sanitized_candidate) {
                if let Some(res) = check_val(&val) {
                    return Some(res);
                }
            }
        }
    }

    if let Some(obj_val) = extract_json(text) {
        if let Some(res) = check_val(&obj_val) {
            return Some(res);
        }
    }

    warn!("Failed to extract JSON array from AI response: {}", &text[..text.len().min(200)]);
    None
}

/// Strip markdown code fences from text
fn strip_code_fences(text: &str) -> String {
    let cleaned = text.trim();

    if let Some(start_idx) = cleaned.find("```") {
        let after_start = &cleaned[start_idx + 3..];
        let content_start = if let Some(newline_pos) = after_start.find('\n') {
            newline_pos + 1
        } else {
            0
        };
        let inner = &after_start[content_start..];
        
        if let Some(end_idx) = inner.find("```") {
            return inner[..end_idx].trim().to_string();
        }
    }

    cleaned.to_string()
}

/// Sanitize common JSON errors in a fast O(N) single pass
pub fn sanitize_ai_json(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut write_idx = 0;
    let mut i = 0;
    
    // We modify the string in-place conceptually by overwriting
    // a buffer to avoid O(N^2) memory allocations.
    let mut buffer = vec!['\0'; chars.len()];

    while i < chars.len() {
        if chars[i] == ',' {
            let mut j = i + 1;
            while j < chars.len() && chars[j].is_whitespace() {
                j += 1;
            }
            if j < chars.len() && (chars[j] == '}' || chars[j] == ']') {
                // Skip the comma by jumping the cursor
                i = j;
                continue;
            }
        }
        buffer[write_idx] = chars[i];
        write_idx += 1;
        i += 1;
    }

    let mut result: String = buffer[..write_idx].iter().collect();

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