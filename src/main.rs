use actix_multipart::Multipart;
use actix_web::{post, App, HttpResponse, HttpServer, Responder};
use chrono::NaiveTime;
use futures_util::{StreamExt, TryStreamExt};
use serde::Serialize;
use std::collections::HashMap;

#[derive(Debug)]
enum LogStatus {
    Start,
    End,
}

#[derive(Debug)]
struct LogEntry {
    timestamp: NaiveTime,
    pid: String,
    status: LogStatus,
    description: String,
}

#[derive(Serialize)]
struct JobResult {
    pid: String,
    description: String,
    start: NaiveTime,
    end: NaiveTime,
    duration_secs: i64,
    level: String,
}

const WARNING_THRESHOLD_SECS: i64 = 60;
const ERROR_THRESHOLD_SECS: i64 = 120;

#[post("/upload")]
async fn upload_file(mut payload: Multipart) -> impl Responder {
    let mut start_events: HashMap<String, LogEntry> = HashMap::new();
    let mut results: Vec<JobResult> = Vec::new();
    let mut buffer = String::new();

    while let Ok(Some(mut field)) = payload.try_next().await {
        while let Some(chunk) = field.next().await {
            let data = match chunk {
                Ok(bytes) => bytes,
                Err(_) => return HttpResponse::BadRequest().body("Invalid chunk"),
            };

            let chunk_str = match std::str::from_utf8(&data) {
                Ok(s) => s,
                Err(_) => return HttpResponse::BadRequest().body("Invalid UTF-8"),
            };

            buffer.push_str(chunk_str);

            // Process complete lines
            let full_chunk = std::mem::take(&mut buffer);
            let mut lines = full_chunk.lines().map(str::to_string).collect::<Vec<String>>();

            // If the last char is not newline, it's an incomplete line
            if !full_chunk.ends_with('\n') {
                if let Some(incomplete) = lines.pop() {
                    buffer = incomplete;
                }
            }

            for line in lines {
                if let Ok(entry) = parse_line(&*line) {
                    match entry.status {
                        LogStatus::Start => {
                            start_events.insert(entry.pid.clone(), entry);
                        }
                        LogStatus::End => {
                            if let Some(start) = start_events.remove(&entry.pid) {
                                let duration = entry.timestamp - start.timestamp;
                                let secs = duration.num_seconds().abs(); // More precise

                                let level = if secs > ERROR_THRESHOLD_SECS {
                                    "ERROR"
                                } else if secs > WARNING_THRESHOLD_SECS {
                                    "WARNING"
                                } else {
                                    "INFO"
                                };

                                results.push(JobResult {
                                    pid: entry.pid,
                                    description: entry.description,
                                    start: start.timestamp,
                                    end: entry.timestamp,
                                    duration_secs: secs,
                                    level: level.to_string(),
                                });
                            }
                        }
                    }
                }
            }
        }
    }

    HttpResponse::Ok().json(results)
}

fn parse_line(line: &str) -> Result<LogEntry, ()> {
    let parts: Vec<&str> = line.splitn(4, ',').collect();
    if parts.len() != 4 {
        return Err(());
    }

    let timestamp = NaiveTime::parse_from_str(parts[0].trim(), "%H:%M:%S").map_err(|_| ())?;
    let pid = parts[1].trim().to_string();
    let status = match parts[2].trim() {
        "START" => LogStatus::Start,
        "END" => LogStatus::End,
        _ => return Err(()),
    };
    let description = parts[3].trim().to_string();

    Ok(LogEntry {
        timestamp,
        pid,
        status,
        description,
    })
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| App::new().service(upload_file))
        .bind(("127.0.0.1", 8080))?
        .run()
        .await
}
