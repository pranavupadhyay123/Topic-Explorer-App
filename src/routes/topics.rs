use actix_web::{web, HttpRequest, HttpResponse};
use crate::db::DbPool;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateTopic {
    pub title: String,
    pub workspace_id: Option<String>,
    pub description: Option<String>,
}

pub async fn list_topics(
    pool: web::Data<Arc<DbPool>>,
    req: HttpRequest,
) -> HttpResponse {
    let conn = match pool.get() {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    };

    let query_params = web::Query::<std::collections::HashMap<String, String>>::from_query(req.query_string())
        .unwrap_or_else(|_| web::Query(std::collections::HashMap::new()));

    let workspace_id = query_params.get("workspace_id");

    let (sql, params): (String, Vec<Box<dyn rusqlite::types::ToSql>>) = if let Some(wid) = workspace_id {
        (
            "SELECT t.id, t.workspace_id, t.title, t.description, t.status, t.explored_at, t.created_at, t.updated_at, \
             w.name as workspace_name, w.color as workspace_color, w.icon as workspace_icon \
             FROM topics t LEFT JOIN workspaces w ON t.workspace_id = w.id \
             WHERE t.workspace_id = ?1 ORDER BY t.created_at DESC".into(),
            vec![Box::new(wid.clone()) as Box<dyn rusqlite::types::ToSql>],
        )
    } else {
        (
            "SELECT t.id, t.workspace_id, t.title, t.description, t.status, t.explored_at, t.created_at, t.updated_at, \
             w.name as workspace_name, w.color as workspace_color, w.icon as workspace_icon \
             FROM topics t LEFT JOIN workspaces w ON t.workspace_id = w.id \
             ORDER BY t.created_at DESC".into(),
            vec![],
        )
    };

    let mut stmt = match conn.prepare(&sql) {
        Ok(s) => s,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    };

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    let rows = stmt.query_map(param_refs.as_slice(), |row| {
        Ok(serde_json::json!({
            "id": row.get::<_, String>(0)?,
            "workspace_id": row.get::<_, Option<String>>(1)?,
            "title": row.get::<_, String>(2)?,
            "description": row.get::<_, String>(3).unwrap_or_default(),
            "status": row.get::<_, String>(4).unwrap_or_else(|_| "pending".into()),
            "explored_at": row.get::<_, Option<String>>(5)?,
            "created_at": row.get::<_, String>(6)?,
            "updated_at": row.get::<_, String>(7)?,
            "workspace_name": row.get::<_, Option<String>>(8)?,
            "workspace_color": row.get::<_, Option<String>>(9)?,
            "workspace_icon": row.get::<_, Option<String>>(10)?,
        }))
    });

    match rows {
        Ok(rows) => {
            let topics: Vec<serde_json::Value> = rows.filter_map(|r| r.ok()).collect();
            HttpResponse::Ok().json(topics)
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

pub async fn create_topic(
    pool: web::Data<Arc<DbPool>>,
    body: web::Json<CreateTopic>,
) -> HttpResponse {
    let conn = match pool.get() {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    };

    let id = Uuid::new_v4().to_string();
    let desc = body.description.clone().unwrap_or_default();

    match conn.execute(
        "INSERT INTO topics (id, workspace_id, title, description) VALUES (?1, ?2, ?3, ?4)",
        rusqlite::params![id, body.workspace_id, body.title, desc],
    ) {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "id": id,
            "title": body.title,
            "workspace_id": body.workspace_id,
            "description": desc,
            "status": "pending",
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

pub async fn delete_topic(
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

    // Delete cascading data
    let _ = conn.execute("DELETE FROM concepts WHERE topic_id = ?1", rusqlite::params![id]);
    let _ = conn.execute("DELETE FROM relationships WHERE topic_id = ?1", rusqlite::params![id]);
    let _ = conn.execute("DELETE FROM knowledge_cards WHERE topic_id = ?1", rusqlite::params![id]);
    let _ = conn.execute("DELETE FROM timeline_events WHERE topic_id = ?1", rusqlite::params![id]);
    let _ = conn.execute("DELETE FROM flashcards WHERE topic_id = ?1", rusqlite::params![id]);
    let _ = conn.execute("DELETE FROM learning_paths WHERE topic_id = ?1", rusqlite::params![id]);
    let _ = conn.execute("DELETE FROM conversations WHERE topic_id = ?1", rusqlite::params![id]);
    let _ = conn.execute("DELETE FROM bookmarks WHERE topic_id = ?1", rusqlite::params![id]);

    match conn.execute("DELETE FROM topics WHERE id = ?1", rusqlite::params![id]) {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({"success": true})),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}
