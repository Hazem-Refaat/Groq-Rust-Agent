# Groq Rust Agent

A template project demonstrating how to implement LLM function calling patterns using Rust. This project showcases a simple calculator implementation as an example of how to structure and handle function calls with Large Language Models.

## About This Template

This project serves as a reference implementation for:
- Setting up LLM function calling infrastructure in Rust
- Handling dynamic function registration and execution
- Managing async communication with LLM APIs
- Parsing and processing LLM function calls
- Structuring response handling and recursive interactions

The calculator functionality is included as a practical example but can be replaced with any other function implementation following the same patterns.

## Features

- Complete function calling implementation template
- Dynamic function registry system
- Async request/response handling
- Error handling patterns
- Example calculator implementation with:
  - Basic arithmetic operations (+, -, *, /)
  - Input validation
  - Error handling
  - Pretty console output

## Architecture

The project demonstrates several key architectural patterns:
- Function registry using `lazy_static` and `HashMap`
- Dynamic function dispatch
- Regex-based function call parsing
- Recursive response handling
- Type-safe parameter validation

## Prerequisites

- Rust (latest stable version)
- A Groq API key (can be adapted for other LLM providers)

## Setup

1. Clone the template:
```bash
git clone https://github.com/Hazem-Refaat/groq-rust-agent.git
cd groq-rust-agent
```

2. Configure your API key:
```bash
echo "GROQ_API_KEY='your_key_here'" > .env
```

3. Build the project:
```bash
cargo build
```

## Demo / Usage

After setup, you can run the project with:
```bash
cargo run
```

## Extending the Template

To add new functions:

1. Define your function handler:
```rust
fn handle_new_function(params: serde_json::Value) -> String {
    // Your implementation here
}
```

2. Register it in the `FUNCTION_REGISTRY`:
```rust
lazy_static! {
    static ref FUNCTION_REGISTRY: HashMap<&'static str, FunctionHandler> = {
        let mut m = HashMap::new();
        m.insert("calculate", handle_calculate as FunctionHandler);
        m.insert("new_function", handle_new_function as FunctionHandler);
        m
    };
}
```

3. Add the tool definition in `main()`:
```rust
Tool {
    tool_type: "function".to_string(),
    function: ToolFunction {
        name: "new_function".to_string(),
        description: "Description of your function".to_string(),
        parameters: // ... parameter definition
    }
}
```

## Project Structure

```
groq-rust-agent/
├── src/
│   └── main.rs         # Core implementation and examples
├── Cargo.toml          # Dependencies
├── .env               # Configuration
└── README.md          # Documentation
```

## Dependencies

- reqwest: Async HTTP client
- serde: JSON serialization
- tokio: Async runtime
- anyhow: Error handling
- regex: Function call parsing
- lazy_static: Static initialization
- dotenv: Configuration management

## Contributing

Feel free to use this template as a starting point for your own LLM function calling implementations.
