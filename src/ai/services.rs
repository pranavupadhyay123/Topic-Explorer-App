use crate::ai::client::{call_ai, get_ai_config, ChatMessage};
use crate::ai::json_utils;
use crate::db::models::{
    AIExplorationResult, ChatMessage as ModelChatMessage, QuizQuestion,
};
use crate::db::DbPool;
use log::info;
use std::sync::Arc;

/// Explore a topic using AI — Now runs concurrently for extreme speed!
pub async fn explore_topic(
    pool: &Arc<DbPool>,
    topic: &str,
) -> Result<AIExplorationResult, String> {
    info!("Starting concurrent exploration for topic: {}", topic);
    
    // Step 1: Generate the core concepts first (we need them for relationships)
    let concepts = generate_concepts_pipeline(pool, topic).await?;
    
    let concept_names: Vec<String> = concepts.iter().map(|c| c.name.clone()).collect();

    // Step 2: Run all other generations at the EXACT SAME TIME in parallel
    // This reduces generation time by 60-80% compared to one giant prompt.
    let (relationships_res, timeline_res, flashcards_res, path_res) = tokio::join!(
        generate_relationships_pipeline(pool, topic, &concept_names),
        generate_timeline_pipeline(pool, topic),
        generate_flashcards_pipeline(pool, topic),
        generate_learning_path_pipeline(pool, topic)
    );

    // Assemble the final result. If any secondary part fails, return empty to prevent crashing.
    let result = AIExplorationResult {
        concepts,
        relationships: relationships_res.unwrap_or_default(),
        timeline: timeline_res.unwrap_or_default(),
        flashcards: flashcards_res.unwrap_or_default(),
        learning_path: path_res.ok(), // .ok() converts Result<T, E> into Option<T> without needing Default
    };

    info!(
        "Exploration complete: {} concepts, {} relationships",
        result.concepts.len(),
        result.relationships.len()
    );

    Ok(result)
}

/// Expand/deep-dive into a specific concept
pub async fn expand_concept(
    pool: &Arc<DbPool>,
    concept_name: &str,
    parent_topic: &str,
) -> Result<AIExplorationResult, String> {
    let config = get_ai_config(pool)?;

    let prompt = format!(
        r#"Deep dive into the concept "{concept}" in the context of "{topic}". Return a JSON object with sub-concepts, relationships between them, and flashcards. Use the same JSON structure:

{{
  "concepts": [
    {{
      "name": "Concept Name",
      "type": "concept|theory|technology|person|event|process|tool|framework|principle|pattern",
      "description": "Brief 1-2 sentence description",
      "importance": 1-10,
      "details": "CRITICAL MANDATORY REQUIREMENT: This field MUST be extremely detailed (minimum 150-300 words). If the topic is a recipe, you MUST provide the FULL list of ingredients and step-by-step cooking instructions here. If it is a programming topic, you MUST explain the architecture, syntax, and use cases comprehensively. For any other topic, provide an exhaustive multi-paragraph explanation. DO NOT give a 1-line summary.",
      "code_examples": [{{ "language": "javascript", "code": "example code", "description": "what it does" }}],
      "external_resources": [{{ "title": "Resource", "url": "https://...", "type": "article|video|docs" }}]
    }}
  ],
  "relationships": [
    {{
      "source": "Concept A",
      "target": "Concept B",
      "relationship_type": "type",
      "description": "CRITICAL: A precise 1-sentence explanation of exactly how these two concepts relate.",
      "strength": 1
    }}
  ],
  "timeline": [
    {{
      "title": "Event",
      "description": "Detail",
      "date_label": "Year",
      "period": "Era",
      "order_index": 1,
      "importance": "high",
      "category": "general"
    }}
  ],
  "flashcards": [
    {{
      "question": "Q",
      "answer": "A",
      "difficulty": 1
    }}
  ],
  "learning_path": {{
    "title": "Path",
    "description": "Desc",
    "steps": [
      {{
        "title": "Step",
        "description": "Learn",
        "resources": ["A"],
        "order": 1
      }}
    ],
    "difficulty": "beginner",
    "estimated_time": "1 week"
  }}
}}

Generate 3-4 sub-concepts, their relationships, 2-3 timeline events, 2-3 flashcards, and a focused learning path.
CRITICAL: You MUST include relationships in the `relationships` array that connect your newly generated sub-concepts back to the parent concept "{concept}". Use appropriate relationship types like "part_of" or "depends_on".
Return ONLY valid JSON."#,
        concept = concept_name,
        topic = parent_topic
    );

    let messages = vec![
        ChatMessage {
            role: "system".into(),
            content: "You are a knowledge exploration AI. Always respond with valid JSON only.".into(),
        },
        ChatMessage {
            role: "user".into(),
            content: prompt,
        },
    ];

    let response = call_ai(&config, &messages).await?;

    let json = json_utils::extract_json(&response)
        .ok_or_else(|| "Failed to parse AI response".to_string())?;

    let result: AIExplorationResult = serde_json::from_value(json)
        .map_err(|e| format!("Failed to deserialize: {}", e))?;

    Ok(result)
}

