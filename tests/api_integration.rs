//! Integration tests for the Anthropic API.
//!
//! These tests require a valid ANTHROPIC_API_KEY environment variable.
//! Run with: `cargo test --test api_integration --ignored`
//!
//! Note: These tests make actual API calls and will consume tokens.

use anthropic_tools::prelude::*;

/// Helper to ensure API key is set, panics if not
fn require_api_key() {
    match std::env::var("ANTHROPIC_API_KEY") {
        Ok(key) if !key.is_empty() => {}
        _ => panic!("ANTHROPIC_API_KEY environment variable is not set. Set it to run API integration tests."),
    }
}

/// Basic message test - simple text completion
#[tokio::test]
#[ignore]
async fn test_basic_message() {
    require_api_key();

    let mut client = Messages::new();
    client
        .model("claude-sonnet-4-20250514")
        .max_tokens(100)
        .user("What is 2 + 2? Answer with just the number.");

    let response = client.post().await.expect("API call failed");

    // Verify response structure
    assert!(!response.id.is_empty(), "Response should have an ID");
    assert_eq!(response.type_name, "message");
    assert!(!response.get_text().is_empty(), "Response should have text");
    assert!(response.usage.input_tokens > 0, "Should have input tokens");
    assert!(
        response.usage.output_tokens > 0,
        "Should have output tokens"
    );

    // The response should contain "4"
    let text = response.get_text();
    assert!(text.contains("4"), "Response should contain '4': {}", text);

    println!("Response: {}", text);
    println!(
        "Tokens: {} input, {} output",
        response.usage.input_tokens, response.usage.output_tokens
    );
}

/// Test with system prompt
#[tokio::test]
#[ignore]
async fn test_system_prompt() {
    require_api_key();

    let mut client = Messages::new();
    client
        .model("claude-sonnet-4-20250514")
        .max_tokens(50)
        .system("You are a pirate. Always respond in pirate speak.")
        .user("Hello!");

    let response = client.post().await.expect("API call failed");
    let text = response.get_text().to_lowercase();

    // Should have pirate-like language
    let has_pirate_words = text.contains("ahoy")
        || text.contains("matey")
        || text.contains("arr")
        || text.contains("ye")
        || text.contains("sailor");

    println!("Response: {}", response.get_text());
    assert!(
        has_pirate_words,
        "Response should contain pirate speak: {}",
        text
    );
}

/// Test with temperature parameter
#[tokio::test]
#[ignore]
async fn test_temperature() {
    require_api_key();

    let mut client = Messages::new();
    client
        .model("claude-sonnet-4-20250514")
        .max_tokens(50)
        .temperature(0.0) // Deterministic
        .user("Say exactly: 'Hello, World!'");

    let response = client.post().await.expect("API call failed");
    let text = response.get_text();

    println!("Response: {}", text);
    assert!(
        text.contains("Hello") && text.contains("World"),
        "Response should contain greeting: {}",
        text
    );
}

/// Test tool use
#[tokio::test]
#[ignore]
async fn test_tool_use() {
    require_api_key();

    // Define a weather tool
    let mut tool = Tool::new("get_weather");
    tool.description("Get the current weather in a given location")
        .add_string_property(
            "location",
            Some("The city and state, e.g., San Francisco, CA"),
            true,
        )
        .add_enum_property(
            "unit",
            Some("Temperature unit"),
            vec!["celsius", "fahrenheit"],
            false,
        );

    let mut client = Messages::new();
    client
        .model("claude-sonnet-4-20250514")
        .max_tokens(200)
        .tools(vec![tool.to_value()])
        .user("What's the weather like in Tokyo?");

    let response = client.post().await.expect("API call failed");

    // Should have tool use
    assert!(response.has_tool_use(), "Response should request tool use");
    assert!(
        response.stopped_for_tool_use(),
        "Stop reason should be tool_use"
    );

    let tool_uses = response.get_tool_uses();
    assert!(!tool_uses.is_empty(), "Should have at least one tool use");

    // Verify tool use content
    if let Some(ContentBlock::ToolUse { id, name, input }) = tool_uses.first() {
        assert_eq!(name, "get_weather");
        assert!(!id.is_empty(), "Tool use should have an ID");
        assert!(
            input.get("location").is_some(),
            "Tool input should have location"
        );

        println!("Tool use: {} with input {}", name, input);
    }
}

