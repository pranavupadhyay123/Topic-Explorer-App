use actix_web::{web, HttpResponse};
use crate::ai::{providers, services};
use crate::db::DbPool;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct ExploreRequest {
    pub topic_id: String,
    pub topic: String,
}

#[derive(Deserialize)]
pub struct ExpandRequest {
    pub topic_id: String,
    pub concept_name: String,
    pub parent_topic: String,
    pub parent_concept_id: Option<String>,
}

#[derive(Deserialize)]
pub struct ExplainRequest {
    pub concept_name: String,
    pub topic: String,
}

#[derive(Deserialize)]
pub struct QuizRequest {
    pub topic: String,
    pub concept_names: Vec<String>,
}

#[derive(Deserialize)]
pub struct TutorRequest {
    pub topic: String,
    pub topic_id: String,
    pub message: String,
    pub conversation_id: Option<String>,
}

#[derive(Deserialize)]
pub struct ModelsRequest {
    pub provider: String,
    pub api_key: Option<String>,
    pub endpoint: Option<String>,
}

/// POST /api/ai/explore — explore a topic or expand a concept
pub async fn explore(
    pool: web::Data<Arc<DbPool>>,
    body: web::Json<serde_json::Value>,
) -> HttpResponse {
    let pool_ref = pool.into_inner();

    // Check if this is an expand request or explore request
    if body.get("concept_name").is_some() {
        // Expand concept
        let concept_name = body["concept_name"].as_str().unwrap_or("").to_string();
        let parent_topic = body["parent_topic"].as_str().unwrap_or("").to_string();
        let topic_id = body["topic_id"].as_str().unwrap_or("").to_string();
        let parent_concept_id = body["parent_concept_id"].as_str().map(|s| s.to_string());

        match services::expand_concept(&pool_ref, &concept_name, &parent_topic).await {
            Ok(result) => {
                // Save results to database
                if let Err(e) = save_exploration_results(&pool_ref, &topic_id, &result, parent_concept_id.as_deref()) {
                    log::error!("Failed to save expansion results: {}", e);
                }

                // Update topic status
                let conn = pool_ref.get().ok();
                if let Some(conn) = conn {
                    let _ = conn.execute(
                        "UPDATE topics SET status = 'explored', explored_at = datetime('now'), updated_at = datetime('now') WHERE id = ?1",
                        rusqlite::params![topic_id],
                    );
                }

                HttpResponse::Ok().json(serde_json::json!({"success": true, "result": serde_json::to_value(&result).unwrap_or_default()}))
            }
            Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
        }
    } else {
        // Explore topic
        let topic = body["topic"].as_str().unwrap_or("").to_string();
        let topic_id = body["topic_id"].as_str().unwrap_or("").to_string();

        if topic.is_empty() {
            return HttpResponse::BadRequest().json(serde_json::json!({"error": "Missing topic"}));
        }

        match services::explore_topic(&pool_ref, &topic).await {
            Ok(result) => {
                // Save results to database
                if let Err(e) = save_exploration_results(&pool_ref, &topic_id, &result, None) {
                    log::error!("Failed to save exploration results: {}", e);
                }

                // Update topic status
                let conn = pool_ref.get().ok();
                if let Some(conn) = conn {
                    let _ = conn.execute(
                        "UPDATE topics SET status = 'explored', explored_at = datetime('now'), updated_at = datetime('now') WHERE id = ?1",
                        rusqlite::params![topic_id],
                    );
                }

                HttpResponse::Ok().json(serde_json::json!({"success": true, "result": serde_json::to_value(&result).unwrap_or_default()}))
            }
            Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
        }
    }
}

