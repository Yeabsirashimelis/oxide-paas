use crate::repository::log_repo::{get_logs, insert_log};
use actix_web::{HttpResponse, Responder, web};
use shared::NewAppLog;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn post_log(
    pool: web::Data<PgPool>,
    log: web::Json<NewAppLog>,
) -> impl Responder {
    match insert_log(pool.get_ref(), &log).await {
        Ok(_) => HttpResponse::Ok().finish(),
        Err(e) => {
            eprintln!("DB Error inserting log: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

pub async fn get_app_logs(
    pool: web::Data<PgPool>,
    path: web::Path<Uuid>,
    query: web::Query<LogQuery>,
) -> impl Responder {
    let app_id = path.into_inner();
    let limit = query.limit.unwrap_or(100);

    match get_logs(pool.get_ref(), app_id, limit).await {
        Ok(logs) => HttpResponse::Ok().json(logs),
        Err(e) => {
            eprintln!("DB Error fetching logs: {}", e);
            HttpResponse::InternalServerError().finish()
        }
    }
}

#[derive(serde::Deserialize)]
pub struct LogQuery {
    pub limit: Option<i64>,
}
