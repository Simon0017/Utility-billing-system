use crate::entities::{customers, invoices, meters, payments};
use sea_orm::{DatabaseConnection, EntityTrait, QueryOrder};
use sea_orm::DbErr;

pub const RATE_PER_UNIT:i32 = 25;
pub const SERVICE_CHARGE:i32 = 10;

pub async fn gen_meter_no(db: &DatabaseConnection) -> Result<String, DbErr> {
    // retrieve the last meter no
    let last_meter = meters::Entity::find()
        .order_by_desc(meters::Column::Id)
        .one(db)
        .await?;

    let new_meter_no = if let Some(meter) = last_meter {
        let last_no = meter.id.trim_start_matches("HMI-");
        let next_no = last_no.parse::<u32>().unwrap_or(0) + 1;
        format!("HMI-{:03}",next_no)
    }else{
        "HMI-001".to_string()
    };

    Ok(new_meter_no)
}

pub async fn gen_customer_no(db: &DatabaseConnection) -> Result<String,DbErr>{
    // retrieve the last customer no
    let last_customer = customers::Entity::find()
        .order_by_desc(customers::Column::Id)
        .one(db)
        .await?;

    let new_customer_id = if let Some(customer) = last_customer {
        let last_id = customer.id.trim_start_matches("CUS-HMI-");
        let next_no = last_id.parse::<u32>().unwrap_or(0) +1;
        format!("CUS-HMI-{:03}",next_no)
    }else{
        "CUS-HMI-001".to_string()
    };

    Ok(new_customer_id)
}

// pub fn gen_password_automaticaly() ->Result<String,&str> {
//     format!("")
// }

pub async fn gen_invoice_no(db: &DatabaseConnection) -> Result<String, DbErr> {
    // retrieve the last meter no
    let last_invoice = invoices::Entity::find()
        .order_by_desc(invoices::Column::Id)
        .one(db)
        .await?;

    let new_invoice_no = if let Some(invoice) = last_invoice {
        let last_no = invoice.id.trim_start_matches("INV-");
        let next_no = last_no.parse::<u32>().unwrap_or(0) + 1;
        format!("INV-{:03}",next_no)
    }else{
        "INV-001".to_string()
    };

    Ok(new_invoice_no)
}

pub fn calculate_invoice_amount(reading_amount:&i32) -> std::io::Result<i32>{
    let subtotal = reading_amount * RATE_PER_UNIT;
    let total = subtotal + SERVICE_CHARGE;
    Ok(total)
}

pub async fn generate_payment_id(db: &DatabaseConnection) ->Result<String,DbErr> {
   let last_payment = payments::Entity::find()
        .order_by_desc(payments::Column::Id)
        .one(db)
        .await?;

    let new_payment_no = if let Some(payment) = last_payment {
        let last_no = payment.id.trim_start_matches("HMI-PMT-");
        let next_no = last_no.parse::<u32>().unwrap_or(0) + 1;
        format!("HMI-PMT-{:03}",next_no)
    }else{
        "HMI-PMT-001".to_string()
    };

    Ok(new_payment_no) 
}