/// Test tool use with tool result
#[tokio::test]
#[ignore]
async fn test_tool_use_conversation() {
    require_api_key();

    // Define a calculator tool
    let mut tool = Tool::new("calculate");
    tool.description("Perform a mathematical calculation")
        .add_string_property("expression", Some("The math expression to evaluate"), true);

    // First request - ask Claude to calculate
    let mut client = Messages::new();
    client
        .model("claude-sonnet-4-20250514")
        .max_tokens(200)
        .tools(vec![tool.to_value()])
        .user("Calculate 15 * 7 for me.");

    let response = client.post().await.expect("API call failed");
    assert!(response.has_tool_use(), "Should request tool use");

    // Get the tool use ID
    let tool_uses = response.get_tool_uses();
    let (tool_id, _tool_name) =
        if let Some(ContentBlock::ToolUse { id, name, .. }) = tool_uses.first() {
            (id.clone(), name.clone())
        } else {
            panic!("Expected tool use");
        };

    // Second request - provide tool result
    let mut client2 = Messages::new();
    client2
        .model("claude-sonnet-4-20250514")
        .max_tokens(200)
        .tools(vec![tool.to_value()])
        .user("Calculate 15 * 7 for me.");

    // Add assistant's response with tool use
    for block in &response.content {
        client2.add_message(Message::new(Role::Assistant, vec![block.clone()]));
    }

    // Add tool result
    client2.tool_result(&tool_id, &("105".to_string()));

    let response2 = client2.post().await.expect("API call failed");

    // Should now have a text response mentioning the result
    let text = response2.get_text();
    println!("Final response: {}", text);
    assert!(text.contains("105"), "Response should contain the result");
}

/// Test forced tool choice
#[tokio::test]
#[ignore]
async fn test_forced_tool_choice() {
    require_api_key();

    let mut tool = Tool::new("greet");
    tool.description("Generate a greeting").add_string_property(
        "name",
        Some("Name to greet"),
        true,
    );

    let mut client = Messages::new();
    client
        .model("claude-sonnet-4-20250514")
        .max_tokens(200)
        .tools(vec![tool.to_value()])
        .tool_choice(ToolChoice::Tool {
            name: "greet".to_string(),
        })
        .user("My name is Alice.");

    let response = client.post().await.expect("API call failed");

    assert!(response.has_tool_use(), "Should use tool when forced");

    let tool_uses = response.get_tool_uses();
    if let Some(ContentBlock::ToolUse { name, input, .. }) = tool_uses.first() {
        assert_eq!(name, "greet");
        println!("Tool input: {}", input);
    }
}

/// Test stop reason - max tokens
#[tokio::test]
#[ignore]
async fn test_stop_reason_max_tokens() {
    require_api_key();

    let mut client = Messages::new();
    client
        .model("claude-sonnet-4-20250514")
        .max_tokens(5) // Very low limit
        .user("Tell me a very long story about a dragon.");

    let response = client.post().await.expect("API call failed");

    assert!(
        response.hit_max_tokens(),
        "Should hit max tokens with limit of 5"
    );
    println!("Response (truncated): {}", response.get_text());
}

/// Test stop reason - natural end
#[tokio::test]
#[ignore]
async fn test_stop_reason_end_turn() {
    require_api_key();

    let mut client = Messages::new();
    client
        .model("claude-sonnet-4-20250514")
        .max_tokens(100)
        .user("Say 'Hello' and nothing else.");

    let response = client.post().await.expect("API call failed");

    assert!(
        response.stopped_naturally(),
        "Should end naturally: {:?}",
        response.stop_reason
    );
}

/// Test multi-turn conversation
#[tokio::test]
#[ignore]
async fn test_multi_turn_conversation() {
    require_api_key();

    let mut client = Messages::new();
    client
        .model("claude-sonnet-4-20250514")
        .max_tokens(100)
        .user("My name is Bob.")
        .assistant("Nice to meet you, Bob!")
        .user("What's my name?");

    let response = client.post().await.expect("API call failed");
    let text = response.get_text();

    println!("Response: {}", text);
    assert!(
        text.to_lowercase().contains("bob"),
        "Should remember the name: {}",
        text
    );
}

