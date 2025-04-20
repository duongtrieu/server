use actix_web::{web, App, HttpServer, Responder, middleware::Logger};
use std::env;

fn startup() {
    println!("Server starting up...");
    env_logger::init_from_env(env_logger::Env::new().default_filter_or("info"));
}

async fn greet() -> impl Responder {
    "Hello from the API server!"
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Call the startup function
    startup();
    
    // Get port from environment or use default 8080
    let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());
    let address = format!("0.0.0.0:{}", port);
    
    println!("Server listening on {}", address);
    
    HttpServer::new(|| {
        App::new()
            .wrap(Logger::default())
            .route("/", web::get().to(greet))
    })
    .workers(2) // Adjust based on your server's capability
    .keep_alive(std::time::Duration::from_secs(120)) // Set keep-alive timeout
    .bind(&address)?
    .run()
    .await
}