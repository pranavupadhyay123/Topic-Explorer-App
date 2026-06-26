use actix_web::{web, HttpResponse};
use crate::db::DbPool;
use std::sync::Arc;

pub async fn health_check(pool: web::Data<Arc<DbPool>>) -> HttpResponse {
    match pool.get() {
        Ok(conn) => {
            match conn.execute_batch("SELECT 1") {
                Ok(_) => HttpResponse::Ok().json(serde_json::json!({
                    "status": "ok",
                    "database": "connected",
                    "workspace": crate::db::get_workspace_dir().display().to_string()
                })),
                Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
                    "status": "error",
                    "database": format!("error: {}", e)
                })),
            }
        }
        Err(e) => HttpResponse::InternalServerError().json(serde_json::json!({
            "status": "error",
            "database": format!("pool error: {}", e)
        })),
    }
}
