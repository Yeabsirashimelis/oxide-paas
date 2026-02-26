use std::{
    fs::{self},
    io::Write,
    path::Path,
};

use reqwest::Client;
use serde::Deserialize;
use shared::Application;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct PaasConfig {
    pub name: String,
    pub runtime: String,
    pub command: String,
    pub port: Option<i32>,
    pub id: Option<Uuid>,
    pub env: Option<std::collections::HashMap<String, String>>,
}

pub async fn deploy_project() -> anyhow::Result<()> {
    let filename = "paas.toml";
    if !Path::new(filename).exists() {
        println!("Initialize the project first. use 'paas init' for that.");
        return Ok(());
    }

    // let _file = File::open(filename)?;

    //read the wholefile into string
    let content = std::fs::read_to_string(filename)?;

    //map/ deserialize it directly into out struct
    let app_data: PaasConfig = match toml::from_str(&content) {
        Result::Ok(data) => data,
        Result::Err(e) => {
            eprintln!("Failed to parse paas.toml: {}", e);
            eprintln!("Check for duplicate keys in your [env] section.");
            return Ok(());
        }
    };

    if let Some(existing_id) = app_data.id {
        println!("Project already deployed (id: {}).", existing_id);
        println!("  - To restart it, use `paas redeploy`.");
        println!("  - To stop it, use `paas stop`.");
        println!("  - To deploy as a brand new app, remove the `id` line from paas.toml.");
        return Ok(());
    }

    println!("Deploying: {} using {}", app_data.name, app_data.command);

    let current_dir = std::env::current_dir()?
        .to_string_lossy()
        .to_string();

    let request_payload = Application {
        name: app_data.name,
        command: app_data.command,
        port: app_data.port.unwrap_or(3000),
        status: shared::AppStatus::PENDING,
        id: None,
        working_dir: current_dir,
        pid: None,
        env_vars: app_data.env.map(|e| serde_json::to_value(e).unwrap_or(serde_json::json!({}))),
    };

    let client = Client::new();
    let url = "http://127.0.0.1:8080/apps";

    let res = client.post(url).json(&request_payload).send().await?;

    if res.status().is_success() {
        let body: serde_json::Value = res.json().await?;
        let application_id: Uuid = body["id"].as_str().unwrap_or_default().parse()?;

        println!("Project Successfully deployed");
        println!("Starting application...");
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;

        let client2 = Client::new();
        let status_url = format!("http://127.0.0.1:8080/apps/{}/status", application_id);
        if let Ok(status_res) = client2.get(&status_url).send().await {
            if let Ok(status_body) = status_res.json::<serde_json::Value>().await {
                let status = status_body["status"].as_str().unwrap_or("UNKNOWN");
                if status == "STOPPED" || status == "CRASHED" {
                    eprintln!("Application failed to start! Check logs with `paas logs`");
                    eprintln!("Note: App ID not saved to paas.toml since it failed to start.");
                } else {
                    // Only write id to paas.toml if app actually started
                    // Insert id BEFORE the [env] section to avoid it being parsed as an env var
                    let content = fs::read_to_string("paas.toml")?;
                    let new_content = if content.contains("[env]") {
                        content.replace("[env]", &format!("id = \"{}\"\n\n[env]", application_id))
                    } else {
                        format!("{}\nid = \"{}\"\n", content, application_id)
                    };
                    fs::write("paas.toml", new_content)?;

                    let port = status_body["port"].as_i64().unwrap_or(0);
                    if port > 0 {
                        println!("Application is running on port {}", port);
                        println!("Local: http://localhost:{}", port);
                    } else {
                        println!("Application is running. Port not yet detected.");
                        println!("Check `paas logs` for the actual port.");
                    }
                }
            }
        }
    } else if res.status() == reqwest::StatusCode::CONFLICT {
        let body = res.text().await.unwrap_or_default();
        eprintln!("Deployment failed: {}", body);
        eprintln!("Tip: Change the port in paas.toml and try again.");
    } else {
        eprintln!("Deployment failed with status: {}", res.status());
    }

    Ok(())
}
