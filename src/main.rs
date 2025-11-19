use actix_web::{App, HttpServer, middleware::from_fn, web};
use kprs_web_api::{data::{candidate::get_candidates_data, voter::get_voters_data, vote::get_votes_count}, middleware::middleware, routes::{voter::{voter_get_api, voter_vote_api}, admin::{admin_reset_api, admin_token_api}}, util::log_something};
use deadpool_redis::{Config as RedisConfig, Runtime as RedisRuntime};
use sqlx::{Pool, Postgres, postgres::PgPoolOptions};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Setup dotenv
    dotenvy::dotenv().unwrap();
    
    // Setup Static Data
    get_voters_data().await;
    get_votes_count().await;
    get_candidates_data().await;

    // Get the database URL from environment variable
    let database_url: String = std::env::var("DATABASE_URL").unwrap();
    let redis_url: String = std::env::var("REDIS_URL").unwrap();
    
    // Setup Redis
    let redis_configuration: RedisConfig = RedisConfig {
        url: Some(redis_url),
        connection: None,
        ..Default::default()
    };

    let redis_pool = redis_configuration.create_pool(Some(RedisRuntime::Tokio1)).unwrap();

    // Setup Postgres
    let postgres_pool: Pool<Postgres> = PgPoolOptions::new()
                  .max_connections(10)
                  .connect(&database_url)
                  .await
                  .unwrap();


    // Setup HTTP Server
    log_something("Setup", "Starting...");
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(redis_pool.clone()))
            .app_data(web::Data::new(postgres_pool.clone()))
            .wrap(from_fn(middleware))
            .service(voter_get_api)
            .service(voter_vote_api)
            .service(admin_reset_api)
            .service(admin_token_api)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
