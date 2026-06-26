use actix_web::{web, HttpRequest, HttpResponse};
use crate::db::DbPool;
use std::sync::Arc;

pub async fn search(
    pool: web::Data<Arc<DbPool>>,
    req: HttpRequest,
) -> HttpResponse {
    let query_params = web::Query::<std::collections::HashMap<String, String>>::from_query(req.query_string())
        .unwrap_or_else(|_| web::Query(std::collections::HashMap::new()));

    let q = match query_params.get("q") {
        Some(q) if !q.is_empty() => q.clone(),
        _ => return HttpResponse::BadRequest().json(serde_json::json!({"error": "Missing search query 'q'"})),
    };

    let conn = match pool.get() {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    };

    let search_pattern = format!("%{}%", q);

    // Search topics
    let mut topics_stmt = conn.prepare(
        "SELECT id, title, description, 'topic' as result_type FROM topics WHERE title LIKE ?1 OR description LIKE ?1 LIMIT 20"
    ).unwrap();

    let topics: Vec<serde_json::Value> = topics_stmt.query_map(rusqlite::params![search_pattern], |row| {
        Ok(serde_json::json!({
            "id": row.get::<_, String>(0)?,
            "title": row.get::<_, String>(1)?,
            "description": row.get::<_, String>(2).unwrap_or_default(),
            "type": "topic",
        }))
    }).ok().map(|r| r.filter_map(|r| r.ok()).collect()).unwrap_or_default();

    // Search concepts
    let mut concepts_stmt = conn.prepare(
        "SELECT id, name, description, topic_id, 'concept' as result_type FROM concepts WHERE name LIKE ?1 OR description LIKE ?1 LIMIT 20"
    ).unwrap();

    let concepts: Vec<serde_json::Value> = concepts_stmt.query_map(rusqlite::params![search_pattern], |row| {
        Ok(serde_json::json!({
            "id": row.get::<_, String>(0)?,
            "title": row.get::<_, String>(1)?,
            "description": row.get::<_, String>(2).unwrap_or_default(),
            "topic_id": row.get::<_, String>(3)?,
            "type": "concept",
        }))
    }).ok().map(|r| r.filter_map(|r| r.ok()).collect()).unwrap_or_default();

    // Search notes
    let mut notes_stmt = conn.prepare(
        "SELECT id, title, content, 'note' as result_type FROM notes WHERE title LIKE ?1 OR content LIKE ?1 LIMIT 20"
    ).unwrap();

    let notes: Vec<serde_json::Value> = notes_stmt.query_map(rusqlite::params![search_pattern], |row| {
        Ok(serde_json::json!({
            "id": row.get::<_, String>(0)?,
            "title": row.get::<_, String>(1)?,
            "description": row.get::<_, String>(2).unwrap_or_default(),
            "type": "note",
        }))
    }).ok().map(|r| r.filter_map(|r| r.ok()).collect()).unwrap_or_default();

    let mut results = vec![];
    results.extend(topics);
    results.extend(concepts);
    results.extend(notes);

    HttpResponse::Ok().json(serde_json::json!({
        "results": results,
        "query": q,
    }))
}
