use actix_web::{web, HttpRequest, HttpResponse};
use crate::db::DbPool;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateConcept {
    pub topic_id: String,
    pub name: String,
    pub r#type: Option<String>,
    pub description: Option<String>,
    pub importance: Option<i32>,
    pub details: Option<String>,
    pub code_examples: Option<String>,
    pub external_resources: Option<String>,
    pub parent_concept_id: Option<String>,
    pub depth: Option<i32>,
}

#[derive(Deserialize)]
pub struct UpdateConcept {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub importance: Option<i32>,
    pub details: Option<String>,
    pub explored: Option<bool>,
}

pub async fn list_concepts(
    pool: web::Data<Arc<DbPool>>,
    req: HttpRequest,
) -> HttpResponse {
    let conn = match pool.get() {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    };

    let query_params = web::Query::<std::collections::HashMap<String, String>>::from_query(req.query_string())
        .unwrap_or_else(|_| web::Query(std::collections::HashMap::new()));

    let topic_id = match query_params.get("topic_id") {
        Some(id) => id.clone(),
        None => return HttpResponse::BadRequest().json(serde_json::json!({"error": "Missing topic_id"})),
    };

    // Fetch concepts
    let mut stmt = match conn.prepare(
        "SELECT id, topic_id, name, type, description, importance, details, code_examples, external_resources, \
         parent_concept_id, depth, explored, created_at FROM concepts WHERE topic_id = ?1 ORDER BY importance DESC"
    ) {
        Ok(s) => s,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    };

    let concepts: Vec<serde_json::Value> = stmt.query_map(rusqlite::params![topic_id], |row| {
        Ok(serde_json::json!({
            "id": row.get::<_, String>(0)?,
            "topic_id": row.get::<_, String>(1)?,
            "name": row.get::<_, String>(2)?,
            "type": row.get::<_, String>(3).unwrap_or_else(|_| "concept".into()),
            "description": row.get::<_, String>(4).unwrap_or_default(),
            "importance": row.get::<_, i32>(5).unwrap_or(5),
            "details": row.get::<_, String>(6).unwrap_or_default(),
            "code_examples": row.get::<_, String>(7).unwrap_or_else(|_| "[]".into()),
            "external_resources": row.get::<_, String>(8).unwrap_or_else(|_| "[]".into()),
            "parent_concept_id": row.get::<_, Option<String>>(9)?,
            "depth": row.get::<_, i32>(10).unwrap_or(0),
            "explored": row.get::<_, i32>(11).unwrap_or(0) != 0,
            "created_at": row.get::<_, String>(12)?,
        }))
    }).ok().map(|r| r.filter_map(|r| r.ok()).collect()).unwrap_or_default();

    // Fetch relationships for this topic
    let mut rel_stmt = match conn.prepare(
        "SELECT id, topic_id, source_concept_id, target_concept_id, relationship_type, description, strength, created_at \
         FROM relationships WHERE topic_id = ?1"
    ) {
        Ok(s) => s,
        Err(_) => return HttpResponse::Ok().json(serde_json::json!({"concepts": concepts, "relationships": []})),
    };

    let relationships: Vec<serde_json::Value> = rel_stmt.query_map(rusqlite::params![topic_id], |row| {
        Ok(serde_json::json!({
            "id": row.get::<_, String>(0)?,
            "topic_id": row.get::<_, String>(1)?,
            "source_concept_id": row.get::<_, String>(2)?,
            "target_concept_id": row.get::<_, String>(3)?,
            "relationship_type": row.get::<_, String>(4).unwrap_or_else(|_| "relates_to".into()),
            "description": row.get::<_, String>(5).unwrap_or_default(),
            "strength": row.get::<_, i32>(6).unwrap_or(5),
            "created_at": row.get::<_, String>(7)?,
        }))
    }).ok().map(|r| r.filter_map(|r| r.ok()).collect()).unwrap_or_default();

    // Fetch timeline events
    let mut tl_stmt = match conn.prepare(
        "SELECT id, topic_id, title, description, date_label, period, order_index, importance, category, created_at \
         FROM timeline_events WHERE topic_id = ?1 ORDER BY order_index"
    ) {
        Ok(s) => s,
        Err(_) => return HttpResponse::Ok().json(serde_json::json!({
            "concepts": concepts, "relationships": relationships, "timeline": [], "flashcards": [], "learning_paths": []
        })),
    };

    let timeline: Vec<serde_json::Value> = tl_stmt.query_map(rusqlite::params![topic_id], |row| {
        Ok(serde_json::json!({
            "id": row.get::<_, String>(0)?,
            "topic_id": row.get::<_, String>(1)?,
            "title": row.get::<_, String>(2)?,
            "description": row.get::<_, String>(3).unwrap_or_default(),
            "date_label": row.get::<_, String>(4).unwrap_or_default(),
            "period": row.get::<_, String>(5).unwrap_or_default(),
            "order_index": row.get::<_, i32>(6).unwrap_or(0),
            "importance": row.get::<_, String>(7).unwrap_or_else(|_| "medium".into()),
            "category": row.get::<_, String>(8).unwrap_or_else(|_| "general".into()),
            "created_at": row.get::<_, String>(9)?,
        }))
    }).ok().map(|r| r.filter_map(|r| r.ok()).collect()).unwrap_or_default();

    // Fetch flashcards
    let mut fc_stmt = match conn.prepare(
        "SELECT id, topic_id, concept_id, question, answer, difficulty, times_reviewed, times_correct, \
         last_reviewed, next_review, created_at FROM flashcards WHERE topic_id = ?1"
    ) {
        Ok(s) => s,
        Err(_) => return HttpResponse::Ok().json(serde_json::json!({
            "concepts": concepts, "relationships": relationships, "timeline": timeline, "flashcards": [], "learning_paths": []
        })),
    };

    let flashcards: Vec<serde_json::Value> = fc_stmt.query_map(rusqlite::params![topic_id], |row| {
        Ok(serde_json::json!({
            "id": row.get::<_, String>(0)?,
            "topic_id": row.get::<_, String>(1)?,
            "concept_id": row.get::<_, Option<String>>(2)?,
            "question": row.get::<_, String>(3)?,
            "answer": row.get::<_, String>(4)?,
            "difficulty": row.get::<_, i32>(5).unwrap_or(1),
            "times_reviewed": row.get::<_, i32>(6).unwrap_or(0),
            "times_correct": row.get::<_, i32>(7).unwrap_or(0),
            "last_reviewed": row.get::<_, Option<String>>(8)?,
            "next_review": row.get::<_, Option<String>>(9)?,
            "created_at": row.get::<_, String>(10)?,
        }))
    }).ok().map(|r| r.filter_map(|r| r.ok()).collect()).unwrap_or_default();

    // Fetch learning paths
    let mut lp_stmt = match conn.prepare(
        "SELECT id, topic_id, title, description, steps, difficulty, estimated_time, created_at \
         FROM learning_paths WHERE topic_id = ?1"
    ) {
        Ok(s) => s,
        Err(_) => return HttpResponse::Ok().json(serde_json::json!({
            "concepts": concepts, "relationships": relationships, "timeline": timeline,
            "flashcards": flashcards, "learning_paths": []
        })),
    };

    let learning_paths: Vec<serde_json::Value> = lp_stmt.query_map(rusqlite::params![topic_id], |row| {
        Ok(serde_json::json!({
            "id": row.get::<_, String>(0)?,
            "topic_id": row.get::<_, String>(1)?,
            "title": row.get::<_, String>(2)?,
            "description": row.get::<_, String>(3).unwrap_or_default(),
            "steps": row.get::<_, String>(4).unwrap_or_else(|_| "[]".into()),
            "difficulty": row.get::<_, String>(5).unwrap_or_else(|_| "beginner".into()),
            "estimated_time": row.get::<_, String>(6).unwrap_or_default(),
            "created_at": row.get::<_, String>(7)?,
        }))
    }).ok().map(|r| r.filter_map(|r| r.ok()).collect()).unwrap_or_default();

    HttpResponse::Ok().json(serde_json::json!({
        "concepts": concepts,
        "relationships": relationships,
        "timeline": timeline,
        "flashcards": flashcards,
        "learning_paths": learning_paths,
    }))
}

