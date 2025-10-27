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
use utils::view_handlers::{root,company,customer,register_meter,load_meters,register_cutomer,add_reading,generate_batch_meter,
                            load_dashboard,load_customers,load_payments,load_readings,gen_invoice,search_meters};
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
                .route("/customer-portal", get(customer))
                .route("/api/meters/register/", post(register_meter))
                .route("/api/meters/", get(load_meters))
                .route("/api/customers/", post(register_cutomer))
                .route("/api/add_readings/", post(add_reading))
                .route("/api/meters/batch/", post(generate_batch_meter))
                .route("/api/dashboard/stats/", get(load_dashboard))
                .route("/api/customers/", get(load_customers))
                .route("/api/readings/", get(load_readings))
                .route("/api/payments/", get(load_payments))
                .route("/api/invoices/generate/{reading_id}/", post(gen_invoice))
                .route("/api/customer/meter/{meter_no}/", get(search_meters))
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
