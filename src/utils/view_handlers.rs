use std::sync::Arc;

use axum::{response::Html};
use sea_orm::{ActiveModelTrait, ActiveValue::Set, DatabaseConnection, EntityTrait};
use tera::{Context};
use crate::{entities::{customers, meters}, utils::helper_functions::{gen_customer_no, gen_meter_no}, TEMPLATES};
use axum::{
    Json,
    response::IntoResponse,
    extract::Extension
};
use serde_json::{json,Value};
use chrono::{Utc};

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

pub async fn register_cutomer(Extension(db): Extension<Arc<DatabaseConnection>>,Json(payload): Json<Value>) -> impl IntoResponse {
    println!("Received JSON: {}", payload);
    // Extract fields dynamically
    let customer_id = gen_customer_no(&db).await.unwrap() ;
    let name = payload.get("name").and_then(|v| v.as_str()).unwrap_or("Guest");
    let email = payload.get("email").and_then(|e| e.as_str()).unwrap_or("customer123@gmail.com");
    let meter_no = payload.get("meter_no").and_then(|m| m.as_str()).unwrap();
    let now = Utc::now().naive_utc();

    let meter_number = meter_no.to_string();
    // password automated later now a static password is configured
    let password = format!("CUS-HMI-001");

    // insert into the db
    let new_customer = customers::ActiveModel{
        id:Set(customer_id.to_owned()),
        name:Set(name.to_owned()),
        email:Set(Some(email.to_owned())),
        password:Set(password.to_owned()),
        created_at:Set(now.to_owned())
    };

    let result = new_customer.insert(&*db).await.unwrap();

    // update the meters table
    if let Some(meter) = meters::Entity::find_by_id(&meter_number).one(&*db).await.unwrap() {
        let mut active_model:meters::ActiveModel = meter.into();
        active_model.customer_id = Set(Some(customer_id.to_string()));
        // save changes
        active_model.update(&*db).await.unwrap();
    }else {
        eprintln!("Meter not found");
    }

    let response = json!({
        "message": format!("Welcome, {}!", name),
        "password":&result.password,
        "status":"Ok"
    });

    Json(response)
}

pub async fn register_meter(Extension(db): Extension<Arc<DatabaseConnection>>) ->impl IntoResponse {
    // gen meter no
    let meter_no = gen_meter_no(&db).await.unwrap();
    let now = Utc::now().naive_utc();

    // insert the data in the database
    let new_meter = meters::ActiveModel{
        id:sea_orm::ActiveValue::Set(meter_no.to_owned()),
        customer_id:sea_orm::ActiveValue::Set(None.to_owned()),
        amount:sea_orm::ActiveValue::Set(None.to_owned()),
        created_at:sea_orm::ActiveValue::Set(now.to_owned())
    };

    let result = new_meter.insert(&*db).await.unwrap();
    
    let response = json!(
        {
            "message":format!("New meter registered"),
            "meter_no":&result.id,
            "status":"okay"
        }
    );

    Json(response)
    
}

pub async fn load_meters(Extension(db): Extension<Arc<DatabaseConnection>>) ->impl IntoResponse {
    // get all the meters
    let meters = meters::Entity::find().all(&*db).await.unwrap();
    
    let response = json!({
        "message":format!("Loaded meters"),
        "meters":meters
    });

    Json(response)
}