use std::env;

use actix_web::{App, HttpResponse, HttpServer, Responder, http::header::ContentType, web};
use serde::Deserialize;
use sqlx::PgPool;

//db connection
async fn connect_db() -> Result<sqlx::PgPool, sqlx::Error> {
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = PgPool::connect(&database_url).await?;

    Ok(pool)
}

#[derive(serde::Deserialize, sqlx::Type, Debug)]
#[sqlx(type_name = "app_status", rename_all = "UPPERCASE")]
enum AppStatus {
    PENDING,
    RUNNING,
    STOPPED,
    FAILED,
}

#[derive(Deserialize, Debug)]
struct Application {
    name: String,
    command: String,
    status: AppStatus,
    port: i32,
}

async fn post_program(pool: web::Data<PgPool>, app: web::Json<Application>) -> impl Responder {
    println!("{:?}", app);
    let query = "INSERT INTO App (name, command, status, port) VALUES ($1, $2, $3, $4)";
    sqlx::query(query)
        .bind(&app.name)
        .bind(&app.command)
        .bind(&app.status)
        .bind(&app.port)
        .execute(pool.get_ref())
        .await
        .unwrap();

    HttpResponse::Ok()
        .content_type(ContentType::plaintext())
        .body("application registered successfully")
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    //load the environmnet variables at the start of the server
    dotenvy::dotenv().ok();

    let addr = ("127.0.0.1", 8080);
    let pool = connect_db().await.expect("DB connection failed");
    println!("app is bound to http://{}:{}", addr.0, addr.1);
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .route("/", web::post().to(post_program))
    })
    .bind(addr)?
    .run()
    .await
}
