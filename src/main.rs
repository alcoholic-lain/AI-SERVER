mod tools;
mod ws_server;

use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use std::sync::Arc;
use tokio::sync::Mutex;
use tools::{get_available_tools, execute_tool, ToolCall};
use regex::Regex;
use ws_server::{WebSocketServer, ClientMessage};

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize, Deserialize, Debug)]
struct StreamChunk {
    choices: Vec<StreamChoice>,
}

#[derive(Serialize, Deserialize, Debug)]
struct StreamChoice {
    delta: StreamDelta,
    finish_reason: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct StreamDelta {
    content: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
struct CustomToolRequest {
    name: String,
    arguments: serde_json::Map<String, serde_json::Value>,
}

/// Converts our tool definitions to a text format the model can understand
fn format_tools_for_prompt() -> String {
    let tools = get_available_tools();
    let mut tool_descriptions = String::from("\n\nYou have access to the following tools:\n\n");

    for tool in tools {
        tool_descriptions.push_str(&format!("Tool: {}\n", tool.name));
        tool_descriptions.push_str(&format!("Description: {}\n", tool.description));
        tool_descriptions.push_str("Parameters:\n");
        for param in tool.parameters {
            tool_descriptions.push_str(&format!("  - {} ({}): {}\n", param.name, param.param_type, param.description));
        }
        tool_descriptions.push_str("\n");
    }

    tool_descriptions.push_str("To use a tool, respond with: <tool_request>[{\"name\": \"tool_name\", \"arguments\": {\"param\": value}}]</tool_request>\n");
    tool_descriptions.push_str("You can call multiple tools by including multiple objects in the array.\n");
    tool_descriptions.push_str("After receiving tool results, provide your final answer to the user.\n");

    tool_descriptions
}

/// Parses custom tool requests from model output
fn parse_tool_requests(text: &str) -> Option<Vec<CustomToolRequest>> {
    let re = Regex::new(r"<tool_request>\s*(\[.*?\])\s*</tool_request>").ok()?;

    for cap in re.captures_iter(text) {
        if let Some(json_str) = cap.get(1) {
            if let Ok(requests) = serde_json::from_str::<Vec<serde_json::Value>>(json_str.as_str()) {
                let mut tool_requests = Vec::new();
                for req in requests {
                    if let (Some(name), Some(args)) = (req.get("name"), req.get("arguments")) {
                        if let (Some(name_str), Some(args_obj)) = (name.as_str(), args.as_object()) {
                            tool_requests.push(CustomToolRequest {
                                name: name_str.to_string(),
                                arguments: args_obj.clone(),
                            });
                        }
                    }
                }
                return Some(tool_requests);
            }
        }
    }
    None
}

async fn chat_completion_stream(
    client: &Client,
    api_base: &str,
    api_key: &str,
    model_name: &str,
    messages: &[Message],
    ws_server: Option<&WebSocketServer>,
) -> Result<String, Box<dyn std::error::Error>> {
    let response = client
        .post(format!("{}/chat/completions", api_base))
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&json!({
            "model": model_name,
            "messages": messages,
            "stream": true
        }))
        .send()
        .await
        .map_err(|e| format!("Failed to send request to LM Studio: {}", e))?;

    let mut stream = response.bytes_stream();
    let mut accumulated_content = String::new();
    let mut in_tool_request = false;

    use futures_util::stream::StreamExt;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(|e| format!("Error reading stream: {}", e))?;
        let chunk_str = String::from_utf8_lossy(&chunk);

        for line in chunk_str.split("\n\n") {
            if line.starts_with("data: ") {
                let json_str = line.trim_start_matches("data: ");
                if json_str == "[DONE]" {
                    break;
                }

                match serde_json::from_str::<StreamChunk>(json_str) {
                    Ok(chunk_data) => {
                        if let Some(choice) = chunk_data.choices.get(0) {
                            if let Some(content) = &choice.delta.content {
                                accumulated_content.push_str(content);

                                // Check for tool request markers
                                if content.contains("<tool_request>") {
                                    in_tool_request = true;
                                }

                                // Only send visible content (not tool requests)
                                if !in_tool_request {
                                    if let Some(ws) = ws_server {
                                        ws.broadcast_json(&json!({
                                            "type": "chunk",
                                            "content": content
                                        })).await;
                                    }
                                }

                                if content.contains("</tool_request>") {
                                    in_tool_request = false;
                                }
                            }
                        }
                    }
                    Err(_) => continue,
                }
            }
        }
    }

    Ok(accumulated_content)
}

