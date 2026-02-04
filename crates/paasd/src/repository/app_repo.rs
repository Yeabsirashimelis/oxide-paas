use crate::models::Application;
use sqlx::{Error, PgPool};

pub async fn insert_application(pool: &PgPool, app: &Application) -> Result<(), Error> {
    let query = "INSERT INTO App (name, command, status, port) VALUES ($1, $2, $3, $4)";
    sqlx::query(query)
        .bind(&app.name)
        .bind(&app.command)
        .bind(&app.status)
        .bind(&app.port)
        .execute(pool)
        .await?;

    Ok(())
}

pub async fn get_applications(pool: &PgPool) -> Result<Vec<Application>, Error> {
    let apps = sqlx::query_as(r#"SELECT id, name, command, status, port FROM App"#)
        .fetch_all(pool)
        .await?;
    Ok(apps)
}

pub async fn get_application(pool: &PgPool, app_id: i32) -> Result<Application, Error> {
    let app = sqlx::query_as(r#"SELECT id, name, command, status, port FROM App where id = $1"#)
        .bind(app_id)
        .fetch_one(pool)
        .await?;

    Ok(app)
}
