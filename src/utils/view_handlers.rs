use axum::response::Html;
use tera::{Context,Tera};
use crate::TEMPLATES;

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