use serde::{Deserialize, Serialize};

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
    ]
}

pub fn execute_tool(tool_call: &ToolCall) -> ToolResult {
    match tool_call.name.as_str() {
        "add" => {
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
        "subtract" => {
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
        "multiply" => {
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
        "divide" => {
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
        "power" => {
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
        "sqrt" => {
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
        _ => ToolResult {
            success: false,
            result: serde_json::json!(null),
            error: Some(format!("Unknown tool: {}", tool_call.name)),
        },
    }
}