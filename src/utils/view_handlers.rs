use std::{sync::Arc, u32};

use axum::{response::Html};
use sea_orm::{ActiveModelTrait, ActiveValue::{NotSet, Set}, ColumnTrait, DatabaseConnection, EntityTrait, QueryFilter};
use serde::{Deserialize, Serialize};
use tera::{Context};
use crate::{entities::{customers, meters, payments, readings}, utils::helper_functions::{gen_customer_no, gen_meter_no}, TEMPLATES};
use axum::{
    Json,
    response::IntoResponse,
    extract::Extension
};
use serde_json::{json,Value};
use chrono::{Datelike, NaiveDate, Utc};
use crate::entities::prelude::*;
use sea_orm::prelude::Decimal;

#[derive(Serialize,Deserialize)]
struct CustomerDashboard{
    id:String,
    name:String,
    email:Option<String>,
    meter_no:Option<String>,
    
}

pub async fn root() ->&'static str {
    "Hello from Hephaestus Motor Inc"
}

pub async  fn company() ->Html<String> {
    // let tera = Tera::new("src/templates/**/*.html").unwrap();
    let mut ctx = Context::new();
    ctx.insert("name", "Hephaestus Motor Inc");
    // retrieve all the meters no that have not been assigned to a customer

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
    let meters = meters::Entity::find()
        .find_also_related(customers::Entity)
        .all(&*db)
        .await.
        unwrap();
    
    let data: Vec<_> = meters
        .into_iter()
        .map(|(meter, customer)| {
            json!({
                "id": meter.id,
                "customer_name": customer.as_ref().map(|c| &c.name),
                "status": "Active",           // temporary placeholder
                "last_reading": 4,            // placeholder
                "meter_no": meter.id
            })
        })
        .collect();
    
    let response = json!({
        "message":format!("Loaded meters"),
        "meters":data
    });

    Json(response)
}

pub async fn add_reading(Extension(db): Extension<Arc<DatabaseConnection>>,Json(payload): Json<Value>) ->impl IntoResponse {
    let meter_no = payload.get("meter_no").and_then(|m|m.as_str()).unwrap().to_string() ;
    let meter_reading = payload.get("reading").and_then(|r|r.as_i64()).unwrap();
    let period = payload.get("period").and_then(|p| p.as_str()).unwrap_or("Unknown").to_string() ;
    let timestamp = Utc::now().naive_utc();

    let new_reading = readings::ActiveModel{
        id:NotSet,
        meter_id:Set(meter_no),
        units:Set(meter_reading as i32),
        timestamp:Set(timestamp),
        period:Set(period)
    };

    let result = new_reading.insert(&*db).await.unwrap();

    let response = json!({
        "message":format!("New reading added"),
        "data":result
    });

    Json(response)

}

pub async fn generate_batch_meter(Extension(db): Extension<Arc<DatabaseConnection>>) ->impl IntoResponse {
    // generate 10 meter numbers
    let mut meters = vec![];
    let meter_no = gen_meter_no(&db).await.unwrap();
    let last_no = meter_no.trim_start_matches("HMI-");
    let mut last_no_as_u32 = last_no.parse::<u32>().unwrap_or(0);

    for _ in 0..=10 {
        let new_meter_no = format!("HMI-{:03}",last_no_as_u32);
        let now = Utc::now().naive_utc();

        // insert the data in the database
        let new_meter = meters::ActiveModel{
            id:sea_orm::ActiveValue::Set(new_meter_no.to_owned()),
            customer_id:sea_orm::ActiveValue::Set(None.to_owned()),
            amount:sea_orm::ActiveValue::Set(None.to_owned()),
            created_at:sea_orm::ActiveValue::Set(now.to_owned())
        };

        meters.push(new_meter);
        last_no_as_u32 +=1;  
    }

    let _result = Meters::insert_many(meters).exec(&*db).await.unwrap();
    let response = json!({
        "message":format!("Created 10 meters")
    });

    Json(response)
}

pub async fn load_dashboard(Extension(db): Extension<Arc<DatabaseConnection>>) ->impl IntoResponse {
    // total customers
    let customers_query = Customers::find().all(&*db).await.unwrap();
    let total_customers = customers_query.len();

    // active meters ie those assigned
    let active_meters = Meters::find()
        .filter(meters::Column::CustomerId.is_not_null())
        .all(&*db)
        .await
        .unwrap().len();

    // monthly revenue
    let date_timestamp = Utc::now();
    let date = &date_timestamp.day();
    let month = &date_timestamp.month();
    let year = &date_timestamp.year();

    // constructing the start and end date
    let start_date = NaiveDate::from_ymd_opt(year.clone(), month.clone(), 1).unwrap();
    let end_date = NaiveDate::from_ymd_opt(year.clone(), month.clone(), date.clone()).unwrap();


    let monthly_payments = Payments::find()
        // remove & references — SeaORM expects the value, not a reference
        .filter(payments::Column::CreatedAt.between(start_date, end_date))
        .all(&*db)
        .await
        .unwrap();

    // Assuming `amount` is NOT an Option<Decimal> (i.e. it's required)
    let total_monthly_revenue: Decimal = monthly_payments
        .iter()
        .map(|t| t.amount)
        .sum();

    // pending payments ie those inovices that are not referenced in payments
    let pending_invoices = Invoices::find()
        .left_join(payments::Entity)
        .filter(payments::Column::InvoiceId.is_null())
        .all(&*db)
        .await
        .unwrap();

    let total_pending_payments:Decimal = pending_invoices
        .iter()
        .map(|inv|inv.amount)
        .sum();

    let response = json!({
        "total_customers":total_customers,
        "active_meters":active_meters,
        "monthly_revenue":total_monthly_revenue,
        "pending_payments":total_pending_payments
    });

    Json(response)


}

pub async fn load_customers(Extension(db): Extension<Arc<DatabaseConnection>>) ->impl IntoResponse {
    let customers = Customers::find()
        .find_also_related(meters::Entity)
        .all(&*db)
        .await
        .unwrap();
    
    let mut customers_response = vec![];

    for (customer, meter) in customers {
        customers_response.push(CustomerDashboard {
            id: customer.id,
            name: customer.name,
            email: customer.email,
            meter_no: meter.map(|m| m.id), // ✅ safely extract from Option<meter::Model>
        });
    }

     let response = json!({
        "message": "Loaded customers with their meters",
        "data": customers_response
    });

    Json(response)
}