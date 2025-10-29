use std::{str, sync::Arc, u32};

use axum::{response::Html,extract::{Query,Path}};
use sea_orm::{sea_query::{Expr}, ActiveModelTrait, ActiveValue::{NotSet, Set}, ColumnTrait, DatabaseConnection, EntityTrait, ExprTrait, QueryFilter, QueryOrder};
use serde::{Deserialize, Serialize};
use tera::{Context};
use crate::{entities::{customers, invoices, meters, payments, readings}, utils::helper_functions::{calculate_invoice_amount, gen_customer_no, gen_invoice_no, gen_meter_no, generate_payment_id, RATE_PER_UNIT, SERVICE_CHARGE}, TEMPLATES};
use axum::{
    Json,
    response::IntoResponse,
    extract::Extension
};
use serde_json::{json,Value};
use chrono::{Datelike, NaiveDate, NaiveDateTime, Utc};
use crate::entities::prelude::*;
use rust_decimal::{prelude::ToPrimitive, Decimal};
use rust_decimal::prelude::FromPrimitive;

#[derive(Serialize,Deserialize)]
struct CustomerDashboard{
    id:String,
    name:String,
    email:Option<String>,
    meter_no:Option<String>,
    
}

#[derive(Serialize,Deserialize)]
pub struct PaymentQuery{
    filter:Option<String>
}

#[derive(Serialize,Deserialize)]
pub struct LatestInvoice{
    invoice_no:String,
    date:NaiveDateTime,
    customer_name:String,
    meter_no:String,
    period:String,
    consumption:i32,
    rate:i32,
    subtotal:i32,
    service_charge:i32,
    total:Decimal
}

#[derive(Serialize,Deserialize)]
pub struct ReadingHistory {
    pub history_id: i32,
    pub period: String,
    pub reading: i32,
    pub consumption: i32,
    pub amount: Decimal,
    pub status: String,
}

