# slack-rs

A comprehensive Rust port of the [Python Slack SDK](https://github.com/slackapi/python-slack-sdk), providing full access to the Slack API Platform with type-safe, idiomatic Rust interfaces.

## Status: In Development 🚧

This project is actively being developed using a Test-Driven Development (TDD) approach, migrating tests from the Python SDK before implementing functionality.

### Completed Features

- ✅ **HTTP Retry System**
  - Exponential backoff with configurable intervals
  - Rate limit handling (HTTP 429)
  - Server error handling (HTTP 5xx)
  - Connection error handling
  - Custom retry handlers
  - 94% test coverage

- ✅ **Block Kit Models**
  - All 7 core block types
  - All 19 element types
  - View system (Modal, Home)
  - Composition objects
  - Comprehensive validation
  - 94% test coverage

- ✅ **Signature Verification**
  - HMAC-SHA256 request validation
  - 5-minute timestamp expiration
  - Constant-time comparison
  - 86% test coverage

- ✅ **Webhook Client**
  - Sync and async webhook sending
  - Text, blocks, and attachments
  - Response types (in_channel, ephemeral)
  - 95% test coverage

- ✅ **Web API Client**
  - 287 Slack API methods implemented
  - Automatic pagination support
  - File upload (files_upload_v2)
  - Sync and async clients
  - 85% test coverage

### Coming Soon
- **OAuth Implementation** - Complete OAuth 2.0 flow
- **Socket Mode** - Real-time WebSocket events
- **Enterprise APIs** - SCIM, Audit Logs

## Quick Start

### Building Slack Messages with Block Kit

```rust
use slack_rs::{
    SectionBlock, DividerBlock, HeaderBlock, InputBlock,
    PlainTextInputElement, ActionsBlock, ButtonElement, ButtonStyle,
    View, TextObject, SlackOption, CheckboxesElement,
};

// Create a simple message
let blocks = vec![
    HeaderBlock::new("Daily Report")?.into(),
    DividerBlock::new().into(),
    SectionBlock::new("*Sales*: $1,234")?.into(),
    SectionBlock::new("*Users*: 5,678")?.into(),
];

// Create an interactive modal
let modal = View::modal()
    .title("User Survey")
    .submit("Submit")
    .close("Cancel")
    .blocks(vec![
        InputBlock::new("Your Name")
            .element(PlainTextInputElement::new("name_input"))
            .build()?,

        InputBlock::new("Interests")
            .element(CheckboxesElement::new("interests", vec![
                SlackOption::new("Rust", "rust")?,
                SlackOption::new("APIs", "apis")?,
            ]))
            .optional(true)
            .build()?,

        ActionsBlock::builder()
            .elements(vec![
                ButtonElement::new("Submit", "submit_btn")
                    .with_style(ButtonStyle::Primary)
                    .build()?
            ])
            .build()?,
    ])
    .build()?;

// Serialize to JSON
let json = serde_json::to_string(&modal)?;
```

### HTTP Retry

```rust
use slack_rs::http_retry::{
    RateLimitErrorHandler, RetryHandler, RetryState, HttpResponse
};

let mut handler = RateLimitErrorHandler::new(3);
let mut state = RetryState::new();

let response = HttpResponse {
    status_code: 429,
    headers: vec![("Retry-After".to_string(), "60".to_string())],
    body: vec![],
};

if handler.can_retry(&state, Some(&response), None) {
    handler.prepare_for_next_attempt(&mut state);
    let wait_time = state.duration_until_next_attempt();
    println!("Will retry after {:?}", wait_time);
}
```

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
slack-rs = "0.1"
tokio = { version = "1", features = ["full"] }
serde_json = "1"
```

## Testing

```bash
# Run all tests
cargo test

# Run with coverage
cargo llvm-cov --lib --html
open target/llvm-cov/html/index.html

# Run with logging
RUST_LOG=debug cargo test -- --nocapture
```

## Code Quality

All code is:
- ✅ Formatted with `rustfmt`
- ✅ Linted with `clippy` (zero warnings)
- ✅ Tested with 94%+ coverage
- ✅ Documented with rustdoc

```bash
# Verify code quality
cargo fmt -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features --workspace
```

## Features

### Implemented

**HTTP Retry:**
- Automatic retry with exponential backoff
- Rate limit, server error, and connection handlers
- Configurable interval calculators
- Structured logging

**Block Kit:**
- 7 block types (Section, Header, Input, Actions, Context, Image, Divider)
- 19 element types (Button, all Select variants, DatePicker, Checkboxes, etc.)
- View system (Modal, Home)
- Full validation and error handling

**Signature Verification:**
- HMAC-SHA256 request validation
- Timestamp expiration (5-minute window)
- Constant-time comparison

**Webhook Client:**
- Synchronous and asynchronous clients
- Text, blocks, and attachments support
- Response types and options

**Web API Client:**
- 287 API methods (chat, conversations, users, files, views, admin, etc.)
- Automatic pagination with iterator support
- File upload support
- Rate limiting and retry handling
- Sync (WebClient) and async (AsyncWebClient) versions

### Planned
- OAuth 2.0 flow
- Socket Mode (WebSocket)
- SCIM API (Enterprise)
- Audit Logs API (Enterprise)

## Contributing

Contributions are welcome! This project follows a strict TDD approach where tests are migrated from the Python SDK before implementation.

## License

MIT

## Acknowledgments

- [Python Slack SDK](https://github.com/slackapi/python-slack-sdk)
- [Slack API Documentation](https://api.slack.com/)

---

**Version:** 0.1.0
**MSRV:** Rust 1.75+
**Tests:** 243 passing
**Coverage:** 94%