/// AI Tutor chat — conversational learning
pub async fn tutor_chat(
    pool: &Arc<DbPool>,
    topic: &str,
    messages_history: &[ModelChatMessage],
    user_message: &str,
) -> Result<String, String> {
    let config = get_ai_config(pool)?;

    let system = format!(
        "You are an expert AI tutor helping the user learn about \"{}\". \
         Be encouraging, use analogies, and explain things clearly. \
         If the user asks about something you're unsure about, be honest. \
         Use markdown formatting for clarity.",
        topic
    );

    let mut messages = vec![ChatMessage {
        role: "system".into(),
        content: system,
    }];

    for msg in messages_history {
        messages.push(ChatMessage {
            role: msg.role.clone(),
            content: msg.content.clone(),
        });
    }

    messages.push(ChatMessage {
        role: "user".into(),
        content: user_message.into(),
    });

    let response = call_ai(&config, &messages).await?;
    Ok(response)
}

/// Get layered explanations
pub async fn get_layered_explanation(
    pool: &Arc<DbPool>,
    concept_name: &str,
    topic: &str,
) -> Result<serde_json::Value, String> {
    let config = get_ai_config(pool)?;

    let prompt = format!(
        r#"Explain the concept "{concept}" (in the context of "{topic}") at 5 different levels. Return a JSON object:

{{
  "eli5": "Explain like I'm 5 years old (simple analogies, no jargon)",
  "beginner": "Beginner level (basic terms, gentle introduction)",
  "intermediate": "Intermediate (some technical detail, examples)",
  "advanced": "Advanced (deep technical detail, edge cases)",
  "expert": "Expert level (research-level depth, cutting-edge aspects)"
}}

Make each explanation progressively more detailed. Return ONLY valid JSON."#,
        concept = concept_name,
        topic = topic
    );

    let messages = vec![
        ChatMessage {
            role: "system".into(),
            content: "You are an expert educator. Always respond with valid JSON only.".into(),
        },
        ChatMessage {
            role: "user".into(),
            content: prompt,
        },
    ];

    let response = call_ai(&config, &messages).await?;

    let json = json_utils::extract_json(&response)
        .ok_or_else(|| "Failed to parse explanation response".to_string())?;

    Ok(json)
}

/// Generate quiz questions
pub async fn generate_quiz(
    pool: &Arc<DbPool>,
    topic: &str,
    concept_names: &[String],
) -> Result<Vec<QuizQuestion>, String> {
    let config = get_ai_config(pool)?;
    let concepts_list = concept_names.join(", ");

    let prompt = format!(
        r#"Generate a quiz about "{topic}" covering these concepts: {concepts}. Return a JSON array of questions:

[
  {{
    "question": "What is...?",
    "options": ["Option A", "Option B", "Option C", "Option D"],
    "correct_answer": 0,
    "explanation": "The correct answer is A because..."
  }}
]

Generate 5-8 multiple choice questions. Vary difficulty. Return ONLY a JSON array."#,
        topic = topic,
        concepts = concepts_list
    );

    let messages = vec![
        ChatMessage {
            role: "system".into(),
            content: "You are a quiz generator. Always respond with a valid JSON array only."
                .into(),
        },
        ChatMessage {
            role: "user".into(),
            content: prompt,
        },
    ];

    let response = call_ai(&config, &messages).await?;
    let json = json_utils::extract_json_array(&response)
        .ok_or_else(|| "Failed to parse quiz response".to_string())?;

    let questions: Vec<QuizQuestion> = serde_json::from_value(json)
        .map_err(|e| format!("Failed to deserialize quiz: {}", e))?;

    Ok(questions)
}

// ─── Pipeline Generation Functions ──────────────────────────────────────────

pub async fn generate_concepts_pipeline(
    pool: &Arc<DbPool>,
    topic: &str,
) -> Result<Vec<crate::db::models::ConceptInput>, String> {
    let config = get_ai_config(pool)?;
    let prompt = format!(
        r#"Generate strictly between 5 and 15 core concepts for the topic: "{topic}".
Return ONLY a JSON array of objects. Schema for each object:
{{
  "name": "Concept Name",
  "type": "concept|theory|technology|person|event|process|tool|framework",
  "description": "Brief summary",
  "importance": 1-10,
  "details": "Write a clear 2-sentence summary.",
  "code_examples": [],
  "external_resources": []
}}"#,
        topic = topic
    );

    let messages = vec![
        ChatMessage { role: "system".into(), content: "Return a valid JSON array only.".into() },
        ChatMessage { role: "user".into(), content: prompt },
    ];
    let response = call_ai(&config, &messages).await?;
    let json = json_utils::extract_json_array(&response).ok_or_else(|| "Failed to parse concepts JSON".to_string())?;
    serde_json::from_value(json).map_err(|e| e.to_string())
}

