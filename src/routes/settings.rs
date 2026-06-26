use actix_web::{web, HttpResponse};
use crate::db::DbPool;
use serde::Deserialize;
use std::sync::Arc;

#[derive(Deserialize)]
pub struct UpdateSettings {
    pub ai_provider: Option<String>,
    pub ai_model: Option<String>,
    pub api_key: Option<String>,
    pub api_endpoint: Option<String>,
    pub theme: Option<String>,
}

pub async fn get_settings(pool: web::Data<Arc<DbPool>>) -> HttpResponse {
    let conn = match pool.get() {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    };

    let mut stmt = match conn.prepare(
        "SELECT id, ai_provider, ai_model, api_key, api_endpoint, theme, created_at, updated_at FROM app_settings WHERE id = 'default'"
    ) {
        Ok(s) => s,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    };

    match stmt.query_row([], |row| {
        Ok(serde_json::json!({
            "id": row.get::<_, String>(0)?,
            "ai_provider": row.get::<_, String>(1).unwrap_or_else(|_| "ollama".into()),
            "ai_model": row.get::<_, String>(2).unwrap_or_else(|_| "llama3".into()),
            "api_key": row.get::<_, String>(3).unwrap_or_default(),
            "api_endpoint": row.get::<_, String>(4).unwrap_or_default(),
            "theme": row.get::<_, String>(5).unwrap_or_else(|_| "dark".into()),
            "created_at": row.get::<_, String>(6)?,
            "updated_at": row.get::<_, String>(7)?,
        }))
    }) {
        Ok(settings) => HttpResponse::Ok().json(settings),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}

pub async fn update_settings(
    pool: web::Data<Arc<DbPool>>,
    body: web::Json<UpdateSettings>,
) -> HttpResponse {
    let conn = match pool.get() {
        Ok(c) => c,
        Err(e) => return HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    };

    let mut updates = vec![];
    let mut params: Vec<Box<dyn rusqlite::types::ToSql>> = vec![];

    if let Some(ref provider) = body.ai_provider {
        updates.push("ai_provider = ?");
        params.push(Box::new(provider.clone()));
    }
    if let Some(ref model) = body.ai_model {
        updates.push("ai_model = ?");
        params.push(Box::new(model.clone()));
    }
    if let Some(ref key) = body.api_key {
        updates.push("api_key = ?");
        params.push(Box::new(key.clone()));
    }
    if let Some(ref endpoint) = body.api_endpoint {
        updates.push("api_endpoint = ?");
        params.push(Box::new(endpoint.clone()));
    }
    if let Some(ref theme) = body.theme {
        updates.push("theme = ?");
        params.push(Box::new(theme.clone()));
    }

    if updates.is_empty() {
        return HttpResponse::BadRequest().json(serde_json::json!({"error": "No fields to update"}));
    }

    updates.push("updated_at = datetime('now')");

    let sql = format!("UPDATE app_settings SET {} WHERE id = 'default'", updates.join(", "));
    let param_refs: Vec<&dyn rusqlite::types::ToSql> = params.iter().map(|p| p.as_ref()).collect();

    match conn.execute(&sql, param_refs.as_slice()) {
        Ok(_) => HttpResponse::Ok().json(serde_json::json!({"success": true})),
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({"error": e.to_string()})),
    }
}
