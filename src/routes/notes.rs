use actix_web::{web, HttpRequest, HttpResponse};
use crate::db::DbPool;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateNote {
    pub topic_id: Option<String>,
    pub workspace_id: Option<String>,
    pub title: Option<String>,
    pub content: Option<String>,
    pub tags: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateNote {
    pub id: String,
    pub title: Option<String>,
    pub content: Option<String>,
    pub tags: Option<String>,
    pub is_pinned: Option<bool>,
}

pub async fn list_notes(
    pool: web::Data<Arc<DbPool>>,
    req: HttpRequest,
) -> HttpResponse {
    let conn = match pool.get() {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    };

    let query_params = web::Query::<std::collections::HashMap<String, String>>::from_query(req.query_string())
        .unwrap_or_else(|_| web::Query(std::collections::HashMap::new()));

    let topic_id = query_params.get("topic_id");
    let workspace_id = query_params.get("workspace_id");

    let (sql, params): (String, Vec<Box<dyn rusqlite::types::ToSql>>) = if let Some(tid) = topic_id {
        (
            "SELECT n.id, n.topic_id, n.workspace_id, n.title, n.content, n.tags, n.is_pinned, n.created_at, n.updated_at, \
             t.title as topic_title, w.name as workspace_name \
             FROM notes n \
             LEFT JOIN topics t ON n.topic_id = t.id \
             LEFT JOIN workspaces w ON n.workspace_id = w.id \
             WHERE n.topic_id = ?1 ORDER BY n.is_pinned DESC, n.updated_at DESC".into(),
            vec![Box::new(tid.clone()) as Box<dyn rusqlite::types::ToSql>],
        )
    } else if let Some(wid) = workspace_id {
        (
            "SELECT n.id, n.topic_id, n.workspace_id, n.title, n.content, n.tags, n.is_pinned, n.created_at, n.updated_at, \
             t.title as topic_title, w.name as workspace_name \
             FROM notes n \
             LEFT JOIN topics t ON n.topic_id = t.id \
             LEFT JOIN workspaces w ON n.workspace_id = w.id \
             WHERE n.workspace_id = ?1 ORDER BY n.is_pinned DESC, n.updated_at DESC".into(),
            vec![Box::new(wid.clone()) as Box<dyn rusqlite::types::ToSql>],
        )
    } else {
        (
            "SELECT n.id, n.topic_id, n.workspace_id, n.title, n.content, n.tags, n.is_pinned, n.created_at, n.updated_at, \
             t.title as topic_title, w.name as workspace_name \
             FROM notes n \
             LEFT JOIN topics t ON n.topic_id = t.id \
             LEFT JOIN workspaces w ON n.workspace_id = w.id \
             ORDER BY n.is_pinned DESC, n.updated_at DESC".into(),
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
            "topic_id": row.get::<_, Option<String>>(1)?,
            "workspace_id": row.get::<_, Option<String>>(2)?,
            "title": row.get::<_, String>(3)?,
            "content": row.get::<_, String>(4).unwrap_or_default(),
            "tags": row.get::<_, String>(5).unwrap_or_else(|_| "[]".into()),
            "is_pinned": row.get::<_, i32>(6).unwrap_or(0) != 0,
            "created_at": row.get::<_, String>(7)?,
            "updated_at": row.get::<_, String>(8)?,
            "topic_title": row.get::<_, Option<String>>(9)?,
            "workspace_name": row.get::<_, Option<String>>(10)?,
        }))
    });

    match rows {
        Ok(rows) => {
            let notes: Vec<serde_json::Value> = rows.filter_map(|r| r.ok()).collect();
            HttpResponse::Ok().json(notes)
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

pub async fn create_note(
    pool: web::Data<Arc<DbPool>>,
    body: web::Json<CreateNote>,
) -> HttpResponse {
    let conn = match pool.get() {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    };

    let id = Uuid::new_v4().to_string();
    let title = body.title.clone().unwrap_or_else(|| "Untitled Note".into());
    let content = body.content.clone().unwrap_or_default();
    let tags = body.tags.clone().unwrap_or_else(|| "[]".into());

    match conn.execute(
        "INSERT INTO notes (id, topic_id, workspace_id, title, content, tags) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        rusqlite::params![id, body.topic_id, body.workspace_id, title, content, tags],
    ) {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "id": id,
            "title": title,
            "content": content,
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

pub async fn update_note(
    pool: web::Data<Arc<DbPool>>,
    body: web::Json<UpdateNote>,
) -> HttpResponse {
    let conn = match pool.get() {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    };

    let mut updates = vec![];
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![];

    if let Some(ref title) = body.title {
        updates.push("title = ?");
        params.push(Box::new(title.clone()));
    }
    if let Some(ref content) = body.content {
        updates.push("content = ?");
        params.push(Box::new(content.clone()));
    }
    if let Some(ref tags) = body.tags {
        updates.push("tags = ?");
        params.push(Box::new(tags.clone()));
    }
    if let Some(pinned) = body.is_pinned {
        updates.push("is_pinned = ?");
        params.push(Box::new(pinned as i32));
    }

    if updates.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({"error": "No fields to update"}));
    }

    updates.push("updated_at = datetime('now')");
    params.push(Box::new(body.id.clone()));

    let sql = format!("UPDATE notes SET {} WHERE id = ?", updates.join(", "));
    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    match conn.execute(&sql, param_refs.as_slice()) {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({"success": true})),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

pub async fn delete_note(
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

    match conn.execute("DELETE FROM notes WHERE id = ?1", rusqlite::params![id]) {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({"success": true})),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}