pub async fn generate_relationships_pipeline(
    pool: &Arc<DbPool>,
    topic: &str,
    concept_names: &[String],
) -> Result<Vec<crate::db::models::RelationshipInput>, String> {
    let config = get_ai_config(pool)?;
    let concepts_list = concept_names.join(", ");
    let prompt = format!(
        r#"For the topic "{topic}", analyze these concepts: {concepts_list}.
Generate strictly between 6 and 15 meaningful relationships among them.
Return ONLY a JSON array of objects. Schema:
{{
  "source": "Exact Concept Name A",
  "target": "Exact Concept Name B",
  "relationship_type": "depends_on|part_of|contains|extends|implements|uses|requires|produces|consumes|enables|causes|improves|replaces|competes_with|alternative_to|parent_of|child_of|inspired_by|evolved_into|similar_to|contradicts|complements",
  "description": "1-sentence explanation of how they relate.",
  "strength": 1-10
}}"#,
        topic = topic, concepts_list = concepts_list
    );

    let messages = vec![
        ChatMessage { role: "system".into(), content: "Return a valid JSON array only.".into() },
        ChatMessage { role: "user".into(), content: prompt },
    ];
    let response = call_ai(&config, &messages).await?;
    let json = json_utils::extract_json_array(&response).ok_or_else(|| "Failed to parse relationships JSON".to_string())?;
    serde_json::from_value(json).map_err(|e| e.to_string())
}

pub async fn generate_flashcards_pipeline(
    pool: &Arc<DbPool>,
    topic: &str,
) -> Result<Vec<crate::db::models::FlashcardInput>, String> {
    let config = get_ai_config(pool)?;
    let prompt = format!(
        r#"Generate 4 to 8 flashcards for the topic "{topic}".
Return ONLY a JSON array of objects. Schema:
{{
  "question": "Question text",
  "answer": "Answer text",
  "difficulty": 1-4
}}"#,
        topic = topic
    );

    let messages = vec![
        ChatMessage { role: "system".into(), content: "Return a valid JSON array only.".into() },
        ChatMessage { role: "user".into(), content: prompt },
    ];
    let response = call_ai(&config, &messages).await?;
    let json = json_utils::extract_json_array(&response).ok_or_else(|| "Failed to parse flashcards JSON".to_string())?;
    serde_json::from_value(json).map_err(|e| e.to_string())
}

pub async fn generate_timeline_pipeline(
    pool: &Arc<DbPool>,
    topic: &str,
) -> Result<Vec<crate::db::models::TimelineEventInput>, String> {
    let config = get_ai_config(pool)?;
    let prompt = format!(
        r#"Generate the historical evolution timeline for "{topic}".
Return ONLY a JSON array of objects. Schema:
{{
  "title": "Event Title",
  "description": "Detailed explanation",
  "date_label": "Year or period",
  "period": "Foundations|Early ideas|Major discoveries|Industrial adoption|Modern developments",
  "order_index": 0,
  "importance": "high|medium|low",
  "category": "discovery|development|milestone|general"
}}"#,
        topic = topic
    );

    let messages = vec![
        ChatMessage { role: "system".into(), content: "Return a valid JSON array only.".into() },
        ChatMessage { role: "user".into(), content: prompt },
    ];
    let response = call_ai(&config, &messages).await?;
    let json = json_utils::extract_json_array(&response).ok_or_else(|| "Failed to parse timeline JSON".to_string())?;
    serde_json::from_value(json).map_err(|e| e.to_string())
}

pub async fn generate_learning_path_pipeline(
    pool: &Arc<DbPool>,
    topic: &str,
) -> Result<crate::db::models::LearningPathInput, String> {
    let config = get_ai_config(pool)?;
    let prompt = format!(
        r#"Generate a comprehensive learning path for mastering "{topic}".
Return ONLY a single JSON object. Schema:
{{
  "title": "Mastering {topic}",
  "description": "A step-by-step guide...",
  "difficulty": "beginner|intermediate|advanced",
  "estimated_time": "e.g., 4 weeks",
  "steps": [
    {{
      "title": "Step title",
      "description": "What to learn in this step",
      "order": 1,
      "resources": []
    }}
  ]
}}"#,
        topic = topic
    );

    let messages = vec![
        ChatMessage { role: "system".into(), content: "Return a valid JSON object only.".into() },
        ChatMessage { role: "user".into(), content: prompt },
    ];
    let response = call_ai(&config, &messages).await?;
    let json = json_utils::extract_json(&response).ok_or_else(|| "Failed to parse learning path JSON".to_string())?;
    serde_json::from_value(json).map_err(|e| e.to_string())
}