#[derive(Serialize,Deserialize)]
pub struct TrendsData{
    labels:Vec<String>,
    values:Vec<Decimal>
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

pub async fn customer() ->Html<String> {
    let mut ctx = Context::new();
    ctx.insert("name", "Hephaestus Motor Inc");

    let rendered = TEMPLATES.render("customer-portal.html", &ctx).unwrap();
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
        .order_by_asc(meters::Column::Id)
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
    let meter_reading = payload.get("reading").and_then(|r|r.as_i64()).unwrap_or(0);
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

pub async fn generate_batch_meter(Extension(db): Extension<Arc<DatabaseConnection>>,Json(payload): Json<Value>) ->impl IntoResponse {
    // generate 10 meter numbers load from the count
    let count = payload.get("count")
        .and_then(|c|c.as_i64()).unwrap_or(10);

    let mut meters = vec![];
    let meter_no = gen_meter_no(&db).await.unwrap();
    let last_no = meter_no.trim_start_matches("HMI-");
    let mut last_no_as_u32 = last_no.parse::<u32>().unwrap_or(0);

    for _ in 0..=count {
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

pub async fn load_readings(Extension(db): Extension<Arc<DatabaseConnection>>) ->impl IntoResponse {
    let readings = Readings::find()
            .find_also_related(meters::Entity)
            .all(&*db)
            .await
            .unwrap();
    
    let data:Vec<_> = readings
        .into_iter()
        .map(|(reading,meter)|{
            let formatted_period = NaiveDate::parse_from_str(&reading.period, "%Y-%m-%d")
                .map(|date|format!("{} {}",date.format("%B"),date.format("%Y")))
                .unwrap_or_else(|_|reading.period.clone());
            let readings_formated = reading.units as i32;
            let amount = calculate_invoice_amount(&readings_formated).unwrap();
            // let formatted_timestamp = reading.timestamp.format("%B")

            json!({
                "meter_no":meter.as_ref().map(|m| &m.id),
                "reading":reading.units,
                "period":formatted_period,
                "amount":amount,
                "date":reading.timestamp,
            })
        })
        .collect();

    let response = json!({
        "message":format!("Loaded meters"),
        "data":data
    });

    Json(response)
}

pub async fn load_payments(Extension(db): Extension<Arc<DatabaseConnection>>,Query(params): Query<PaymentQuery>) ->impl IntoResponse {
    let filter = params.filter.unwrap_or_else(||"all".to_string());

    let payments = match filter.as_str() {
        "all" =>{
            let payments = Payments::find()
                .find_also_related(customers::Entity)
                .all(&*db)
                .await
                .unwrap();

            let data:Vec<_> = payments.into_iter().map(|(payment,customer)|{
                json!({
                    "customer_name":customer.as_ref().map(|c|&c.name),
                    "amount":payment.amount,
                    "status":"All".to_string(),
                    "date":payment.created_at,
                    "id":payment.id
                })
            }).collect();

            data
        },
        "pending" => {
            let payments = Payments::find()
            .left_join(Invoices)
            .find_also_related(customers::Entity)
            .filter(Expr::col(invoices::Column::Id).is_null())
            .all(&*db)
            .await
            .unwrap();

            let data:Vec<_> = payments.into_iter().map(|(payment,customer)|{
                json!({
                    "customer_name":customer.as_ref().map(|c|&c.name),
                    "amount":payment.amount,
                    "status":"Pending".to_string(),
                    "date":payment.created_at,
                    "id":payment.id
                })
            }).collect();

            data

        },
        "completed" => {
            let payments = Payments::find()
            .left_join(Invoices)
            .find_also_related(customers::Entity)
            .filter(Expr::col(invoices::Column::Id).is_not_null())
            .all(&*db)
            .await
            .unwrap();

            let data:Vec<_> = payments.into_iter().map(|(payment,customer)|{
                json!({
                    "customer_name":customer.as_ref().map(|c|&c.name),
                    "amount":payment.amount,
                    "status":"Completed".to_string(),
                    "date":payment.created_at,
                    "id":payment.id
                })
            }).collect();

            data
        },
        "defaulters" => {
            //modify later so that defaulters are pending after 5 months from today
            let  month = Utc::now().month();
            let  year = Utc::now().year();

            let defaulting_date = if month <=5 {
                let default_month = &month +12 - 5;
                let default_year = &year - 1;
                NaiveDate::from_ymd_opt(default_year, default_month, 30)
            }else{
                NaiveDate::from_ymd_opt(year.clone(), month.clone() -5, 30)
            };
            
            let payments = Payments::find()
            .left_join(Invoices)
            .find_also_related(customers::Entity)
            .filter(Expr::col(invoices::Column::Id).is_null())
            .filter(payments::Column::CreatedAt.gt(defaulting_date))
            .all(&*db)
            .await
            .unwrap();

            let data:Vec<_> = payments.into_iter().map(|(payment,customer)|{
                json!({
                    "customer_name":customer.as_ref().map(|c|&c.name),
                    "amount":payment.amount,
                    "status":"Defaulted".to_string(),
                    "date":payment.created_at,
                    "id":payment.id
                })
            }).collect();

            data
        },
        _ => {
            let payments = Payments::find()
                .find_also_related(customers::Entity)
                .all(&*db)
                .await
                .unwrap();
            let data:Vec<_> = payments.into_iter().map(|(payment,customer)|{
                json!({
                    "customer_name":customer.as_ref().map(|c|&c.name),
                    "amount":payment.amount,
                    "status":"All".to_string(),
                    "date":payment.created_at,
                    "id":payment.id
                })
            }).collect();

            data
        },
    };

    Json(payments)


}

pub async fn gen_invoice(Extension(db): Extension<Arc<DatabaseConnection>>,Path(reading_id): Path<String>,Json(payload): Json<Value>) ->impl IntoResponse {
    let meter_no = reading_id;
    let amount = payload.get("amount").and_then(|a| a.as_f64()).unwrap_or(0.0);
    let invoice_id = gen_invoice_no(&*db).await.unwrap();
    let now = Utc::now().naive_utc() ;

    // prevent duplicate invoices


    // get the customer and the amount in the readings
    // 1. get the customer 
    let meter = Meters::find()
        .filter(meters::Column::Id.contains(&meter_no))
        .one(&*db)
        .await
        .unwrap();
    if let Some(meter) = meter  {
        let customer_id = meter.customer_id;
        if let Some(id) = customer_id  {
            // insert into the db 
             let invoice = invoices::ActiveModel{
                id:Set(invoice_id),
                customer_id:Set(id),
                amount:Set(Decimal::from_f64(amount).unwrap_or(Decimal::ZERO)),
                created_at:Set(now)
            };

            let _result = invoice.insert(&*db).await.unwrap();

        }
    }

    let response = json!({
        "message":format!("Generate a new invoice for {}",&meter_no)
    });

    Json(response)

   
}

pub async fn search_meters(Extension(db): Extension<Arc<DatabaseConnection>>,Path(meter_no): Path<String>) ->impl IntoResponse {
    /* Data:
    -balance
    -last_payment
    -latest_reading
    -latest_invoice
        -invoice_no
        -date
        -customer_name
        -meter_no
        -period
        -consumption
        -rate
        -subtotal
        -service_charge
        -total
    -history
        -period
        -reading
        -consumption
        -amount
        -status
    -consumption_trend
        -labels vec 
        -values  vec (units)
    -payment_history
        -labels vec
        -values vec (amount)

     */
    let meter_no = meter_no;
    let meter = Meters::find()
        .filter(meters::Column::Id.contains(&meter_no))
        .one(&*db)
        .await
        .unwrap();

    let customers_id = meter.clone().and_then(|m|m.customer_id).unwrap_or_else(||"Unknown".to_string());
    let customer = Customers::find()
        .filter(customers::Column::Id.contains(customers_id.clone()))
        .one(&*db)
        .await
        .unwrap();

    let customer_name = customer.clone().and_then(|c|Some(c.name)).unwrap_or_else(||"Anon".to_string());
    
    // balance
    let invoices_payments = Invoices::find()
        .find_also_related(payments::Entity)
        .filter(invoices::Column::CustomerId.contains(customers_id.clone()))
        .all(&*db)
        .await
        .unwrap();

    let balance:Decimal = invoices_payments.into_iter().map(|(invoice,payment)|{
        // if payment is found for that invoice the balance of the payment model is eval,if no bal_amount then default 
        // to 0 if no payment found for the invoice then take the invoice amt as the amount
       match payment {
            // case 1: payment exists → use its balance or default to 0
            Some(pay) => pay
                .bal_amount
                .unwrap_or(Decimal::ZERO),

            // case 2: no payment → use the invoice amount
            None => invoice.amount
        }
    })
    .sum();

    // last payment
    let last_payment_model = Payments::find()
        .filter(payments::Column::CustomerId.contains(customers_id.clone()))
        .order_by_desc(payments::Column::CreatedAt)
        .one(&*db)
        .await
        .unwrap();

    let latest_payment: Decimal = last_payment_model.and_then(|l|Some(l.amount)).unwrap_or(Decimal::ZERO);

    // latest_reading
    let last_reading_model = Readings::find()
        .filter(readings::Column::MeterId.contains(meter_no.clone()))
        .order_by_desc(readings::Column::Timestamp)
        .one(&*db)
        .await
        .unwrap();
    let last_reading = last_reading_model.and_then(|l|Some(l.units)).unwrap_or(0);

    // latest invoice
    let invoice = Invoices::find()
        .order_by_desc(invoices::Column::CreatedAt)
        .filter(invoices::Column::CustomerId.contains(customers_id.clone()))
        .one(&*db)
        .await
        .unwrap();

    let invoice_data = LatestInvoice{
        invoice_no:invoice.as_ref().map(|inv|inv.id.clone()).unwrap_or("Unknown".to_string()),
        customer_name:customer_name.clone(),
        meter_no:meter_no.clone(),
        rate:RATE_PER_UNIT,
        service_charge:SERVICE_CHARGE,
        consumption:invoice.as_ref().map(|inv|{
            let amount = Decimal::to_i32(&inv.amount).unwrap_or(0);
            let consumption = amount - SERVICE_CHARGE;
            consumption / RATE_PER_UNIT
        }).unwrap_or(0),
        date:invoice.as_ref().map(|inv|inv.created_at).unwrap_or_else(||Utc::now().naive_utc()),
        period:invoice.as_ref().map(|inv|inv.created_at.month()).unwrap_or_else(||Utc::now().month()).to_string(),
        subtotal:invoice.as_ref().map(|inv|{
            let amount = Decimal::to_i32(&inv.amount).unwrap_or(0);
            amount - SERVICE_CHARGE
        }).unwrap_or(0),
        total:invoice.as_ref().map(|inv|inv.amount).unwrap_or(Decimal::ZERO)

    };

    // history
    let readings_with_meters = Readings::find()
        .find_also_related(meters::Entity)
        .filter(readings::Column::MeterId.contains(meter_no.clone()))
        .all(&*db)
        .await
        .unwrap();
    
    let mut history_data = Vec::new();

    // consumption trend
    let mut consumption_labels = Vec::new();
    let mut consumptions_values = Vec::new();

    for (reading, maybe_meter) in readings_with_meters {
        if let Some(meter) = maybe_meter {

            // collect consumption data
            consumption_labels.push(reading.period.clone());
            consumptions_values.push(Decimal::from(reading.units));

            // Get the customer for this meter
            let customer = Customers::find_by_id(meter.customer_id.clone().unwrap_or_default())
                .one(&*db)
                .await
                .unwrap();

            // Find latest payment for that customer (optional)
            let payment = if let Some(cust) = &customer {
                Payments::find()
                    .filter(payments::Column::CustomerId.eq(cust.id.clone()))
                    .order_by_desc(payments::Column::CreatedAt)
                    .one(&*db)
                    .await
                    .unwrap()
            } else {
                None
            };

            // Compute the fields
            let reading_val = reading.units;
            let amount = meter.amount.unwrap_or(Decimal::ZERO);
            let consumption = reading_val;

            let status = if let Some(p) = payment {
                if p.amount >= amount {
                    "Paid".to_string()
                } else {
                    "Pending".to_string()
                }
            } else {
                "Unpaid".to_string()
            };

            history_data.push(ReadingHistory {
                history_id: reading.id,
                period: reading.period,
                reading: reading.units,
                consumption,
                amount,
                status,
            });
        }
    }

    // payment trend
    let mut payment_labels = Vec::new();
    let mut payment_values = Vec::new();

    // fetch the payments related to the customer
    let payments = Payments::find()
        .filter(payments::Column::CustomerId.contains(customers_id.clone()))
        .order_by_asc(payments::Column::CreatedAt)
        .all(&*db)
        .await
        .unwrap();

    for pmt in payments{
        payment_labels.push(pmt.created_at.to_string());
        payment_values.push(pmt.amount);
    }



    // ++++++++++++++++++JSON RESPONSE+++++++++++++++++++
    let response = json!({
        "balance":balance,
        "last_payment":latest_payment,
        "latest_reading":last_reading,
        "latest_invoice":invoice_data,
        "history":history_data,
        "consumption_trend":TrendsData{
            labels:consumption_labels,
            values:consumptions_values
        },
        "payment_history":TrendsData{
            labels:payment_labels,
            values:payment_values
        }
    });

    Json(response)

}

pub async  fn process_payments(Extension(db): Extension<Arc<DatabaseConnection>>,Json(payload): Json<Value>) ->impl IntoResponse {
    let meter_no = payload.get("meter_no")
        .and_then(|m|m.as_str())
        .unwrap_or("Unknown")
        .to_string();
    let amount = payload.get("amount")
        .and_then(|a|a.as_f64())
        .unwrap_or(0.0);

    // get the customer_id for the meter and the invoice_id
    let meter = Meters::find()
        .filter(meters::Column::Id.contains(&meter_no))
        .one(&*db)
        .await
        .unwrap();

    let customers_id = meter.clone().and_then(|m|m.customer_id).unwrap_or_else(||"Unknown".to_string());

    // invoice_id is the latest_invoice
    let latest_invoice = Invoices::find()
        .filter(invoices::Column::CustomerId.contains(customers_id.clone()))
        .order_by_desc(invoices::Column::CreatedAt)
        .one(&*db)
        .await
        .unwrap();

    let invoice_id = latest_invoice.clone().and_then(|inv|Some(inv.id)).unwrap_or("Unknown".to_string());
    let gen_id = generate_payment_id(&db).await.unwrap();
    let amount_formatted = Decimal::from_f64(amount).unwrap_or(Decimal::ZERO);

    let payment = payments::ActiveModel{
        id:Set(gen_id.to_owned()),
        invoice_id:Set(invoice_id.to_owned()),
        customer_id:Set(customers_id.to_owned()),
        amount:Set(amount_formatted.to_owned()),
        bal_amount:Set(None.to_owned()),
        created_at:Set(Utc::now().naive_utc().to_owned())
    };
    
    let _result = payment.insert(&*db).await.unwrap();
    let response = json!({
        "message":"Payment Completed".to_string()
    });

    Json(response)
}