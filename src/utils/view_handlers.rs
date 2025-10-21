use axum::response::Html;
use tera::{Context};

use crate::TEMPLATES;

pub async fn root() ->&'static str {
    "Hello from Hephaestus Motor Inc"
}

pub async  fn company() ->Html<String> {
    let mut ctx = Context::new();
    ctx.insert("name", "Hephaestus Motor Inc");

    let rendered = TEMPLATES.render("pages/company_portal.html", &ctx).unwrap();
    Html(rendered)
}