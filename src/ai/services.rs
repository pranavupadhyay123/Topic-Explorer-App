use crate::ai::client::{call_ai, get_ai_config, ChatMessage};
use crate::ai::json_utils;
use crate::db::models::{
    AIExplorationResult, ChatMessage as ModelChatMessage, QuizQuestion,
};
use crate::db::DbPool;
use log::info;
use std::sync::Arc;

/// Explore a topic using AI — returns concepts, relationships, timeline, flashcards, learning path
pub async fn explore_topic(
    pool: &Arc<DbPool>,
    topic: &str,
) -> Result<AIExplorationResult, String> {
    let config = get_ai_config(pool)?;

    let prompt = format!(
        r#"
        You are an elite AI Knowledge Architect.

        Your task is NOT to summarize a topic.

        Your task is to build a complete machine-readable knowledge graph and learning framework for the topic.

        TOPIC:
        "{topic}"

        ===========================================================
        GOALS
        ===========================================================

        Imagine this JSON will power an interactive application where users can:

        • Click any node to infinitely expand it
        • Learn from beginner to PhD level
        • View relationships
        • View historical evolution
        • View future predictions
        • Generate quizzes
        • Study visually
        • Understand every dependency

        Therefore NEVER omit information.

        If information exists, include it.

        ===========================================================
        GENERAL RULES
        ===========================================================

        Generate information that is:

        • technically accurate
        • comprehensive
        • logically organized
        • educational
        • interconnected
        • non-repetitive
        • hierarchical

        Every concept should have enough information to become an article by itself.

        Never write shallow descriptions.

        Every "details" field must contain between 150 and 300 words.

        ===========================================================
        CONCEPTS
        ===========================================================

        Generate between 3 and 6 concepts.

        Each concept should include:

        {{
        "name":"Concept Name",
        "type":"concept|theory|technology|person|event|process|tool|framework",
        "description":"Brief summary",
        "importance":1-10,

        "details":"CRITICAL: Write an extremely comprehensive (300+ words) markdown explanation. You MUST include sections for: History/Origin, Working Principles, Internal Architecture, Advantages & Limitations, Common Mistakes, Best Practices, Real-World Industry Usage, Future Trends, and Mathematics/Algorithms (if applicable).",

        "code_examples":[
        {{
        "language":"python",
        "code":"print('example')",
        "description":"What it does"
        }}
        ],

        "external_resources":[]
        }}

        ===========================================================
        RELATIONSHIPS
        ===========================================================

        Generate between 4 and 8 meaningful relationships.
        Each relationship MUST follow this JSON schema:
        {{
          "source": "Concept A Name",
          "target": "Concept B Name",
          "relationship_type": "depends_on|part_of|contains|extends|implements|uses|requires|produces|consumes|enables|causes|improves|replaces|competes_with|alternative_to|parent_of|child_of|inspired_by|evolved_into|similar_to|contradicts|complements",
          "description": "CRITICAL: A precise 1-sentence explanation of exactly how these two concepts relate.",
          "strength": 1-10 (integer)
        }}

        ===========================================================
        TIMELINE
        ===========================================================

        Generate the complete historical evolution.
        Each event MUST follow this JSON schema:
        {{
          "title": "Event Title",
          "description": "Detailed explanation",
          "date_label": "Year or period",
          "period": "Foundations|Early ideas|Major discoveries|Industrial adoption|Modern developments",
          "order_index": 0 (integer),
          "importance": "high|medium|low",
          "category": "discovery|development|milestone|general"
        }}

        ===========================================================
        FLASHCARDS
        ===========================================================

        Generate 4 to 6 flashcards.
        Each flashcard MUST follow this JSON schema:
        {{
          "question": "Question text",
          "answer": "Answer text",
          "difficulty": 1-4 (integer, where 1=Easy, 2=Medium, 3=Hard, 4=Expert)
        }}

        ===========================================================
        LEARNING PATH
        ===========================================================

        Generate a structured curriculum.
        The learning path MUST follow this JSON schema:
        {{
          "title": "Path Title",
          "description": "Path description",
          "steps": [
            {{
              "title": "Step 1",
              "description": "What to learn",
              "resources": ["Book A", "Course B"],
              "order": 1 (integer)
            }}
          ],
          "difficulty": "beginner|intermediate|advanced|expert",
          "estimated_time": "Time string"
        }}

        ===========================================================
        MISCONCEPTIONS
        ===========================================================

        Generate common misconceptions.

        Explain why they are incorrect.

        ===========================================================
        COMPARISONS
        ===========================================================

        Compare this topic with similar technologies or concepts.

        ===========================================================
        APPLICATIONS
        ===========================================================

        List applications in:

        Education

        Healthcare

        Finance

        Robotics

        AI

        Cloud

        Cybersecurity

        Gaming

        IoT

        Research

        Industry

        ===========================================================
        OUTPUT
        ===========================================================

        Return ONLY valid JSON.

        No markdown.

        No explanations.

        No comments.

        No prose outside JSON.

        The JSON MUST follow this schema:

        {{
        "concepts": [...],
        "relationships": [...],
        "timeline": [...],
        "flashcards": [...],
        "learning_path": {{...}}
        }}

        The output should be detailed enough that an interactive knowledge explorer can recursively expand every node without losing context.
        "#,
        topic = topic
        );

    let messages = vec![
        ChatMessage {
            role: "system".into(),
            content: "You are a knowledge exploration AI. Always respond with valid JSON only. No markdown, no extra text.".into(),
        },
        ChatMessage {
            role: "user".into(),
            content: prompt,
        },
    ];

    info!("Exploring topic: {}", topic);
    let response = call_ai(&config, &messages).await?;

    let json = json_utils::extract_json(&response)
        .ok_or_else(|| "Failed to parse AI response as JSON".to_string())?;

    let result: AIExplorationResult = serde_json::from_value(json)
        .map_err(|e| format!("Failed to deserialize exploration result: {}", e))?;

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

    // Add conversation history
    for msg in messages_history {
        messages.push(ChatMessage {
            role: msg.role.clone(),
            content: msg.content.clone(),
        });
    }

    // Add the new user message
    messages.push(ChatMessage {
        role: "user".into(),
        content: user_message.into(),
    });

    let response = call_ai(&config, &messages).await?;
    Ok(response)
}

/// Get layered explanations (ELI5 → Beginner → Intermediate → Advanced → Expert)
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

/// Generate quiz questions about explored concepts
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