/// Save AI exploration results to the database
fn save_exploration_results(
    pool: &Arc<DbPool>,
    topic_id: &str,
    result: &crate::db::models::AIExplorationResult,
    parent_concept_id: Option<&str>,
) -> Result<(), String> {
    let conn = pool.get().map_err(|e| e.to_string())?;

    // Map concept names to IDs for relationship linking
    let mut concept_name_to_id: std::collections::HashMap<String, String> = std::collections::HashMap::new();

    // Pre-populate with existing concepts for this topic
    if let Ok(mut stmt) = conn.prepare("SELECT id, name FROM concepts WHERE topic_id = ?1") {
        if let Ok(existing) = stmt.query_map(rusqlite::params![topic_id], |row| {
            let id: String = row.get(0)?;
            let name: String = row.get(1)?;
            Ok((name.to_lowercase(), id))
        }) {
            for (name, id) in existing.flatten() {
                concept_name_to_id.insert(name, id);
            }
        }
    }

    // Insert concepts
    for concept in &result.concepts {
        let id = Uuid::new_v4().to_string();
        let code_examples_json = serde_json::to_string(&concept.code_examples).unwrap_or_else(|_| "[]".into());
        let external_resources_json = serde_json::to_string(&concept.external_resources).unwrap_or_else(|_| "[]".into());
        let depth = if parent_concept_id.is_some() { 1 } else { 0 };

        conn.execute(
            "INSERT INTO concepts (id, topic_id, name, type, description, importance, details, code_examples, external_resources, parent_concept_id, depth) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
            rusqlite::params![
                id, topic_id, concept.name, concept.r#type, concept.description,
                concept.importance, concept.details, code_examples_json, external_resources_json,
                parent_concept_id, depth
            ],
        ).map_err(|e| format!("Failed to insert concept: {}", e))?;

        concept_name_to_id.insert(concept.name.to_lowercase(), id.clone());

        // FORCEFULLY link newly discovered concepts back to their parent
        if let Some(parent_id) = parent_concept_id {
            let rel_id = Uuid::new_v4().to_string();
            let _ = conn.execute(
                "INSERT INTO relationships (id, topic_id, source_concept_id, target_concept_id, relationship_type, description, strength) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                rusqlite::params![rel_id, topic_id, parent_id, id, "part_of", "Sub-concept", 8],
            );
        }
    }

    // Insert relationships
    for rel in &result.relationships {
        let source_id = concept_name_to_id.get(&rel.source.to_lowercase());
        let target_id = concept_name_to_id.get(&rel.target.to_lowercase());

        if let (Some(src), Some(tgt)) = (source_id, target_id) {
            let id = Uuid::new_v4().to_string();
            let _ = conn.execute(
                "INSERT INTO relationships (id, topic_id, source_concept_id, target_concept_id, relationship_type, description, strength) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
                rusqlite::params![id, topic_id, src, tgt, rel.relationship_type, rel.description, rel.strength],
            );
        }
    }

    // Insert timeline events
    for (i, event) in result.timeline.iter().enumerate() {
        let id = Uuid::new_v4().to_string();
        let _ = conn.execute(
            "INSERT INTO timeline_events (id, topic_id, title, description, date_label, period, order_index, importance, category) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            rusqlite::params![
                id, topic_id, event.title, event.description, event.date_label,
                event.period, event.order_index.max(i as i32), event.importance, event.category
            ],
        );
    }

    // Insert flashcards
    for fc in &result.flashcards {
        let id = Uuid::new_v4().to_string();
        let _ = conn.execute(
            "INSERT INTO flashcards (id, topic_id, question, answer, difficulty) VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![id, topic_id, fc.question, fc.answer, fc.difficulty],
        );
    }

    // Insert learning path
    if let Some(ref lp) = result.learning_path {
        let id = Uuid::new_v4().to_string();
        let steps_json = serde_json::to_string(&lp.steps).unwrap_or_else(|_| "[]".into());
        let _ = conn.execute(
            "INSERT INTO learning_paths (id, topic_id, title, description, steps, difficulty, estimated_time) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            rusqlite::params![id, topic_id, lp.title, lp.description, steps_json, lp.difficulty, lp.estimated_time],
        );
    }

    Ok(())
}

/// POST /api/ai/explain
pub async fn explain(
    pool: web::Data<Arc<DbPool>>,
    body: web::Json<ExplainRequest>,
) -> HttpResponse {
    let pool_ref = pool.into_inner();

    match services::get_layered_explanation(&pool_ref, &body.concept_name, &body.topic).await {
        Ok(explanation) => HttpResponse::Ok().json(explanation),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}

/// POST /api/ai/quiz
pub async fn quiz(
    pool: web::Data<Arc<DbPool>>,
    body: web::Json<QuizRequest>,
) -> HttpResponse {
    let pool_ref = pool.into_inner();

    match services::generate_quiz(&pool_ref, &body.topic, &body.concept_names).await {
        Ok(questions) => HttpResponse::Ok().json(questions),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}

/// POST /api/ai/tutor
pub async fn tutor(
    pool: web::Data<Arc<DbPool>>,
    body: web::Json<TutorRequest>,
) -> HttpResponse {
    let pool_ref = pool.into_inner();

    // Load conversation history if conversation_id provided
    let history = if let Some(ref convo_id) = body.conversation_id {
        let conn = match pool_ref.get() {
            Ok(c) => c,
            Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
        };

        let messages_json: String = conn
            .query_row(
                "SELECT messages FROM conversations WHERE id = ?1",
                rusqlite::params![convo_id],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| "[]".into());

        serde_json::from_str(&messages_json).unwrap_or_default()
    } else {
        vec![]
    };

    match services::tutor_chat(&pool_ref, &body.topic, &history, &body.message).await {
        Ok(response) => {
            // Save conversation
            let conn = pool_ref.get().ok();
            let convo_id = body.conversation_id.clone().unwrap_or_else(|| Uuid::new_v4().to_string());

            if let Some(conn) = conn {
                let mut messages = history.clone();
                messages.push(crate::db::models::ChatMessage {
                    role: "user".into(),
                    content: body.message.clone(),
                });
                messages.push(crate::db::models::ChatMessage {
                    role: "assistant".into(),
                    content: response.clone(),
                });

                let messages_json = serde_json::to_string(&messages).unwrap_or_else(|_| "[]".into());

                if body.conversation_id.is_some() {
                    let _ = conn.execute(
                        "UPDATE conversations SET messages = ?1, updated_at = datetime('now') WHERE id = ?2",
                        rusqlite::params![messages_json, convo_id],
                    );
                } else {
                    let _ = conn.execute(
                        "INSERT INTO conversations (id, topic_id, title, messages) VALUES (?1, ?2, ?3, ?4)",
                        rusqlite::params![convo_id, body.topic_id, format!("Chat about {}", body.topic), messages_json],
                    );
                }
            }

            HttpResponse::Ok().json(serde_json::json!({
                "response": response,
                "conversation_id": convo_id,
            }))
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}

/// POST /api/ai/models — fetch available models from a provider
pub async fn models(body: web::Json<ModelsRequest>) -> HttpResponse {
    let api_key = body.api_key.clone().unwrap_or_default();
    let endpoint = body.endpoint.clone().unwrap_or_default();

    match providers::fetch_models_from_provider(&body.provider, &api_key, &endpoint).await {
        Ok(models) => HttpResponse::Ok().json(models),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e})),
    }
}
