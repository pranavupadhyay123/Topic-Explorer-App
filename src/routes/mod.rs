pub mod ai_routes;
pub mod concepts;
pub mod health;
pub mod notes;
pub mod search;
pub mod settings;
pub mod topics;
pub mod workspaces;

use actix_web::web;

/// Register all API routes
pub fn configure(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/api")
            // Health
            .route("/health", web::get().to(health::health_check))
            // Workspaces
            .route("/workspaces", web::get().to(workspaces::list_workspaces))
            .route("/workspaces", web::post().to(workspaces::create_workspace))
            .route("/workspaces", web::put().to(workspaces::update_workspace))
            .route("/workspaces", web::delete().to(workspaces::delete_workspace))
            // Topics
            .route("/topics", web::get().to(topics::list_topics))
            .route("/topics", web::post().to(topics::create_topic))
            .route("/topics", web::put().to(topics::update_topic_workspace))
            .route("/topics", web::delete().to(topics::delete_topic))
            // Concepts
            .route("/concepts", web::get().to(concepts::list_concepts))
            .route("/concepts", web::post().to(concepts::create_concept))
            .route("/concepts", web::put().to(concepts::update_concept))
            .route("/concepts", web::delete().to(concepts::delete_concept))
            // Notes
            .route("/notes", web::get().to(notes::list_notes))
            .route("/notes", web::post().to(notes::create_note))
            .route("/notes", web::put().to(notes::update_note))
            .route("/notes", web::delete().to(notes::delete_note))
            // Settings
            .route("/settings", web::get().to(settings::get_settings))
            .route("/settings", web::put().to(settings::update_settings))
            // Search
            .route("/search", web::get().to(search::search))
            // AI routes
            .route("/ai/explore", web::post().to(ai_routes::explore))
            .route("/ai/explain", web::post().to(ai_routes::explain))
            .route("/ai/quiz", web::post().to(ai_routes::quiz))
            .route("/ai/tutor", web::post().to(ai_routes::tutor))
            .route("/ai/models", web::post().to(ai_routes::models)),
    );
}
