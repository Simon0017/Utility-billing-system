mod entities;
mod utils;
use sea_orm::{sqlx::Result, *};
use std::env;
use std::net::SocketAddr;
use dotenvy::dotenv;
use tower_http::cors::{CorsLayer,Any};
use tower_http::trace::{TraceLayer};
use axum::{
    routing::{post,get},
    Router,
    extract::Extension
};
use utils::view_handlers::{root,company,register_meter,load_meters,register_cutomer};
use tera::{Tera};
use once_cell::sync::Lazy;
use tower_http::services::ServeDir;
use std::path::PathBuf;
use tracing_subscriber;
use std::sync::Arc;


// make templates lazy and global
pub static TEMPLATES:Lazy<Tera>= Lazy::new(||{
                        match Tera::new("src/templates/**/*.html") {
                            Ok(t) =>t,
                            Err(e) =>{
                                eprintln!("Template Parsing Error: {e}");
                                std::process::exit(1);
                            }
                        }
                    }                    
                );

// serve static files

#[tokio::main]
async  fn main()->Result<(),DbErr> {
    // load the env vars
    dotenv().ok();

    // ==============SETTING UP THE DB================================
    // get the db url
    let db_url = env::var("DATABASE_URL").expect("Database url must be in the .env file");

    let db = Database::connect(&db_url).await?;
    let db = Arc::new(db); //shared ownership

    // ++++++++++++++++++++++++LOGGING SETUP+++++++++++++++++++++++++
    tracing_subscriber::fmt().with_env_filter("info").init();

    // =======SETTING UP THE MIDDLEWARE=========================================
    let cors = CorsLayer::new()
                        .allow_origin(Any)
                        .allow_methods(Any)
                        .allow_headers(Any);

    // +++++++++++++++++++++++++=DEFINING THE STATIC RENDERING+++++++++++++++++++++++
    // Absolute path to staic dir
    let static_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/static");

    // Serve static files at /static/*
    let static_service = ServeDir::new(static_dir);

    
    // ++++++++++++++++++++++++++++++++++DEFINE ROUTES++++++++++++++++++++++++++++++
    let app = Router::new()
                                .route("/", get(root))
                                .route("/company-portal", get(company))
                                .route("/api/meters/register/", post(register_meter))
                                .route("/api/meters/", get(load_meters))
                                .route("/api/customers/", post(register_cutomer))
                                .layer(Extension(db))
                                .nest_service("/static", static_service)
                                .layer(cors)
                                .layer(TraceLayer::new_for_http()) ; //for logging


    // +++++++++++++++++++++++++SERVER SETUP+++++++++++++++++++++++++++++++++++++++++++
    let addr = SocketAddr::from(([127,0,0,1],3000));
    println!("Server is running on http://{}",addr);

    axum::serve(tokio::net::TcpListener::bind(&addr).await.unwrap(), app)
            .await
            .unwrap();

    Ok(())
}