async fn process_message(
    user_message: String,
    messages: Arc<Mutex<Vec<Message>>>,
    client: &Client,
    api_base: &str,
    api_key: &str,
    model_name: &str,
    ws_server: &WebSocketServer,
) -> Result<(), Box<dyn std::error::Error>> {
    // Add user message
    {
        let mut msgs = messages.lock().await;
        msgs.push(Message {
            role: "user".to_string(),
            content: user_message,
        });
    }

    // Send start of assistant message
    ws_server.broadcast_json(&json!({
        "type": "start",
        "role": "assistant"
    })).await;

    let max_iterations = 5;
    for iteration in 0..max_iterations {
        let msgs = messages.lock().await.clone();
        drop(msgs);

        let response = chat_completion_stream(
            client,
            api_base,
            api_key,
            model_name,
            &messages.lock().await,
            Some(ws_server),
        ).await?;

        // Check for tool requests
        if let Some(tool_requests) = parse_tool_requests(&response) {
            // Add assistant message
            {
                let mut msgs = messages.lock().await;
                msgs.push(Message {
                    role: "assistant".to_string(),
                    content: response.clone(),
                });
            }

            // Hide any streaming text and show tool execution UI
            ws_server.broadcast_json(&json!({
                "type": "tool_start",
                "tools": tool_requests.iter().map(|t| t.name.clone()).collect::<Vec<_>>()
            })).await;

            let mut tool_results = Vec::new();

            for tool_req in tool_requests {
                let tool_call = ToolCall {
                    name: tool_req.name.clone(),
                    arguments: serde_json::Value::Object(tool_req.arguments.clone()),
                };

                let result = execute_tool(&tool_call);

                let result_msg = if result.success {
                    format!("Tool '{}' returned: {}", tool_req.name, result.result)
                } else {
                    format!("Tool '{}' error: {}", tool_req.name, result.error.as_ref().unwrap())
                };

                tool_results.push(result_msg.clone());

                ws_server.broadcast_json(&json!({
                    "type": "tool_result",
                    "tool": tool_req.name,
                    "result": result.result,
                    "success": result.success
                })).await;
            }

            // Add tool results
            {
                let mut msgs = messages.lock().await;
                msgs.push(Message {
                    role: "tool".to_string(),
                    content: format!("<tool_results>\n{}\n</tool_results>", tool_results.join("\n")),
                });
            }

            if iteration < max_iterations - 1 {
                ws_server.broadcast_json(&json!({
                    "type": "start",
                    "role": "assistant"
                })).await;
            }
        } else {
            // No tool requests, save final response
            {
                let mut msgs = messages.lock().await;
                msgs.push(Message {
                    role: "assistant".to_string(),
                    content: response,
                });
            }
            break;
        }
    }

    // Send end of message
    ws_server.broadcast_json(&json!({
        "type": "end"
    })).await;

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_base = env::var("LM_STUDIO_API_BASE").unwrap_or_else(|_| "http://localhost:1234/v1".to_string());
    let api_key = env::var("LM_STUDIO_API_KEY").unwrap_or_else(|_| "not-needed".to_string());
    let model_name = env::var("LM_STUDIO_MODEL").unwrap_or_else(|_| "ibm/granite-3.1-8b".to_string());
    let ws_port: u16 = env::var("WS_PORT").unwrap_or_else(|_| "8080".to_string()).parse().unwrap_or(8080);

    let tools_description = format_tools_for_prompt();

    let messages = Arc::new(Mutex::new(vec![Message {
        role: "system".to_string(),
        content: format!("You are a helpful AI assistant with access to mathematical tools.{}\n\nWhen users ask mathematical questions that require calculations, you should use the appropriate tools. After using tools and receiving results, provide a clear answer to the user.", tools_description),
    }]));

    let client = Client::new();

    // Start WebSocket server
    let ws_server = WebSocketServer::new(ws_port).await?;
    println!("\x1b[1;32m✓ WebSocket server started on ws://localhost:{}\x1b[0m", ws_port);
    println!("\x1b[1;32m✓ Web UI available at http://localhost:{}\x1b[0m", ws_port);

    let messages_clone = messages.clone();
    let client_clone = client.clone();
    let api_base_clone = api_base.clone();
    let api_key_clone = api_key.clone();
    let model_name_clone = model_name.clone();
    let ws_server_clone = ws_server.clone();

    // Handle incoming WebSocket messages
    tokio::spawn(async move {
        loop {
            if let Some(msg) = ws_server_clone.receive_message().await {
                match msg {
                    ClientMessage::SendMessage { content } => {
                        let _ = process_message(
                            content,
                            messages_clone.clone(),
                            &client_clone,
                            &api_base_clone,
                            &api_key_clone,
                            &model_name_clone,
                            &ws_server_clone,
                        ).await;
                    }
                }
            }
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }
    });

    println!("\x1b[1;34mChat-IBM WebSocket Server Running\x1b[0m");
    println!("\x1b[1;34mPress Ctrl+C to stop\x1b[0m");

    // Keep running
    tokio::signal::ctrl_c().await?;
    println!("\n\x1b[1;34mShutting down...\x1b[0m");

    Ok(())
}