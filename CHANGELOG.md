# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Audit Logs API client (sync and async) with bearer token authentication
- Comprehensive examples directory with 5 example applications:
  - `basic_bot.rs`: Simple message sending bot
  - `oauth_app.rs`: OAuth 2.0 flow implementation
  - `socket_mode_app.rs`: Real-time event handling
  - `webhook_handler.rs`: Webhook endpoint with signature verification
  - `block_kit_builder.rs`: Rich UI component building
- Production-ready benchmarks for Block Kit serialization and Web API operations
- Complete API documentation for all public interfaces
- CONTRIBUTING.md with development guidelines
- Integration test framework

### Changed
- Updated README.md to reflect production-ready status
- Enhanced documentation with usage examples

## [0.1.0] - 2025-01-16

### Added
- HTTP Retry System with exponential backoff
  - Rate limit handling (HTTP 429)
  - Server error handling (HTTP 5xx)
  - Connection error handling
  - Configurable retry handlers
  - 94% test coverage

- Block Kit Models
  - 7 core block types (Section, Header, Input, Actions, Context, Image, Divider)
  - 19 element types (Button, all Select variants, DatePicker, Checkboxes, etc.)
  - View system (Modal, Home)
  - Composition objects
  - Comprehensive validation
  - 94% test coverage

- Signature Verification
  - HMAC-SHA256 request validation
  - 5-minute timestamp expiration
  - Constant-time comparison
  - 86% test coverage

- Webhook Client
  - Sync and async webhook sending
  - Text, blocks, and attachments support
  - Response types (in_channel, ephemeral)
  - 95% test coverage

- Web API Client
  - 287 Slack API methods implemented
  - Automatic pagination support
  - File upload (files_upload_v2)
  - Sync (WebClient) and async (AsyncWebClient) clients
  - 85% test coverage

- OAuth Implementation
  - Complete OAuth 2.0 flow
  - 4 storage backends (Cache, File, SQLite, State)
  - Token rotation support
  - Enterprise Grid support
  - 51 tests, 85% coverage

- Socket Mode
  - WebSocket client for real-time events
  - Event handler system
  - Auto-reconnection
  - 19 tests

- SCIM API
  - User and group management
  - SCIM 2.0 compliant
  - CRUD operations with PATCH support
  - Search and filtering
  - 16 tests

### Testing
- 407+ passing tests across all modules
- Test-Driven Development (TDD) approach
- Tests ported from Python Slack SDK
- Zero warnings with Clippy
- Comprehensive documentation tests

### Documentation
- Complete rustdoc for all public APIs
- 5 working examples
- README with quick start guide
- Architecture documentation

## Release Notes

### Initial Release (0.1.0)

This is the initial release of slack-rs, a comprehensive Rust port of the Python Slack SDK. The project was developed using a Test-Driven Development approach, ensuring high code quality and reliability from the start.

**Highlights:**
- Feature parity with core Python SDK functionality
- Type-safe, idiomatic Rust interfaces
- Async/await support with Tokio
- Comprehensive error handling
- Production-ready with 407+ tests
- Extensive documentation and examples

**Getting Started:**
```toml
[dependencies]
slack-rs = "0.1"
tokio = { version = "1", features = ["full"] }
```

**Simple Example:**
```rust
use slack_rs::web::AsyncWebClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = AsyncWebClient::new("xoxb-your-token");
    let response = client.chat_post_message(
        "#general",
        "Hello from Rust!",
        None, None, None, None, None, None, None, None,
        None, None, None, None, None, None, None, None, None, None,
    ).await?;
    Ok(())
}
```

## Versioning Strategy

This project follows Semantic Versioning:
- **MAJOR**: Incompatible API changes
- **MINOR**: Backward-compatible functionality additions
- **PATCH**: Backward-compatible bug fixes

## Migration Guides

### From Python SDK

The Rust SDK maintains similar patterns to the Python SDK while leveraging Rust's type system and ownership model:

**Python:**
```python
from slack_sdk import WebClient
client = WebClient(token="xoxb-token")
response = client.chat_postMessage(channel="#general", text="Hello")
```

**Rust:**
```rust
use slack_rs::web::AsyncWebClient;
let client = AsyncWebClient::new("xoxb-token");
let response = client.chat_post_message("#general", "Hello", ...).await?;
```

Key differences:
- Async by default (use `WebClient` for sync operations)
- Error handling with `Result` type
- Type-safe builder patterns for complex structures
- Explicit parameter naming

[Unreleased]: https://github.com/slackapi/slack-rs/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/slackapi/slack-rs/releases/tag/v0.1.0