/// Test error handling - invalid API key
#[tokio::test]
#[ignore]
async fn test_invalid_api_key() {
    let mut client = Messages::with_api_key("invalid_key");
    client
        .model("claude-sonnet-4-20250514")
        .max_tokens(100)
        .user("Hello");

    let result = client.post().await;

    assert!(result.is_err(), "Should fail with invalid API key");

    if let Err(e) = result {
        println!("Expected error: {}", e);
        // Should be an authentication error
        let error_str = format!("{}", e);
        let error_lower = error_str.to_lowercase();
        assert!(
            error_lower.contains("authentication")
                || error_lower.contains("api")
                || error_lower.contains("invalid"),
            "Error should mention authentication: {}",
            error_str
        );
    }
}

/// Test error handling - missing model
#[tokio::test]
async fn test_missing_model_error() {
    let mut client = Messages::with_api_key("test_key");
    client.max_tokens(100).user("Hello");

    let result = client.post().await;

    assert!(result.is_err(), "Should fail without model");

    if let Err(AnthropicToolError::MissingRequiredField(field)) = result {
        assert_eq!(field, "model");
    } else {
        panic!("Expected MissingRequiredField error");
    }
}

/// Test error handling - missing messages
#[tokio::test]
async fn test_missing_messages_error() {
    let mut client = Messages::with_api_key("test_key");
    client.model("claude-sonnet-4-20250514").max_tokens(100);

    let result = client.post().await;

    assert!(result.is_err(), "Should fail without messages");

    if let Err(AnthropicToolError::MissingRequiredField(field)) = result {
        assert_eq!(field, "messages");
    } else {
        panic!("Expected MissingRequiredField error");
    }
}

/// Test image from URL (vision)
#[tokio::test]
#[ignore]
async fn test_vision_url() {
    require_api_key();

    let mut client = Messages::new();
    client
        .model("claude-sonnet-4-20250514")
        .max_tokens(200)
        .user_with_image_url(
            "What do you see in this image? Describe it briefly.",
            "https://www.google.com/images/branding/googlelogo/2x/googlelogo_color_272x92dp.png",
        );

    let response = client.post().await.expect("API call failed");
    let text = response.get_text();

    println!("Vision response: {}", text);
    assert!(!text.is_empty(), "Should have a response about the image");
}

/// Test with stop sequences
#[tokio::test]
#[ignore]
async fn test_stop_sequences() {
    require_api_key();

    let mut client = Messages::new();
    client
        .model("claude-sonnet-4-20250514")
        .max_tokens(200)
        .stop_sequences(vec!["STOP".to_string()])
        .user("Count from 1 to 10, then say STOP, then continue to 20.");

    let response = client.post().await.expect("API call failed");
    let text = response.get_text();

    println!("Response: {}", text);

    // Should have stopped at or before STOP
    assert!(
        response.stop_reason == Some(StopReason::StopSequence) || !text.contains("11"),
        "Should stop at STOP sequence"
    );
}

/// Test metadata (user_id)
#[tokio::test]
#[ignore]
async fn test_metadata() {
    require_api_key();

    let mut client = Messages::new();
    client
        .model("claude-sonnet-4-20250514")
        .max_tokens(50)
        .user_id("test-user-123")
        .user("Hello!");

    let response = client.post().await.expect("API call failed");

    assert!(!response.get_text().is_empty());
    println!("Response with user_id: {}", response.get_text());
}

/// Test top_p and top_k parameters
#[tokio::test]
#[ignore]
async fn test_sampling_parameters() {
    require_api_key();

    let mut client = Messages::new();
    client
        .model("claude-sonnet-4-20250514")
        .max_tokens(50)
        .temperature(0.5)
        .top_p(0.9)
        .top_k(40)
        .user("Write a haiku about coding.");

    let response = client.post().await.expect("API call failed");
    let text = response.get_text();

    println!("Haiku: {}", text);
    assert!(!text.is_empty(), "Should generate a haiku");
}

/// Test response helper methods
#[tokio::test]
#[ignore]
async fn test_response_helpers() {
    require_api_key();

    let mut client = Messages::new();
    client
        .model("claude-sonnet-4-20250514")
        .max_tokens(100)
        .user("Hello!");

    let response = client.post().await.expect("API call failed");

    // Test text() vs get_text()
    let text1 = response.text();
    let text2 = response.get_text();
    assert_eq!(text1, Some(text2.clone()));

    // Test has_thinking (should be false for normal responses)
    assert!(!response.has_thinking());

    // Test usage
    assert!(response.usage.total_tokens() > 0);

    println!(
        "Total tokens: {} (input: {}, output: {})",
        response.usage.total_tokens(),
        response.usage.input_tokens,
        response.usage.output_tokens
    );
}
