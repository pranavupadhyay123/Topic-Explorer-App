use actix_web::{web, HttpRequest, HttpResponse};
use crate::db::DbPool;
use serde::Deserialize;
use std::sync::Arc;
use uuid::Uuid;

#[derive(Deserialize)]
pub struct CreateWorkspace {
    pub name: String,
    pub description: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
}

#[derive(Deserialize)]
pub struct UpdateWorkspace {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub color: Option<String>,
    pub icon: Option<String>,
}

#[derive(Deserialize)]
pub struct DeleteQuery {
    pub id: String,
}

pub async fn list_workspaces(pool: web::Data<Arc<DbPool>>) -> HttpResponse {
    let conn = match pool.get() {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    };

    let mut stmt = match conn.prepare(
        "SELECT id, name, description, color, icon, created_at, updated_at FROM workspaces ORDER BY created_at DESC"
    ) {
        Ok(s) => s,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    };

    let rows = stmt.query_map([], |row| {
        Ok(serde_json::json!({
            "id": row.get::<_, String>(0)?,
            "name": row.get::<_, String>(1)?,
            "description": row.get::<_, String>(2).unwrap_or_default(),
            "color": row.get::<_, String>(3).unwrap_or_else(|_| "#6366f1".into()),
            "icon": row.get::<_, String>(4).unwrap_or_else(|_| "📚".into()),
            "created_at": row.get::<_, String>(5)?,
            "updated_at": row.get::<_, String>(6)?,
        }))
    });

    match rows {
        Ok(rows) => {
            let workspaces: Vec<serde_json::Value> = rows.filter_map(|r| r.ok()).collect();
            HttpResponse::Ok().json(workspaces)
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

pub async fn create_workspace(
    pool: web::Data<Arc<DbPool>>,
    body: web::Json<CreateWorkspace>,
) -> HttpResponse {
    let conn = match pool.get() {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    };

    let id = Uuid::new_v4().to_string();
    let color = body.color.clone().unwrap_or_else(|| "#6366f1".into());
    let icon = body.icon.clone().unwrap_or_else(|| "📚".into());
    let desc = body.description.clone().unwrap_or_default();

    match conn.execute(
        "INSERT INTO workspaces (id, name, description, color, icon) VALUES (?1, ?2, ?3, ?4, ?5)",
        rusqlite::params![id, body.name, desc, color, icon],
    ) {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({
            "id": id,
            "name": body.name,
            "description": desc,
            "color": color,
            "icon": icon,
        })),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

pub async fn update_workspace(
    pool: web::Data<Arc<DbPool>>,
    body: web::Json<UpdateWorkspace>,
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
    if let Some(ref color) = body.color {
        updates.push("color = ?");
        params.push(Box::new(color.clone()));
    }
    if let Some(ref icon) = body.icon {
        updates.push("icon = ?");
        params.push(Box::new(icon.clone()));
    }

    if updates.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({"error": "No fields to update"}));
    }

    updates.push("updated_at = datetime('now')");
    params.push(Box::new(body.id.clone()));

    let sql = format!(
        "UPDATE workspaces SET {} WHERE id = ?",
        updates.join(", ")
    );

    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    match conn.execute(&sql, param_refs.as_slice()) {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({"success": true})),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

pub async fn delete_workspace(
    pool: web::Data<Arc<DbPool>>,
    req: HttpRequest,
) -> HttpResponse {
    let id = match req.headers().get("x-id") {
        Some(v) => v.to_str().unwrap_or("").to_string(),
        None => {
            // Try query string
            let query = web::Query::<std::collections::HashMap<String, String>>::from_query(req.query_string());
            match query {
                Ok(q) => q.get("id").cloned().unwrap_or_default(),
                Err(_) => return HttpResponse::BadRequest().json(serde_json::json!({"error": "Missing id"})),
            }
        }
    };

    if id.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({"error": "Missing id"}));
    }

    let conn = match pool.get() {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    };

    // Delete workspace and cascade (topics will cascade to concepts, relationships, etc.)
    let _ = conn.execute("UPDATE topics SET workspace_id = NULL WHERE workspace_id = ?1", rusqlite::params![id]);
    let _ = conn.execute("UPDATE notes SET workspace_id = NULL WHERE workspace_id = ?1", rusqlite::params![id]);

    match conn.execute("DELETE FROM workspaces WHERE id = ?1", rusqlite::params![id]) {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({"success": true})),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}
