use serde::{Deserialize, Serialize};
use crate::data_base::Database;

#[derive(Debug, Serialize, Deserialize)]
pub struct Tool {
    pub name: String,
    pub description: String,
    pub parameters: Vec<Parameter>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    pub param_type: String,
    pub description: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolCall {
    pub name: String,
    pub arguments: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ToolResult {
    pub success: bool,
    pub result: serde_json::Value,
    pub error: Option<String>,
}

pub fn get_available_tools() -> Vec<Tool> {
    vec![
        // Mathematical tools
        Tool {
            name: "add".to_string(),
            description: "Add two numbers together".to_string(),
            parameters: vec![
                Parameter {
                    name: "a".to_string(),
                    param_type: "number".to_string(),
                    description: "First number".to_string(),
                },
                Parameter {
                    name: "b".to_string(),
                    param_type: "number".to_string(),
                    description: "Second number".to_string(),
                },
            ],
        },
        Tool {
            name: "subtract".to_string(),
            description: "Subtract second number from first number".to_string(),
            parameters: vec![
                Parameter {
                    name: "a".to_string(),
                    param_type: "number".to_string(),
                    description: "First number".to_string(),
                },
                Parameter {
                    name: "b".to_string(),
                    param_type: "number".to_string(),
                    description: "Second number".to_string(),
                },
            ],
        },
        Tool {
            name: "multiply".to_string(),
            description: "Multiply two numbers together".to_string(),
            parameters: vec![
                Parameter {
                    name: "a".to_string(),
                    param_type: "number".to_string(),
                    description: "First number".to_string(),
                },
                Parameter {
                    name: "b".to_string(),
                    param_type: "number".to_string(),
                    description: "Second number".to_string(),
                },
            ],
        },
        Tool {
            name: "divide".to_string(),
            description: "Divide first number by second number".to_string(),
            parameters: vec![
                Parameter {
                    name: "a".to_string(),
                    param_type: "number".to_string(),
                    description: "Numerator".to_string(),
                },
                Parameter {
                    name: "b".to_string(),
                    param_type: "number".to_string(),
                    description: "Denominator".to_string(),
                },
            ],
        },
        Tool {
            name: "power".to_string(),
            description: "Raise first number to the power of second number".to_string(),
            parameters: vec![
                Parameter {
                    name: "base".to_string(),
                    param_type: "number".to_string(),
                    description: "Base number".to_string(),
                },
                Parameter {
                    name: "exponent".to_string(),
                    param_type: "number".to_string(),
                    description: "Exponent".to_string(),
                },
            ],
        },
        Tool {
            name: "sqrt".to_string(),
            description: "Calculate square root of a number".to_string(),
            parameters: vec![
                Parameter {
                    name: "value".to_string(),
                    param_type: "number".to_string(),
                    description: "Number to find square root of".to_string(),
                },
            ],
        },
        // Database tools
        Tool {
            name: "get_conversation_summary".to_string(),
            description: "Get a summary of messages in a conversation".to_string(),
            parameters: vec![
                Parameter {
                    name: "conversation_id".to_string(),
                    param_type: "number".to_string(),
                    description: "ID of the conversation to summarize".to_string(),
                },
                Parameter {
                    name: "message_limit".to_string(),
                    param_type: "number".to_string(),
                    description: "Number of recent messages to include (default: 50)".to_string(),
                },
            ],
        },
        Tool {
            name: "search_conversation".to_string(),
            description: "Search for messages in a conversation".to_string(),
            parameters: vec![
                Parameter {
                    name: "conversation_id".to_string(),
                    param_type: "number".to_string(),
                    description: "ID of the conversation to search".to_string(),
                },
                Parameter {
                    name: "search_term".to_string(),
                    param_type: "string".to_string(),
                    description: "Text to search for in messages".to_string(),
                },
            ],
        },
        Tool {
            name: "send_message".to_string(),
            description: "Send a message to a conversation as a specific user".to_string(),
            parameters: vec![
                Parameter {
                    name: "conversation_id".to_string(),
                    param_type: "number".to_string(),
                    description: "ID of the conversation".to_string(),
                },
                Parameter {
                    name: "username".to_string(),
                    param_type: "string".to_string(),
                    description: "Username of the sender".to_string(),
                },
                Parameter {
                    name: "content".to_string(),
                    param_type: "string".to_string(),
                    description: "Message content".to_string(),
                },
            ],
        },
        Tool {
            name: "get_user_conversations".to_string(),
            description: "Get all conversations for a specific user".to_string(),
            parameters: vec![
                Parameter {
                    name: "username".to_string(),
                    param_type: "string".to_string(),
                    description: "Username to get conversations for".to_string(),
                },
            ],
        },
        Tool {
            name: "get_conversation_stats".to_string(),
            description: "Get statistics about a conversation".to_string(),
            parameters: vec![
                Parameter {
                    name: "conversation_id".to_string(),
                    param_type: "number".to_string(),
                    description: "ID of the conversation".to_string(),
                },
            ],
        },
        Tool {
            name: "list_all_conversations".to_string(),
            description: "List all conversations in the database".to_string(),
            parameters: vec![],
        },
        Tool {
            name: "find_user".to_string(),
            description: "Find a user by username or email".to_string(),
            parameters: vec![
                Parameter {
                    name: "search_term".to_string(),
                    param_type: "string".to_string(),
                    description: "Username or email to search for".to_string(),
                },
            ],
        },
    ]
}

pub fn execute_tool(tool_call: &ToolCall) -> ToolResult {
    match tool_call.name.as_str() {
        // Mathematical tools
        "add" => execute_add(tool_call),
        "subtract" => execute_subtract(tool_call),
        "multiply" => execute_multiply(tool_call),
        "divide" => execute_divide(tool_call),
        "power" => execute_power(tool_call),
        "sqrt" => execute_sqrt(tool_call),

        // Database tools
        "get_conversation_summary" => execute_get_conversation_summary(tool_call),
        "search_conversation" => execute_search_conversation(tool_call),
        "send_message" => execute_send_message(tool_call),
        "get_user_conversations" => execute_get_user_conversations(tool_call),
        "get_conversation_stats" => execute_get_conversation_stats(tool_call),
        "list_all_conversations" => execute_list_all_conversations(tool_call),
        "find_user" => execute_find_user(tool_call),

        _ => ToolResult {
            success: false,
            result: serde_json::json!(null),
            error: Some(format!("Unknown tool: {}", tool_call.name)),
        },
    }
}

// ===== MATHEMATICAL TOOL IMPLEMENTATIONS =====

fn execute_add(tool_call: &ToolCall) -> ToolResult {
    let a = tool_call.arguments["a"].as_f64();
    let b = tool_call.arguments["b"].as_f64();

    match (a, b) {
        (Some(a), Some(b)) => ToolResult {
            success: true,
            result: serde_json::json!(a + b),
            error: None,
        },
        _ => ToolResult {
            success: false,
            result: serde_json::json!(null),
            error: Some("Invalid arguments for add".to_string()),
        },
    }
}

fn execute_subtract(tool_call: &ToolCall) -> ToolResult {
    let a = tool_call.arguments["a"].as_f64();
    let b = tool_call.arguments["b"].as_f64();

    match (a, b) {
        (Some(a), Some(b)) => ToolResult {
            success: true,
            result: serde_json::json!(a - b),
            error: None,
        },
        _ => ToolResult {
            success: false,
            result: serde_json::json!(null),
            error: Some("Invalid arguments for subtract".to_string()),
        },
    }
}

fn execute_multiply(tool_call: &ToolCall) -> ToolResult {
    let a = tool_call.arguments["a"].as_f64();
    let b = tool_call.arguments["b"].as_f64();

    match (a, b) {
        (Some(a), Some(b)) => ToolResult {
            success: true,
            result: serde_json::json!(a * b),
            error: None,
        },
        _ => ToolResult {
            success: false,
            result: serde_json::json!(null),
            error: Some("Invalid arguments for multiply".to_string()),
        },
    }
}

fn execute_divide(tool_call: &ToolCall) -> ToolResult {
    let a = tool_call.arguments["a"].as_f64();
    let b = tool_call.arguments["b"].as_f64();

    match (a, b) {
        (Some(a), Some(b)) => {
            if b == 0.0 {
                ToolResult {
                    success: false,
                    result: serde_json::json!(null),
                    error: Some("Division by zero".to_string()),
                }
            } else {
                ToolResult {
                    success: true,
                    result: serde_json::json!(a / b),
                    error: None,
                }
            }
        }
        _ => ToolResult {
            success: false,
            result: serde_json::json!(null),
            error: Some("Invalid arguments for divide".to_string()),
        },
    }
}

fn execute_power(tool_call: &ToolCall) -> ToolResult {
    let base = tool_call.arguments["base"].as_f64();
    let exponent = tool_call.arguments["exponent"].as_f64();

    match (base, exponent) {
        (Some(base), Some(exponent)) => ToolResult {
            success: true,
            result: serde_json::json!(base.powf(exponent)),
            error: None,
        },
        _ => ToolResult {
            success: false,
            result: serde_json::json!(null),
            error: Some("Invalid arguments for power".to_string()),
        },
    }
}

fn execute_sqrt(tool_call: &ToolCall) -> ToolResult {
    let value = tool_call.arguments["value"].as_f64();

    match value {
        Some(value) => {
            if value < 0.0 {
                ToolResult {
                    success: false,
                    result: serde_json::json!(null),
                    error: Some("Cannot calculate square root of negative number".to_string()),
                }
            } else {
                ToolResult {
                    success: true,
                    result: serde_json::json!(value.sqrt()),
                    error: None,
                }
            }
        }
        _ => ToolResult {
            success: false,
            result: serde_json::json!(null),
            error: Some("Invalid argument for sqrt".to_string()),
        },
    }
}

// ===== DATABASE TOOL IMPLEMENTATIONS =====

fn execute_get_conversation_summary(tool_call: &ToolCall) -> ToolResult {
    let conversation_id = tool_call.arguments["conversation_id"].as_i64().unwrap_or(0) as i32;
    let message_limit = tool_call.arguments["message_limit"].as_i64().unwrap_or(50) as i32;

    match Database::new() {
        Ok(db) => match db.get_conversation_summary(conversation_id, message_limit) {
            Ok(summary) => ToolResult {
                success: true,
                result: serde_json::json!(summary),
                error: None,
            },
            Err(e) => ToolResult {
                success: false,
                result: serde_json::json!(null),
                error: Some(format!("Database error: {}", e)),
            },
        },
        Err(e) => ToolResult {
            success: false,
            result: serde_json::json!(null),
            error: Some(format!("Failed to connect to database: {}", e)),
        },
    }
}

fn execute_search_conversation(tool_call: &ToolCall) -> ToolResult {
    let conversation_id = tool_call.arguments["conversation_id"].as_i64().unwrap_or(0) as i32;
    let search_term = tool_call.arguments["search_term"].as_str().unwrap_or("");

    match Database::new() {
        Ok(db) => match db.search_messages(conversation_id, search_term) {
            Ok(messages) => ToolResult {
                success: true,
                result: serde_json::json!({
                    "found": messages.len(),
                    "messages": messages,
                }),
                error: None,
            },
            Err(e) => ToolResult {
                success: false,
                result: serde_json::json!(null),
                error: Some(format!("Search error: {}", e)),
            },
        },
        Err(e) => ToolResult {
            success: false,
            result: serde_json::json!(null),
            error: Some(format!("Failed to connect to database: {}", e)),
        },
    }
}

fn execute_send_message(tool_call: &ToolCall) -> ToolResult {
    let conversation_id = tool_call.arguments["conversation_id"].as_i64().unwrap_or(0) as i32;
    let username = tool_call.arguments["username"].as_str().unwrap_or("");
    let content = tool_call.arguments["content"].as_str().unwrap_or("");

    if username.is_empty() || content.is_empty() {
        return ToolResult {
            success: false,
            result: serde_json::json!(null),
            error: Some("Username and content are required".to_string()),
        };
    }

    match Database::new() {
        Ok(db) => {
            match db.find_user_by_username(username) {
                Ok(Some(user)) => {
                    match db.insert_message(conversation_id, user.id, content, None) {
                        Ok(message_id) => ToolResult {
                            success: true,
                            result: serde_json::json!({
                                "message_id": message_id,
                                "conversation_id": conversation_id,
                                "user": username,
                                "content": content,
                            }),
                            error: None,
                        },
                        Err(e) => ToolResult {
                            success: false,
                            result: serde_json::json!(null),
                            error: Some(format!("Failed to send message: {}", e)),
                        },
                    }
                },
                Ok(None) => ToolResult {
                    success: false,
                    result: serde_json::json!(null),
                    error: Some(format!("User '{}' not found", username)),
                },
                Err(e) => ToolResult {
                    success: false,
                    result: serde_json::json!(null),
                    error: Some(format!("Database error: {}", e)),
                },
            }
        },
        Err(e) => ToolResult {
            success: false,
            result: serde_json::json!(null),
            error: Some(format!("Failed to connect to database: {}", e)),
        },
    }
}

fn execute_get_user_conversations(tool_call: &ToolCall) -> ToolResult {
    let username = tool_call.arguments["username"].as_str().unwrap_or("");

    if username.is_empty() {
        return ToolResult {
            success: false,
            result: serde_json::json!(null),
            error: Some("Username is required".to_string()),
        };
    }

    match Database::new() {
        Ok(db) => {
            match db.find_user_by_username(username) {
                Ok(Some(user)) => {
                    match db.find_conversations_by_user(user.id) {
                        Ok(conversations) => ToolResult {
                            success: true,
                            result: serde_json::json!({
                                "user": username,
                                "conversations": conversations,
                            }),
                            error: None,
                        },
                        Err(e) => ToolResult {
                            success: false,
                            result: serde_json::json!(null),
                            error: Some(format!("Failed to get conversations: {}", e)),
                        },
                    }
                },
                Ok(None) => ToolResult {
                    success: false,
                    result: serde_json::json!(null),
                    error: Some(format!("User '{}' not found", username)),
                },
                Err(e) => ToolResult {
                    success: false,
                    result: serde_json::json!(null),
                    error: Some(format!("Database error: {}", e)),
                },
            }
        },
        Err(e) => ToolResult {
            success: false,
            result: serde_json::json!(null),
            error: Some(format!("Failed to connect to database: {}", e)),
        },
    }
}

fn execute_get_conversation_stats(tool_call: &ToolCall) -> ToolResult {
    let conversation_id = tool_call.arguments["conversation_id"].as_i64().unwrap_or(0) as i32;

    match Database::new() {
        Ok(db) => match db.get_conversation_statistics(conversation_id) {
            Ok(stats) => ToolResult {
                success: true,
                result: stats,
                error: None,
            },
            Err(e) => ToolResult {
                success: false,
                result: serde_json::json!(null),
                error: Some(format!("Failed to get statistics: {}", e)),
            },
        },
        Err(e) => ToolResult {
            success: false,
            result: serde_json::json!(null),
            error: Some(format!("Failed to connect to database: {}", e)),
        },
    }
}

fn execute_list_all_conversations(tool_call: &ToolCall) -> ToolResult {
    match Database::new() {
        Ok(db) => match db.get_all_conversations() {
            Ok(conversations) => ToolResult {
                success: true,
                result: serde_json::json!({
                    "total": conversations.len(),
                    "conversations": conversations,
                }),
                error: None,
            },
            Err(e) => ToolResult {
                success: false,
                result: serde_json::json!(null),
                error: Some(format!("Failed to list conversations: {}", e)),
            },
        },
        Err(e) => ToolResult {
            success: false,
            result: serde_json::json!(null),
            error: Some(format!("Failed to connect to database: {}", e)),
        },
    }
}

fn execute_find_user(tool_call: &ToolCall) -> ToolResult {
    let search_term = tool_call.arguments["search_term"].as_str().unwrap_or("");

    if search_term.is_empty() {
        return ToolResult {
            success: false,
            result: serde_json::json!(null),
            error: Some("Search term is required".to_string()),
        };
    }

    match Database::new() {
        Ok(db) => match db.search_users(search_term, None) {
            Ok(users) => ToolResult {
                success: true,
                result: serde_json::json!({
                    "found": users.len(),
                    "users": users,
                }),
                error: None,
            },
            Err(e) => ToolResult {
                success: false,
                result: serde_json::json!(null),
                error: Some(format!("Search error: {}", e)),
            },
        },
        Err(e) => ToolResult {
            success: false,
            result: serde_json::json!(null),
            error: Some(format!("Failed to connect to database: {}", e)),
        },
    }
}