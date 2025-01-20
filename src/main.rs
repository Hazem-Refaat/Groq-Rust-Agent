use anyhow::Result;
use dotenv::dotenv;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use std::io::{self, Write};
use regex::Regex;
use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    static ref FUNCTION_REGEX: Regex = Regex::new(r"<function=(\w+)(\{.*?\})>").unwrap();
    static ref FUNCTION_REGISTRY: HashMap<&'static str, FunctionHandler> = {
        let mut m = HashMap::new();
        m.insert("calculate", handle_calculate as FunctionHandler);
        m
    };
}

#[derive(Serialize, Deserialize, Debug, Clone)]
struct Message {
    role: String,
    #[serde(default)]
    content: String,
}

#[derive(Serialize, Debug, Clone)]
struct ToolFunction {
    name: String,
    description: String,
    parameters: ToolFunctionParameters,
}

#[derive(Serialize, Debug, Clone)]
struct ToolFunctionParameters {
    #[serde(rename = "type")]
    param_type: String,
    properties: serde_json::Value,
    required: Vec<String>,
}

#[derive(Serialize, Debug, Clone)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    tools: Vec<Tool>,
    tool_choice: String,
}

#[derive(Serialize, Debug, Clone)]
struct Tool {
    #[serde(rename = "type")]
    tool_type: String,
    function: ToolFunction,
}

#[derive(Deserialize, Debug, Clone)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize, Debug, Clone)]
struct Choice {
    message: Option<Message>,
    text: Option<String>,
}

// Define function type
type FunctionHandler = fn(serde_json::Value) -> String;

fn handle_calculate(params: serde_json::Value) -> String {
    println!("\n🔧 Function 'calculate' called with parameters: {}", params);
    if let (Some(a), Some(b), Some(op)) = (
        params.get("a").and_then(|v| v.as_f64()),
        params.get("b").and_then(|v| v.as_f64()),
        params.get("operation").and_then(|v| v.as_str()),
    ) {
        let result = match op {
            "+" => a + b,
            "-" => a - b,
            "*" => a * b,
            "/" if b != 0.0 => a / b,
            "/" => return format!("Error: Division by zero"),
            _ => return format!("Error: Unknown operation '{}'", op),
        };
        
        let output = format!("The result of {} {} {} is {}", a, op, b, result);
        println!("📤 Function output: {}", output);
        output
    } else {
        let error = "Invalid parameters for calculation".to_string();
        println!("❌ Function error: {}", error);
        error
    }
}

fn handle_chat_response<'a>(
    client: &'a Client,
    api_key: &'a str,
    chat_response: ChatResponse,
) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send + 'a>> {
    Box::pin(async move {
        for choice in chat_response.choices {
            if let Some(message) = choice.message {
                if let Some(captures) = FUNCTION_REGEX.captures(&message.content) {
                    if let (Some(function_name), Some(params_str)) = (captures.get(1), captures.get(2)) {
                        let function_name = function_name.as_str();
                        let params_str = params_str.as_str();
                        
                        println!("\n🤖 Model requested function: {}", function_name);
                        println!("📥 With parameters: {}", params_str);
                        
                        if let Ok(params) = serde_json::from_str(params_str) {
                            if let Some(handler) = FUNCTION_REGISTRY.get(function_name) {
                                let result = handler(params);
                                println!("✅ Function executed successfully");
                                
                                let new_message = Message {
                                    role: "user".to_string(),
                                    content: result,
                                };

                                let new_request_payload = ChatRequest {
                                    model: "llama-3.3-70b-versatile".to_string(),
                                    messages: vec![new_message],
                                    tools: vec![],
                                    tool_choice: "auto".to_string(),
                                };

                                let response = client
                                    .post("https://api.groq.com/openai/v1/chat/completions")
                                    .header("Content-Type", "application/json")
                                    .header("Authorization", format!("Bearer {}", api_key))
                                    .json(&new_request_payload)
                                    .send()
                                    .await?;

                                let new_chat_response: ChatResponse = response.json().await?;
                                handle_chat_response(client, api_key, new_chat_response).await?;
                            } else {
                                println!("❌ Function '{}' not found in registry", function_name);
                            }
                        } else {
                            println!("❌ Invalid parameter format: {}", params_str);
                        }
                    }
                } else {
                    println!("\n🤖 Chatbot: {}", message.content);
                }
            } else if let Some(text) = choice.text {
                println!("\n🤖 Chatbot: {}", text);
            }
        }
        Ok(())
    })
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    let api_key = env::var("GROQ_API_KEY").expect("GROQ_API_KEY not set");
    println!("Loaded API key");

    let client = Client::new();

    let tools = vec![Tool {
        tool_type: "function".to_string(),
        function: ToolFunction {
            name: "calculate".to_string(),
            description: "Calculator tool that performs basic arithmetic operations".to_string(),
            parameters: ToolFunctionParameters {
                param_type: "object".to_string(),
                properties: serde_json::json!({
                    "a": {
                        "type": "number",
                        "description": "First number",
                    },
                    "b": {
                        "type": "number",
                        "description": "Second number",
                    },
                    "operation": {
                        "type": "string",
                        "description": "Operation to perform (+, -, *, /)",
                        "enum": ["+", "-", "*", "/"]
                    }
                }),
                required: vec!["a".to_string(), "b".to_string(), "operation".to_string()],
            },
        },
    }];

    loop {
        print!("Enter your message: ");
        io::stdout().flush()?;
        let mut user_input = String::new();
        io::stdin().read_line(&mut user_input)?;
        let user_input = user_input.trim();

        if user_input.eq_ignore_ascii_case("exit") {
            println!("Exiting...");
            break;
        }

        let request_payload = ChatRequest {
            model: "llama-3.3-70b-versatile".to_string(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: "You are a helpful assistant with access to a calculator. When users want to perform arithmetic operations, use the calculate function by responding with: <function=calculate{\"a\": number1, \"b\": number2, \"operation\": \"op\"}> where op can be +, -, *, or /. After receiving results, provide a friendly response.".to_string(),
                },
                Message {
                    role: "user".to_string(),
                    content: user_input.to_string(),
                },
            ],
            tools: tools.clone(),
            tool_choice: "auto".to_string(),
        };

        let response = client
            .post("https://api.groq.com/openai/v1/chat/completions")
            .header("Content-Type", "application/json")
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&request_payload)
            .send()
            .await?;

        let chat_response: ChatResponse = response.json().await?;
        handle_chat_response(&client, &api_key, chat_response).await?;
    }

    Ok(())
}
