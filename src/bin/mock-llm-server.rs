use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[derive(Deserialize)]
struct LlmRequest {
    prompt: String,
}

#[derive(Serialize)]
struct LlmResponse {
    output: String,
}

#[derive(Clone)]
struct AppState {
    attempt_count: Arc<std::sync::atomic::AtomicUsize>,
    fail_attempts: usize,
}

async fn handle_connection(mut stream: tokio::net::TcpStream, state: AppState) {
    let mut buffer = [0u8; 65536];
    
    match stream.read(&mut buffer).await {
        Ok(n) if n > 0 => {
            let request = String::from_utf8_lossy(&buffer[..n]);
            
            // Debug: print first line of request
            let first_line = request.lines().next().unwrap_or("");
            println!("Received request: {}", first_line);
            
            // Simple HTTP/1.1 request parsing - be more lenient
            if !request.contains("POST") || !request.contains("/llm") {
                println!("Not a POST to /llm, returning 404");
                let response = b"HTTP/1.1 404 Not Found\r\nConnection: close\r\n\r\n";
                let _ = stream.write_all(response).await;
                let _ = stream.shutdown().await;
                return;
            }
            
            // Find the JSON body (after \r\n\r\n)
            let body_start = request.find("\r\n\r\n").map(|i| i + 4).unwrap_or(0);
            let body_str = &request[body_start..];
            
            // Find Content-Length header to get exact body length
            let mut content_length = 0;
            for line in request.lines() {
                if line.to_lowercase().starts_with("content-length:") {
                    if let Some(len_str) = line.split(':').nth(1) {
                        content_length = len_str.trim().parse().unwrap_or(0);
                        break;
                    }
                }
            }
            
            // Extract body - use Content-Length if available, otherwise use everything after headers
            let body = if content_length > 0 && body_str.len() >= content_length {
                &body_str[..content_length]
            } else {
                body_str.trim_end_matches('\0').trim()
            };
            
            println!("Body received ({} bytes): {}", body.len(), &body[..body.len().min(100)]);
            
            // Parse JSON
            let req: LlmRequest = match serde_json::from_str(body) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Failed to parse JSON: {}. Body: {}", e, &body[..body.len().min(200)]);
                    let error_response = format!(
                        "HTTP/1.1 400 Bad Request\r\nContent-Type: application/json\r\nConnection: close\r\n\r\n{{\"error\":\"{}\"}}",
                        e
                    );
                    let _ = stream.write_all(error_response.as_bytes()).await;
                    let _ = stream.shutdown().await;
                    return;
                }
            };
            
            let attempt = state
                .attempt_count
                .fetch_add(1, std::sync::atomic::Ordering::SeqCst)
                + 1;

            println!("Mock LLM: Received request (attempt {})", attempt);
            println!("Prompt preview: {}...", &req.prompt[..req.prompt.len().min(200)]);

            // Fail first N attempts to test retry logic
            let output = if attempt <= state.fail_attempts {
                println!("Mock LLM: Returning invalid JSON (testing retry logic)");
                r#"{"invalid": "json", missing_fields: true}"#
            } else {
                println!("Mock LLM: Returning valid JSON");
                r#"{
        "name": "Task Management & Collaboration System",
        "rationale": "A comprehensive system for managing tasks and projects with real-time collaboration. Uses PostgreSQL for reliable data persistence, WebSockets for instant updates, and a RESTful API for clean integration. The responsive frontend ensures mobile compatibility, while authentication secures all operations.",
        "components": [
            {
                "id": "task-service",
                "responsibility": "Core task CRUD operations, task assignment, and status management",
                "api": "POST /api/tasks - Create task\nGET /api/tasks - List tasks\nGET /api/tasks/:id - Get task\nPUT /api/tasks/:id - Update task\nDELETE /api/tasks/:id - Delete task"
            },
            {
                "id": "project-service",
                "responsibility": "Project management, project membership, and project-level settings",
                "api": "POST /api/projects - Create project\nGET /api/projects - List projects\nGET /api/projects/:id - Get project\nPUT /api/projects/:id - Update project"
            },
            {
                "id": "websocket-service",
                "responsibility": "Real-time updates for task changes, project updates, and collaboration events",
                "api": "WS /ws - WebSocket connection\nMessages: {type: 'task_updated', data: {...}}\n{type: 'project_updated', data: {...}}"
            },
            {
                "id": "auth-service",
                "responsibility": "User authentication, authorization, and session management",
                "api": "POST /api/auth/login - Login\nPOST /api/auth/register - Register\nPOST /api/auth/logout - Logout\nGET /api/auth/me - Get current user"
            },
            {
                "id": "postgres-db",
                "responsibility": "Data persistence for tasks, projects, users, and relationships",
                "api": "Database schema:\n- users(id, email, password_hash, name)\n- projects(id, name, owner_id, created_at)\n- tasks(id, project_id, title, description, status, assignee_id, created_at)\n- project_members(project_id, user_id, role)"
            }
        ],
        "risks": [
            "WebSocket connections need proper scaling strategy (consider Redis pub/sub for multi-server)",
            "PostgreSQL connection pooling required for high concurrency",
            "Real-time updates may overwhelm mobile clients - implement rate limiting",
            "Authentication tokens must have proper expiration and refresh mechanism",
            "Task assignment conflicts when multiple users assign simultaneously"
        ]
    }"#
            };
            
            // The output is already a JSON string, wrap it in LlmResponse
            // output field should contain the raw JSON string
            let response_obj = LlmResponse {
                output: output.to_string(),
            };
            let response_body = serde_json::to_string(&response_obj)
                .map_err(|e| {
                    eprintln!("Failed to serialize response: {}", e);
                    e
                })
                .unwrap();
            
            // Debug: print response
            println!("Sending response: {}", &response_body[..response_body.len().min(200)]);
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                response_body.len(),
                response_body
            );
            let _ = stream.write_all(response.as_bytes()).await;
        }
        _ => {}
    }
    
    let _ = stream.shutdown().await;
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let port = std::env::var("MOCK_LLM_PORT")
        .unwrap_or_else(|_| "8081".to_string()) // Use 8081 as default to avoid conflicts
        .parse::<u16>()
        .unwrap_or(8081);

    let fail_attempts = std::env::var("MOCK_LLM_FAIL_ATTEMPTS")
        .unwrap_or_else(|_| "1".to_string())
        .parse::<usize>()
        .unwrap_or(1);

    let state = AppState {
        attempt_count: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        fail_attempts,
    };

    let addr = format!("0.0.0.0:{}", port);
    let listener = TcpListener::bind(&addr).await?;

    println!("Mock LLM server listening on http://{}", addr);
    println!("Will fail first {} attempt(s) to test retry logic", fail_attempts);
    println!("Send POST requests to http://{}/llm", addr);
    println!("Using simple HTTP/1.1 server (no HTTP/2, no upgrades)");

    loop {
        match listener.accept().await {
            Ok((stream, _)) => {
                let state = state.clone();
                tokio::spawn(async move {
                    handle_connection(stream, state).await;
                });
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }
}
