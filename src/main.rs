pub mod ai;
pub mod db;
pub mod routes;

use actix_cors::Cors;
use actix_files as fs;
use actix_web::{web, App, HttpServer, middleware};
use log::info;
use std::sync::Arc;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    // Initialize database
    let pool = db::init_db().expect("Failed to initialize database");
    let pool = Arc::new(pool);

    info!("Database initialized at ~/TopicExplorer/.workspace/topic_explorer.db");
    info!("Starting Topic Explorer server at http://127.0.0.1:3080");

    let pool_data = web::Data::new(pool.clone());

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .wrap(middleware::Logger::default())
            .app_data(pool_data.clone())
            // API routes
            .configure(routes::configure)
            // Serve static files
            .service(fs::Files::new("/", "./static").index_file("index.html"))
    })
    .bind("127.0.0.1:3080")?
    .run()
    .await
}