pub async fn create_concept(
    pool: web::Data<Arc<DbPool>>,
    body: web::Json<CreateConcept>,
) -> HttpResponse {
    let conn = match pool.get() {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    };

    let id = Uuid::new_v4().to_string();

    match conn.execute(
        "INSERT INTO concepts (id, topic_id, name, type, description, importance, details, code_examples, external_resources, parent_concept_id, depth) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        rusqlite::params![
            id,
            body.topic_id,
            body.name,
            body.r#type.clone().unwrap_or_else(|| "concept".into()),
            body.description.clone().unwrap_or_default(),
            body.importance.unwrap_or(5),
            body.details.clone().unwrap_or_default(),
            body.code_examples.clone().unwrap_or_else(|| "[]".into()),
            body.external_resources.clone().unwrap_or_else(|| "[]".into()),
            body.parent_concept_id,
            body.depth.unwrap_or(0),
        ],
    ) {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({"id": id, "name": body.name})),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

pub async fn update_concept(
    pool: web::Data<Arc<DbPool>>,
    body: web::Json<UpdateConcept>,
) -> HttpResponse {
    let conn = match pool.get() {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    };

    let mut updates = vec![];
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![];

    if let Some(ref name) = body.name {
        updates.push("name = ?");
        params.push(Box::new(name.clone()));
    }
    if let Some(ref desc) = body.description {
        updates.push("description = ?");
        params.push(Box::new(desc.clone()));
    }
    if let Some(imp) = body.importance {
        updates.push("importance = ?");
        params.push(Box::new(imp));
    }
    if let Some(ref details) = body.details {
        updates.push("details = ?");
        params.push(Box::new(details.clone()));
    }
    if let Some(explored) = body.explored {
        updates.push("explored = ?");
        params.push(Box::new(explored as i32));
    }

    if updates.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({"error": "No fields to update"}));
    }

    params.push(Box::new(body.id.clone()));

    let sql = format!("UPDATE concepts SET {} WHERE id = ?", updates.join(", "));
    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    match conn.execute(&sql, param_refs.as_slice()) {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({"success": true})),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

pub async fn delete_concept(
    pool: web::Data<Arc<DbPool>>,
    req: HttpRequest,
) -> HttpResponse {
    let query_params = web::Query::<std::collections::HashMap<String, String>>::from_query(req.query_string())
        .unwrap_or_else(|_| web::Query(std::collections::HashMap::new()));

    let id = match query_params.get("id") {
        Some(id) => id.clone(),
        None => return HttpResponse::BadRequest().json(serde_json::json!({"error": "Missing id"})),
    };

    let conn = match pool.get() {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    };

    // Delete related data
    let _ = conn.execute("DELETE FROM knowledge_cards WHERE concept_id = ?1", rusqlite::params![id]);
    let _ = conn.execute("DELETE FROM relationships WHERE source_concept_id = ?1 OR target_concept_id = ?1", rusqlite::params![id]);

    match conn.execute("DELETE FROM concepts WHERE id = ?1", rusqlite::params![id]) {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({"success": true})),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}
