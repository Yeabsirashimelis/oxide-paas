use std::{
    io::{BufRead, BufReader},
    process::{Command, Stdio},
};

use actix_web::{App, HttpResponse, HttpServer, Responder, web};
use reqwest::Client;
use shared::{Application, NewAppLog};
use uuid::Uuid;

async fn run_program(app: web::Json<Application>) -> impl Responder {
    println!("Starting application: {}", app.name);
    println!("Working directory: {}", app.working_dir);
    println!("Command: {}", app.command);

    let parts: Vec<&str> = app.command.split_whitespace().collect();

    if parts.is_empty() {
        eprintln!("Empty command provided");
        return HttpResponse::BadRequest().body("Empty command");
    }

    let program = parts[0].to_string();
    let args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();
    let working_dir = app.working_dir.clone();
    let app_id = app.id.unwrap();

    #[cfg(target_os = "windows")]
    let program = if program == "npm" {
        "npm.cmd".to_string()
    } else {
        program
    };

    let child = Command::new(&program)
        .args(&args)
        .current_dir(&working_dir)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn();

    match child {
        Ok(mut process) => {
            let pid = process.id();
            println!("Application started with PID: {:?}", pid);

            let stdout = process.stdout.take();
            let stderr = process.stderr.take();

            // Spawn a task to read stdout and send to paasd
            if let Some(stdout) = stdout {
                let app_id_clone = app_id;
                tokio::task::spawn_blocking(move || {
                    let reader = BufReader::new(stdout);
                    let rt = tokio::runtime::Handle::current();
                    for line in reader.lines() {
                        match line {
                            Ok(line) => {
                                println!("[stdout] {}", line);
                                let log = NewAppLog {
                                    app_id: app_id_clone,
                                    stream: "stdout".to_string(),
                                    message: line,
                                };
                                rt.block_on(send_log(log));
                            }
                            Err(e) => eprintln!("Error reading stdout: {}", e),
                        }
                    }
                });
            }

            // Spawn a task to read stderr and send to paasd
            if let Some(stderr) = stderr {
                let app_id_clone = app_id;
                tokio::task::spawn_blocking(move || {
                    let reader = BufReader::new(stderr);
                    let rt = tokio::runtime::Handle::current();
                    for line in reader.lines() {
                        match line {
                            Ok(line) => {
                                eprintln!("[stderr] {}", line);
                                let log = NewAppLog {
                                    app_id: app_id_clone,
                                    stream: "stderr".to_string(),
                                    message: line,
                                };
                                rt.block_on(send_log(log));
                            }
                            Err(e) => eprintln!("Error reading stderr: {}", e),
                        }
                    }
                });
            }

            HttpResponse::Ok().body(format!("Started with PID: {}", pid))
        }
        Err(e) => {
            eprintln!("Failed to execute process: {}", e);
            HttpResponse::InternalServerError().body(format!("Failed to start: {}", e))
        }
    }
}

async fn send_log(log: NewAppLog) {
    let client = Client::new();
    let url = format!("http://127.0.0.1:8080/apps/{}/logs", log.app_id);
    if let Err(e) = client.post(&url).json(&log).send().await {
        eprintln!("Failed to send log to paasd: {}", e);
    }
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let addr = ("127.0.0.1", 8001);
    println!("app is bound to http://{}:{}", addr.0, addr.1);
    HttpServer::new(move || App::new().route("/run", web::post().to(run_program)))
        .bind(addr)?
        .run()
        .await
}
