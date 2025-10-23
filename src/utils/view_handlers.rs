use axum::response::Html;
use sea_orm::DatabaseConnection;
use tera::{Context};
use crate::TEMPLATES;
use axum::{
    Json,
    response::IntoResponse
};
use serde_json::{json,Value};

pub async fn root() ->&'static str {
    "Hello from Hephaestus Motor Inc"
}

pub async  fn company() ->Html<String> {
    // let tera = Tera::new("src/templates/**/*.html").unwrap();
    let mut ctx = Context::new();
    ctx.insert("name", "Hephaestus Motor Inc");

    let rendered = TEMPLATES.render("company_portal.html", &ctx).unwrap();
    Html(rendered)
}

pub async fn register_cutomer(db:&DatabaseConnection,Json(payload): Json<Value>) -> impl IntoResponse {
    println!("Received JSON: {}", payload);
    // Extract fields dynamically
    // let name = payload.get("name").and_then(|v| v.as_str()).unwrap_or("Guest");
    // let age = payload.get("age").and_then(|v| v.as_i64()).unwrap_or(-1);

    // // Build response JSON dynamically
    // let response = json!({
    //     "message": format!("Welcome, {}!", name),
    //     "age_received": age,
    //     "payload": payload, // Echo back the original JSON
    //     "status": "ok"
    // });

    // Json(response)
}

pub async fn register_meter() {
    
}