# Slack SDK Rust Migration Plan

## Project Goal
Port the Python Slack SDK from [slackapi/python-slack-sdk](https://github.com/slackapi/python-slack-sdk) (version 3.38.0) to Rust, creating a complete, production-ready Rust implementation of the Slack API Platform SDK.

## Development Approach: Test-Driven Development (TDD)

**Critical Principle:** Tests must be migrated BEFORE implementation.

### TDD Strategy
1. **Phase 1: Test Migration** - Port Python tests to Rust for each component
2. **Phase 2: Implementation** - Write Rust code to pass the migrated tests
3. **Phase 3: Validation** - Ensure functional correctness against benchmarks

This approach ensures:
- Functional correctness guaranteed by existing test suite
- Clear definition of "done" (all tests pass)
- Regression prevention during development
- API compatibility with Python SDK patterns

---

## Python SDK Analysis Summary

### Current State
- **Version:** 3.38.0
- **License:** MIT
- **Python Version:** 3.7+
- **Main Package:** `slack_sdk` (118 Python files)
- **Test Suite:** 165 test files, ~900 test methods, ~18,000 lines of test code
- **Dependencies:** Minimal (stdlib only for core, optional: aiohttp, boto3, SQLAlchemy)

### Core Components
1. **Web API Client** - 260+ API methods, pagination, file upload
2. **Webhook Client** - Incoming webhooks and response_url
3. **Socket Mode** - Real-time WebSocket events (4 implementations)
4. **OAuth** - Complete OAuth 2.0 flow with multiple storage backends
5. **Block Kit Models** - Type-safe builders for UI components
6. **HTTP Retry** - Exponential backoff with custom handlers
7. **Signature Verification** - HMAC-SHA256 request validation
8. **Audit Logs API** - Enterprise audit log access
9. **SCIM API** - User provisioning (SCIM 2.0)
10. **RTM API** - Legacy real-time messaging (low priority)

### Architecture Patterns
- Dual sync/async implementation (urllib + aiohttp)
- Builder pattern for Block Kit models
- Retry handler chain
- Storage abstraction for OAuth
- Response objects with automatic pagination
- Mock servers for comprehensive testing

---

## Rust Migration Phases

### Phase 0: Project Setup 

**Status:** Basic project initialized

**Tasks:**
- [x] Create Cargo workspace
- [x] Set up basic project structure
- [ ] Add core dependencies (latest versions)
- [ ] Configure linting and formatting
- [ ] Configure CI/CD pipeline
- [ ] Set up documentation structure
- [ ] Set up logging infrastructure

**Dependencies to Add (Latest Versions):**
```toml
[package]
name = "slack-rs"
version = "0.1.0"
edition = "2021"
rust-version = "1.75"  # MSRV - latest stable features

[dependencies]
# HTTP client with all features
reqwest = { version = "0.12", features = [
    "json",
    "multipart",
    "stream",
    "blocking",
    "gzip",
    "brotli",
    "deflate"
] }

# Async runtime
tokio = { version = "1.40", features = ["full"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Error handling
thiserror = "2.0"
anyhow = "1.0"

# WebSocket
tokio-tungstenite = "0.24"

# Cryptography (for signature verification)
hmac = "0.12"
sha2 = "0.10"

# Utilities
url = "2.5"
bytes = "1.7"
futures = "0.3"

# Logging (mirroring Python SDK logging patterns)
tracing = "0.1"           # Structured logging
tracing-subscriber = { version = "0.3", features = ["env-filter", "fmt", "json"] }
log = "0.4"               # For compatibility with other crates

# HTTP server for mock testing
axum = "0.7"              # Latest axum for mock servers in tests
tower = "0.5"             # Middleware for retry logic
tower-http = "0.6"        # HTTP middleware utilities

# Database support for OAuth storage
rusqlite = { version = "0.32", optional = true }
sqlx = { version = "0.8", optional = true, features = ["runtime-tokio", "sqlite", "postgres", "mysql"] }

# AWS S3 support for OAuth storage
aws-sdk-s3 = { version = "1.50", optional = true }
aws-config = { version = "1.5", optional = true }

# Time handling
chrono = "0.4"

[dev-dependencies]
# Testing frameworks
wiremock = "0.6"          # HTTP mocking
axum-test = "15.0"        # Axum testing utilities
tokio-test = "0.4"
assert_json_diff = "2.0"
pretty_assertions = "1.4"
test-case = "3.3"         # Parameterized tests
proptest = "1.4"          # Property-based testing

# Test utilities
tempfile = "3.12"         # Temporary files for testing
mockall = "0.13"          # Mocking framework

# Benchmarking
criterion = { version = "0.5", features = ["html_reports", "async_tokio"] }

[features]
default = ["sqlite"]
full = ["sqlite", "postgres", "mysql", "s3"]
sqlite = ["rusqlite"]
postgres = ["sqlx/postgres"]
mysql = ["sqlx/mysql"]
s3 = ["aws-sdk-s3", "aws-config"]

[[bench]]
name = "web_client"
harness = false

[[bench]]
name = "block_kit_serialization"
harness = false
```

**Code Quality Configuration:**

Create `rustfmt.toml`:
```toml
# Rust formatting configuration
edition = "2021"
max_width = 100
hard_tabs = false
tab_spaces = 4
newline_style = "Unix"
use_small_heuristics = "Default"
reorder_imports = true
reorder_modules = true
remove_nested_parens = true
format_code_in_doc_comments = true
wrap_comments = true
comment_width = 100
normalize_comments = true
normalize_doc_attributes = true
```

Create `clippy.toml`:
```toml
# Clippy linting configuration
msrv = "1.75"
cognitive-complexity-threshold = 30
too-many-arguments-threshold = 8
type-complexity-threshold = 500
```

Create `.cargo/config.toml`:
```toml
[target.'cfg(all())']
rustflags = [
    "-W", "rust-2021-compatibility",
    "-W", "missing-docs",
    "-W", "missing-debug-implementations",
]

[build]
# Always use all CPU cores for compilation
jobs = -1

[alias]
# Custom cargo commands
lint = "clippy --all-targets --all-features -- -D warnings"
fmt-check = "fmt --all -- --check"
test-all = "test --all-features --workspace"
```

**Linting Commands:**
```bash
# Format code
cargo fmt

# Check formatting without changing files
cargo fmt -- --check

# Run clippy with all checks
cargo clippy --all-targets --all-features -- -D warnings

# Run all lints (add to CI)
cargo fmt -- --check && cargo clippy --all-targets --all-features -- -D warnings

# Run tests with all features
cargo test --all-features --workspace
```

---

### Phase 1: Foundation & HTTP Retry (Weeks 1-2)

**Priority:** HIGH - Required by all other components

#### Test Migration (Week 1)
- [ ] Port `tests/slack_sdk/http_retry/` tests
  - `test_builtin_handlers.py` � `tests/http_retry/builtin_handlers.rs`
  - `test_builtin_interval_calculators.py` � `tests/http_retry/interval_calculators.rs`
  - `test_handler.py` � `tests/http_retry/handler.rs`
  - `test_state.py` � `tests/http_retry/state.rs`
- [ ] Create mock HTTP server infrastructure
- [ ] Port custom retry handler test fixtures
- [ ] Setup test utilities for retry scenarios

**Test Files:** 8 test files, ~50 test methods

#### Implementation (Week 2)
- [ ] Create error types (`src/error.rs`)
  - `SlackError` enum
  - `SlackApiError` with response details
  - Error conversion traits
- [ ] Implement retry state machine (`src/http_retry/state.rs`)
- [ ] Implement interval calculators (`src/http_retry/interval.rs`)
  - `BackoffIntervalCalculator`
  - `RandomJitterCalculator`
- [ ] Implement retry handlers (`src/http_retry/handler.rs`)
  - `RetryHandler` trait
  - `ConnectionErrorHandler`
  - `RateLimitHandler`
  - `ServerErrorHandler`
- [ ] Integrate retry logic with reqwest middleware

**Deliverable:** Retry system passes all migrated tests

---

### Phase 2: Block Kit Models (Weeks 3-5)

**Priority:** HIGH - Core to SDK, well-defined structure, good TDD candidate

#### Test Migration (Weeks 3-4)
- [ ] Port `tests/slack_sdk/models/blocks/` tests
  - `test_blocks.py` (48KB) � `tests/models/blocks.rs`
  - All block types: Section, Actions, Context, Divider, Header, Image, Input, RichText, Video, File, Call
- [ ] Port `tests/slack_sdk/models/elements/` tests
  - `test_elements.py` (50KB) � `tests/models/elements.rs`
  - All elements: Button, Select menus, DatePicker, TimePicker, Radio, Checkbox, TextInput, FileInput
- [ ] Port `tests/slack_sdk/models/views/` tests
  - `test_views.py` � `tests/models/views.rs`
  - Modal and Home tab views
- [ ] Port `tests/slack_sdk/models/attachments/` tests
  - `test_attachments.py` � `tests/models/attachments.rs`
- [ ] Port `tests/slack_sdk/models/objects/` tests
  - `test_objects.py` � `tests/models/objects.rs`
  - Text objects, Option, OptionGroup, ConfirmationDialog
- [ ] Port `tests/slack_sdk/models/messages/` tests
  - `test_messages.py` � `tests/models/messages.rs`
- [ ] Port JSON fixtures (69 files in `tests/slack_sdk_fixture/`)

**Test Files:** 18 test files, ~200 test methods

#### Implementation (Weeks 4-5)
- [ ] Design core model traits (`src/models/mod.rs`)
  - `Block` trait
  - `Element` trait
  - `CompositionObject` trait
  - JSON serialization/deserialization
- [ ] Implement text objects (`src/models/objects.rs`)
  - `PlainTextObject`
  - `MarkdownTextObject`
  - `Option`, `OptionGroup`
  - `ConfirmationDialog`
- [ ] Implement all block types (`src/models/blocks/`)
  - `section.rs`, `actions.rs`, `context.rs`, `divider.rs`, `header.rs`
  - `image.rs`, `input.rs`, `rich_text.rs`, `video.rs`, `file.rs`, `call.rs`
- [ ] Implement all element types (`src/models/elements/`)
  - `button.rs`, `select.rs`, `overflow.rs`, `datepicker.rs`, `timepicker.rs`
  - `radio_buttons.rs`, `checkboxes.rs`, `text_input.rs`, `file_input.rs`
- [ ] Implement views (`src/models/views.rs`)
  - `ModalView`, `HomeTabView`, `WorkflowStepView`
- [ ] Implement attachments (`src/models/attachments.rs`)
- [ ] Implement messages (`src/models/messages.rs`)
- [ ] Add validation rules (length limits, required fields)
- [ ] Add builder patterns for ergonomic construction

**Deliverable:** Complete Block Kit implementation passes all tests

---

### Phase 3: Signature Verification (Week 6)

**Priority:** HIGH - Security critical, simple implementation

#### Test Migration
- [ ] Port `tests/slack_sdk/signature/test_signature_verifier.py`
  - HMAC generation tests
  - Request validation tests
  - Timestamp expiration tests
  - Edge cases (malformed headers, etc.)

**Test Files:** 1 test file, ~15 test methods

#### Implementation
- [ ] Create `SignatureVerifier` (`src/signature/mod.rs`)
  - HMAC-SHA256 computation
  - Request body + timestamp signing
  - Timestamp validation (5-minute window)
  - Header parsing
- [ ] Add async/sync variants

**Deliverable:** Signature verification passes all tests

---

### Phase 4: Webhook Client (Week 7)

**Priority:** HIGH - Simple, commonly used, good early win

#### Test Migration
- [ ] Port `tests/slack_sdk/webhook/` tests
  - `test_webhook.py` � `tests/webhook/client.rs`
  - Text, blocks, attachments sending
  - Response types (ephemeral, in_channel)
  - Timeout handling
  - Proxy support
  - Error scenarios
- [ ] Port async webhook tests
  - `test_async_webhook.py` � `tests/webhook/async_client.rs`

**Test Files:** 4 test files, ~40 test methods

#### Implementation
- [ ] Create `WebhookClient` (`src/webhook/client.rs`)
  - Sync implementation using `reqwest::blocking`
  - POST method with JSON body
  - Support text, blocks, attachments
  - Response parsing
  - Timeout configuration
  - Proxy configuration
- [ ] Create `AsyncWebhookClient` (`src/webhook/async_client.rs`)
  - Async implementation using `reqwest`
  - Same API as sync version
- [ ] Create `WebhookResponse` (`src/webhook/response.rs`)
  - Status code
  - Body parsing

**Deliverable:** Webhook client (sync + async) passes all tests

---

### Phase 5: Web API Client - Core (Weeks 8-11)

**Priority:** CRITICAL - Central component, most complex

#### Test Migration (Weeks 8-9)
- [ ] Port `tests/slack_sdk/web/` core tests
  - `test_web_client.py` � `tests/web/client.rs`
  - `test_slack_response.py` � `tests/web/response.rs`
  - `test_base_client.py` � `tests/web/base_client.rs`
  - `test_internal_utils.py` � `tests/web/internal_utils.rs`
- [ ] Port pagination tests
- [ ] Port file upload tests
- [ ] Port error handling tests
- [ ] Port user agent tests
- [ ] Port rate limiting tests
- [ ] Setup mock Web API server for tests
  - Port `mock_web_api_handler.py`
  - Token-based routing
  - State management for retry scenarios

**Test Files:** 15 test files, ~300 test methods

#### Implementation (Weeks 9-11)
- [ ] Create `WebClient` (`src/web/client.rs`)
  - Base HTTP client with retry integration
  - Authentication (Bearer token)
  - Request building
  - Response parsing
  - Error handling
- [ ] Create `SlackResponse` (`src/web/response.rs`)
  - JSON response wrapper
  - Pagination iterator support
  - Status code, headers
- [ ] Implement internal utilities (`src/web/internal_utils.rs`)
  - User agent construction
  - URL building
  - Header management
- [ ] Implement core API methods (priority subset):
  - `api_test()`
  - `auth_test()`
  - `chat_post_message()`
  - `chat_update()`
  - `chat_delete()`
  - `conversations_list()`
  - `conversations_history()`
  - `users_list()`
  - `users_info()`
  - `files_upload()` (v2 and legacy)
- [ ] Create `AsyncWebClient` (`src/web/async_client.rs`)
  - Async version with same API
- [ ] Implement pagination
  - Iterator trait for cursored responses
  - Automatic page fetching

**Deliverable:** Core Web API client with essential methods passes all tests

---

### Phase 6: Web API Client - Complete Coverage (Weeks 12-14)

**Priority:** HIGH - Comprehensive API coverage

#### Test Migration (Week 12)
- [ ] Port remaining Web API tests
  - Admin API tests
  - Apps API tests
  - Calls API tests
  - Search API tests
  - Team API tests
  - Usergroups API tests
  - Workflows API tests
  - And all other API categories

**Test Files:** ~30 additional test files

#### Implementation (Weeks 13-14)
- [ ] Implement all 260+ Web API methods
  - Use code generation or macros where appropriate
  - Group by API category (admin, apps, auth, bookmarks, calls, chat, conversations, etc.)
- [ ] Create method parameter structs
  - Type-safe parameters with builder pattern
  - Optional fields
  - Validation
- [ ] Add specialized response types
  - Conversation types
  - User types
  - Message types
  - File types

**Deliverable:** Complete Web API client with all methods passes all tests

---

### Phase 7: OAuth Implementation (Weeks 15-17)

**Priority:** HIGH - Required for multi-workspace apps

#### Test Migration (Week 15)
- [ ] Port OAuth tests
  - `test_generator.py` � `tests/oauth/authorize_url_generator.rs`
  - `test_models.py` � `tests/oauth/models.rs`
  - `test_installation_store/test_file.py` � `tests/oauth/installation_store/file.rs`
  - `test_installation_store/test_sqlite3.py` � `tests/oauth/installation_store/sqlite3.rs`
  - `test_state_store/` tests
  - `test_token_rotation.py` � `tests/oauth/token_rotation.rs`
- [ ] Port OpenID Connect tests

**Test Files:** 25 test files, ~100 test methods

#### Implementation (Weeks 16-17)
- [ ] Create OAuth models (`src/oauth/models.rs`)
  - `Installation`
  - `Bot`
  - `OAuthV2Response`
- [ ] Create authorize URL generator (`src/oauth/authorize_url_generator.rs`)
  - URL building
  - Scope formatting
  - State management
- [ ] Create installation store trait (`src/oauth/installation_store/mod.rs`)
  - `InstallationStore` trait (async)
  - Find/save/delete operations
- [ ] Implement file-based storage (`src/oauth/installation_store/file.rs`)
- [ ] Implement SQLite storage (`src/oauth/installation_store/sqlite.rs`)
  - Using `rusqlite` or `sqlx`
- [ ] Create state store trait (`src/oauth/state_store/mod.rs`)
  - `OAuthStateStore` trait
  - TTL support
- [ ] Implement file-based state store
- [ ] Implement SQLite state store
- [ ] Create token rotation handler (`src/oauth/token_rotation.rs`)
  - Automatic refresh
  - Storage update

**Deliverable:** OAuth flow with file and SQLite storage passes all tests

---

### Phase 8: OAuth - Additional Storage Backends (Week 18)

**Priority:** MEDIUM - Optional storage backends

#### Test Migration
- [ ] Port storage backend tests
  - SQLAlchemy � SQLx tests
  - Amazon S3 tests
  - In-memory cache tests

#### Implementation
- [ ] Implement SQLx storage (`src/oauth/installation_store/sqlx.rs`)
  - PostgreSQL support
  - MySQL support
- [ ] Implement S3 storage (`src/oauth/installation_store/s3.rs`)
  - Using `aws-sdk-s3`
- [ ] Implement in-memory cache (`src/oauth/installation_store/cache.rs`)
  - Thread-safe cache
  - TTL support

**Deliverable:** Additional storage backends pass all tests

---

### Phase 9: Socket Mode - Core (Weeks 19-21)

**Priority:** MEDIUM - Real-time event handling

#### Test Migration (Week 19)
- [ ] Port Socket Mode tests
  - `test_builtin.py` � `tests/socket_mode/builtin.rs`
  - `test_request.py` � `tests/socket_mode/request.rs`
  - `test_response.py` � `tests/socket_mode/response.rs`
  - `test_interactions_builtin.py` � `tests/socket_mode/interactions.rs`
- [ ] Port mock Socket Mode server
  - `mock_socket_mode_server.py` � test utilities

**Test Files:** 12 test files, ~80 test methods

#### Implementation (Weeks 20-21)
- [ ] Create Socket Mode request/response types (`src/socket_mode/types.rs`)
  - `SocketModeRequest`
  - `SocketModeResponse`
  - Envelope parsing
- [ ] Create `SocketModeClient` (`src/socket_mode/client.rs`)
  - WebSocket connection using `tokio-tungstenite`
  - Connection establishment via `apps.connections.open`
  - Event receiving
  - Acknowledgment sending
  - Auto-reconnection logic
- [ ] Create listener system (`src/socket_mode/listener.rs`)
  - Event type routing
  - Async handler registration
  - Thread pool for concurrent processing
- [ ] Add connection management
  - Keepalive (ping/pong)
  - State machine
  - Error recovery

**Deliverable:** Socket Mode client passes all tests

---

### Phase 10: SCIM API (Week 22)

**Priority:** LOW - Enterprise-only feature

#### Test Migration
- [ ] Port SCIM tests
  - `test_client.py` � `tests/scim/client.rs`
  - User CRUD tests
  - Group CRUD tests
  - Search tests

**Test Files:** 3 test files, ~30 test methods

#### Implementation
- [ ] Create `ScimClient` (`src/scim/client.rs`)
  - User operations
  - Group operations
  - Search with filters
- [ ] Create SCIM models (`src/scim/models.rs`)
  - `User`, `Group`
  - SCIM 2.0 schema compliance

**Deliverable:** SCIM client passes all tests

---

### Phase 11: Audit Logs API (Week 23)

**Priority:** LOW - Enterprise-only feature

#### Test Migration
- [ ] Port Audit Logs tests
  - `test_client.py` � `tests/audit_logs/client.rs`
  - `test_response.py` � `tests/audit_logs/response.rs`
  - Schema and action tests

**Test Files:** 3 test files, ~25 test methods

#### Implementation
- [ ] Create `AuditLogsClient` (`src/audit_logs/client.rs`)
  - Schemas endpoint
  - Actions endpoint
  - Logs endpoint with pagination
- [ ] Create audit log models (`src/audit_logs/models.rs`)
  - `LogEntry`
  - Cursor-based pagination

**Deliverable:** Audit Logs client passes all tests

---

### Phase 12: RTM API (Week 24) - OPTIONAL

**Priority:** VERY LOW - Deprecated by Slack

**Decision:** Skip unless specifically requested. RTM is legacy and Slack recommends Socket Mode instead.

---

### Phase 13: Polish & Documentation (Weeks 25-26)

**Priority:** HIGH - Production readiness

#### Tasks
- [ ] API documentation (rustdoc)
  - All public APIs documented
  - Code examples
  - Usage patterns
- [ ] README.md
  - Getting started guide
  - Feature comparison with Python SDK
  - Migration guide
  - Examples
- [ ] CONTRIBUTING.md
- [ ] CHANGELOG.md
- [ ] Integration examples
  - Basic bot
  - OAuth app
  - Socket Mode app
  - Webhook handler
- [ ] Performance benchmarks
- [ ] Security audit
  - Dependency audit
  - Code review
  - Penetration testing considerations
- [ ] CI/CD finalization (see CI/CD Configuration section below)
  - Test coverage reporting (cargo-tarpaulin or cargo-llvm-cov)
  - Linting (clippy with deny warnings)
  - Formatting (rustfmt with check)
  - Security audit (cargo-audit)
  - Dependency updates (dependabot)
  - Release automation (cargo-release)
  - Documentation generation and publishing

---

### Phase 14: Integration Testing & Validation (Week 27-28)

**Priority:** CRITICAL - Final validation

#### Tasks
- [ ] Port integration tests
  - Real API integration tests (require tokens)
  - End-to-end workflows
- [ ] Create test Slack workspace
- [ ] Manual testing of all features
- [ ] Performance testing
  - Load testing Web API client
  - Socket Mode connection stability
  - Memory leak detection
- [ ] Compatibility testing
  - Compare responses with Python SDK
  - Validate Block Kit rendering
- [ ] Beta release
  - Publish to crates.io (0.1.0-beta)
  - Gather community feedback

---

## Project Structure

```
slack-rs/
   Cargo.toml                   # Workspace configuration
   Cargo.lock
   README.md
   CLAUDE.md                    # This file
   CHANGELOG.md
   LICENSE                      # MIT

   src/
      lib.rs                   # Public API exports
      error.rs                 # Error types
   
      web/                     # Web API client
         mod.rs
         client.rs            # Sync client
         async_client.rs      # Async client
         response.rs          # Response types
         internal_utils.rs    # Internal utilities
         methods/             # API method implementations
             mod.rs
             chat.rs
             conversations.rs
             users.rs
             ...
   
      webhook/                 # Webhook client
         mod.rs
         client.rs
         async_client.rs
         response.rs
   
      socket_mode/             # Socket Mode client
         mod.rs
         client.rs
         types.rs
         listener.rs
         connection.rs
   
      oauth/                   # OAuth implementation
         mod.rs
         models.rs
         authorize_url_generator.rs
         installation_store/
            mod.rs
            file.rs
            sqlite.rs
            sqlx.rs
            s3.rs
            cache.rs
         state_store/
            mod.rs
            file.rs
            sqlite.rs
         token_rotation.rs
   
      models/                  # Block Kit models
         mod.rs
         blocks/
            mod.rs
            section.rs
            actions.rs
            context.rs
            ...
         elements/
            mod.rs
            button.rs
            select.rs
            ...
         objects.rs           # Text objects, options, etc.
         views.rs             # Modal, home tab views
         attachments.rs       # Legacy attachments
         messages.rs          # Message types
   
      http_retry/              # HTTP retry logic
         mod.rs
         handler.rs           # Retry handler trait
         builtin_handlers.rs  # Connection, rate limit, server error
         interval.rs          # Backoff calculators
         state.rs             # Retry state
   
      signature/               # Request verification
         mod.rs
   
      scim/                    # SCIM API
         mod.rs
         client.rs
         models.rs
   
      audit_logs/              # Audit Logs API
          mod.rs
          client.rs
          models.rs

   tests/                       # Integration tests
      common/                  # Shared test utilities
         mod.rs
         mock_server.rs       # Mock HTTP server
         fixtures.rs          # JSON fixtures
         assertions.rs        # Custom assertions
   
      web/                     # Web client tests
         client.rs
         response.rs
         pagination.rs
         ...
   
      webhook/                 # Webhook tests
      socket_mode/             # Socket Mode tests
      oauth/                   # OAuth tests
      models/                  # Block Kit model tests
      http_retry/              # Retry logic tests
      signature/               # Signature verification tests
      scim/                    # SCIM tests
      audit_logs/              # Audit Logs tests

   examples/                    # Usage examples
      basic_bot.rs
      oauth_app.rs
      socket_mode_app.rs
      webhook_handler.rs
      block_kit_builder.rs

   benches/                     # Performance benchmarks
       web_client.rs
       block_kit_serialization.rs
```

---

## Testing Strategy

### Test Organization
- **Unit tests:** Inline with source code (`#[cfg(test)] mod tests`) - REQUIRED for all modules
- **Integration tests:** In `tests/` directory
- **Test utilities:** `tests/common/` module
- **Fixtures:** Embedded JSON files using `include_str!`

### Unit Testing Requirements

**All modules MUST have unit tests covering:**

1. **Happy path scenarios** - Normal, expected usage
2. **Error cases** - Invalid inputs, network failures, API errors
3. **Edge cases** - Boundary conditions, empty values, maximum sizes
4. **Serialization/deserialization** - JSON round-trips for all models
5. **Builder patterns** - All builder methods and validation
6. **State transitions** - For state machines (retry, connection, etc.)

**Unit Test Structure:**

```rust
// src/models/blocks/section.rs

use serde_json::json;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SectionBlock {
    #[serde(rename = "type")]
    pub block_type: String,
    pub text: Option<TextObject>,
    pub block_id: Option<String>,
    // ... other fields
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn test_section_block_minimal() {
        // Test minimal valid construction
        let block = SectionBlock {
            block_type: "section".to_string(),
            text: Some(TextObject::plain("Hello")),
            block_id: None,
        };

        assert_eq!(block.block_type, "section");
    }

    #[test]
    fn test_section_block_serialization() {
        // Test JSON serialization round-trip
        let input = json!({
            "type": "section",
            "text": {
                "type": "plain_text",
                "text": "Hello world"
            }
        });

        let block: SectionBlock = serde_json::from_value(input.clone()).unwrap();
        let output = serde_json::to_value(&block).unwrap();

        assert_eq!(input, output);
    }

    #[test]
    fn test_section_block_deserialization_error() {
        // Test invalid JSON handling
        let invalid = json!({
            "type": "section",
            "text": "not an object"  // Invalid structure
        });

        let result: Result<SectionBlock, _> = serde_json::from_value(invalid);
        assert!(result.is_err());
    }

    #[test]
    fn test_section_block_builder() {
        // Test builder pattern
        let block = SectionBlock::builder()
            .text("Hello")
            .block_id("section1")
            .build()
            .unwrap();

        assert_eq!(block.block_id, Some("section1".to_string()));
    }

    #[test]
    fn test_section_block_validation() {
        // Test validation rules (e.g., text length limits)
        let long_text = "a".repeat(3001);
        let result = SectionBlock::builder()
            .text(&long_text)
            .build();

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SlackError::Validation(_)));
    }
}
```

**Error Handling Unit Tests:**

```rust
// src/error.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slack_api_error_display() {
        let error = SlackApiError {
            error: "channel_not_found".to_string(),
            status_code: 404,
        };

        assert_eq!(
            error.to_string(),
            "Slack API error (404): channel_not_found"
        );
    }

    #[test]
    fn test_error_conversion_from_reqwest() {
        let reqwest_error = reqwest::Error::from(/* ... */);
        let slack_error: SlackError = reqwest_error.into();

        assert!(matches!(slack_error, SlackError::Http(_)));
    }
}
```

**HTTP Retry Unit Tests:**

```rust
// src/http_retry/handler.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limit_handler_detects_429() {
        let handler = RateLimitHandler::default();
        let response = HttpResponse {
            status_code: 429,
            headers: HashMap::new(),
            body: vec![],
        };
        let state = RetryState::default();

        assert!(handler.can_retry(&state, &response));
    }

    #[test]
    fn test_rate_limit_handler_respects_max_attempts() {
        let handler = RateLimitHandler::new(3);
        let mut state = RetryState {
            current_attempt: 3,
            ..Default::default()
        };

        let response = HttpResponse {
            status_code: 429,
            headers: HashMap::new(),
            body: vec![],
        };

        assert!(!handler.can_retry(&state, &response));
    }

    #[test]
    fn test_backoff_calculator() {
        let calculator = BackoffIntervalCalculator::new(2.0);

        assert_eq!(calculator.calculate(1), 1);
        assert_eq!(calculator.calculate(2), 2);
        assert_eq!(calculator.calculate(3), 4);
        assert_eq!(calculator.calculate(4), 8);
    }

    #[test]
    fn test_backoff_calculator_with_max() {
        let calculator = BackoffIntervalCalculator::new(2.0)
            .with_max_interval(10);

        assert_eq!(calculator.calculate(5), 10); // Would be 16, capped at 10
    }
}
```

**WebClient Unit Tests:**

```rust
// src/web/client.rs

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_construction() {
        let client = WebClient::new("xoxb-test-token").unwrap();

        assert_eq!(client.token, "xoxb-test-token");
        assert_eq!(client.base_url, "https://slack.com/api/");
    }

    #[test]
    fn test_client_strips_token_whitespace() {
        let client = WebClient::new("  xoxb-test-token  ").unwrap();

        assert_eq!(client.token, "xoxb-test-token");
    }

    #[test]
    fn test_client_empty_token_error() {
        let result = WebClient::new("");

        assert!(result.is_err());
        assert!(matches!(result.unwrap_err(), SlackError::InvalidToken));
    }

    #[test]
    fn test_user_agent_construction() {
        let client = WebClient::builder()
            .token("xoxb-test")
            .user_agent_prefix("MyApp/1.0")
            .build()
            .unwrap();

        let user_agent = client.get_user_agent();
        assert!(user_agent.contains("MyApp/1.0"));
        assert!(user_agent.contains("slack-rs/"));
    }

    #[test]
    fn test_api_url_building() {
        let client = WebClient::new("xoxb-test").unwrap();
        let url = client.build_url("chat.postMessage");

        assert_eq!(url, "https://slack.com/api/chat.postMessage");
    }

    #[test]
    fn test_custom_base_url() {
        let client = WebClient::builder()
            .token("xoxb-test")
            .base_url("https://custom.slack.com/api/")
            .build()
            .unwrap();

        assert_eq!(client.base_url, "https://custom.slack.com/api/");
    }

    #[test]
    fn test_base_url_trailing_slash_added() {
        let client = WebClient::builder()
            .token("xoxb-test")
            .base_url("https://slack.com/api")
            .build()
            .unwrap();

        assert_eq!(client.base_url, "https://slack.com/api/");
    }
}
```

**Parameterized Tests:**

```rust
// src/models/objects.rs

use test_case::test_case;

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case("plain_text", "Hello" ; "plain text object")]
    #[test_case("mrkdwn", "*bold*" ; "markdown object")]
    fn test_text_object_types(text_type: &str, text: &str) {
        let json = json!({
            "type": text_type,
            "text": text
        });

        let obj: TextObject = serde_json::from_value(json).unwrap();
        assert_eq!(obj.text, text);
    }

    #[test_case(0 ; "empty string")]
    #[test_case(1 ; "single character")]
    #[test_case(3000 ; "max length")]
    fn test_text_length_validation(length: usize) {
        let text = "a".repeat(length);
        let result = TextObject::plain(&text);

        assert!(result.is_ok());
    }

    #[test_case(3001 ; "over max length")]
    #[test_case(10000 ; "way over max")]
    fn test_text_length_validation_errors(length: usize) {
        let text = "a".repeat(length);
        let result = TextObject::plain(&text);

        assert!(result.is_err());
    }
}
```

**Async Unit Tests:**

```rust
// src/web/async_client.rs

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::test;

    #[tokio::test]
    async fn test_async_client_construction() {
        let client = AsyncWebClient::new("xoxb-test").unwrap();

        assert_eq!(client.token, "xoxb-test");
    }

    #[tokio::test]
    async fn test_concurrent_requests() {
        let client = Arc::new(AsyncWebClient::new("xoxb-test").unwrap());

        let handles: Vec<_> = (0..10)
            .map(|_| {
                let client = Arc::clone(&client);
                tokio::spawn(async move {
                    // Test concurrent API calls
                    client.api_test().await
                })
            })
            .collect();

        for handle in handles {
            assert!(handle.await.is_ok());
        }
    }
}
```

**Property-Based Testing (Optional, Advanced):**

```rust
// Add to dev-dependencies: proptest = "1.4"

#[cfg(test)]
mod proptests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_block_id_any_valid_string(s in "[a-zA-Z0-9_-]{1,255}") {
            let block = SectionBlock::builder()
                .block_id(&s)
                .text("test")
                .build();

            prop_assert!(block.is_ok());
        }

        #[test]
        fn test_text_under_limit(s in ".{0,3000}") {
            let result = TextObject::plain(&s);
            prop_assert!(result.is_ok());
        }
    }
}
```

### Unit Test Coverage Goals

**Per Module:**
- **Minimum 80% line coverage** for all production code
- **100% coverage** for:
  - Public APIs
  - Error handling paths
  - Validation logic
  - Serialization/deserialization

**Test Commands:**
```bash
# Run unit tests only
cargo test --lib

# Run unit tests with coverage
cargo llvm-cov --lib --html

# Run specific module tests
cargo test --lib models::blocks::section

# Run tests with output
cargo test --lib -- --nocapture

# Run tests in parallel
cargo test --lib -- --test-threads=4
```

### Mock Server Strategy
Use `axum` and `wiremock` to create mock Slack API server:
- **Primary:** `axum` for full-featured mock servers (mirrors Python's SimpleHTTPRequestHandler)
- **Alternative:** `wiremock` for simpler endpoint mocking
- Token-based routing (same pattern as Python SDK)
- State management for retry scenarios
- Support for both sync and async tests

**Axum Mock Server Example:**
```rust
use axum::{Router, extract::State, routing::post, Json};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
struct MockServerState {
    request_count: Arc<Mutex<u32>>,
    rate_limit_count: Arc<Mutex<u32>>,
}

async fn mock_api_handler(
    State(state): State<MockServerState>,
    headers: HeaderMap,
    body: String,
) -> (StatusCode, Json<Value>) {
    let token = headers.get("authorization")
        .and_then(|v| v.to_str().ok())
        .unwrap_or("");

    // Token-based routing (like Python SDK)
    match token {
        "Bearer xoxb-ratelimited" => {
            let mut count = state.rate_limit_count.lock().unwrap();
            *count += 1;
            if *count <= 1 {
                return (StatusCode::TOO_MANY_REQUESTS, json!({"ok": false, "error": "ratelimited"}));
            }
        }
        "Bearer xoxb-api_test" => {
            return (StatusCode::OK, json!({"ok": true}));
        }
        _ => {}
    }

    (StatusCode::OK, json!({"ok": true}))
}

fn create_mock_server() -> Router {
    let state = MockServerState {
        request_count: Arc::new(Mutex::new(0)),
        rate_limit_count: Arc::new(Mutex::new(0)),
    };

    Router::new()
        .route("/api/:method", post(mock_api_handler))
        .with_state(state)
}
```

### Test Execution
```bash
# Run all tests
cargo test

# Run specific test module
cargo test web::client

# Run with output
cargo test -- --nocapture

# Run async tests
cargo test --features tokio-test
```

### Coverage Goals
- Minimum 80% code coverage
- 100% coverage for critical paths (auth, API calls, error handling)
- All 900+ migrated tests passing

---

## Logging Infrastructure

The Rust SDK will mirror the Python SDK's logging patterns using `tracing` for structured logging.

### Python SDK Logging Analysis

The Python SDK uses the standard `logging` module with these patterns:

1. **Logger per module:** Each client/module has its own logger
2. **Constructor injection:** `logger: Optional[logging.Logger] = None`
3. **Default logger:** `logging.getLogger(__name__)` if not provided
4. **Log levels:**
   - `DEBUG`: Request/response details, retry attempts
   - `INFO`: Retry notifications, connection events
   - `WARNING`: Auth issues, deprecated features
   - `ERROR`: Failed requests, decoding errors

**Python SDK Logging Examples:**
```python
# Constructor
def __init__(self, logger: Optional[logging.Logger] = None):
    self._logger = logger if logger is not None else logging.getLogger(__name__)

# Usage
if self._logger.level <= logging.DEBUG:
    self._logger.debug(f"Sending request: {method} {url}")

self._logger.info(f"Going to retry the same request: {req.method} {req.full_url}")
self._logger.error(f"Failed to send a request to Slack API server: {err}")
```

### Rust Logging Implementation

Use `tracing` for structured, async-aware logging:

**Core Logging Setup:**
```rust
// src/logging.rs

use tracing::{debug, info, warn, error, instrument};
use tracing_subscriber::{EnvFilter, fmt, prelude::*};

/// Initialize the global tracing subscriber
/// Call once at application startup
pub fn init_logging() {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::from_default_env()
            .add_directive("slack_rs=info".parse().unwrap()))
        .init();
}

/// Initialize with custom filter
pub fn init_logging_with_filter(filter: &str) {
    tracing_subscriber::registry()
        .with(fmt::layer())
        .with(EnvFilter::new(filter))
        .init();
}
```

**Client-Level Logging:**
```rust
// src/web/client.rs

use tracing::{debug, info, warn, error, instrument, Span};

pub struct WebClient {
    token: String,
    base_url: String,
    // Store a span for this client instance
    span: Span,
}

impl WebClient {
    pub fn new(token: impl Into<String>) -> Result<Self, SlackError> {
        let token = token.into();

        // Create a span for this client (mimics Python's logger per instance)
        let span = tracing::info_span!(
            "slack_web_client",
            client_id = %uuid::Uuid::new_v4(),
        );

        Ok(Self {
            token,
            base_url: "https://slack.com/api/".to_string(),
            span,
        })
    }

    /// Make an API call with logging
    #[instrument(skip(self, params), fields(method = %method))]
    pub async fn api_call(
        &self,
        method: &str,
        params: Option<Value>,
    ) -> Result<SlackResponse, SlackError> {
        let _enter = self.span.enter();

        debug!(
            method = method,
            url = %format!("{}{}", self.base_url, method),
            "Sending API request"
        );

        match self.execute_request(method, params).await {
            Ok(response) => {
                debug!(
                    method = method,
                    status = response.status_code,
                    "Received API response"
                );
                Ok(response)
            }
            Err(e) => {
                error!(
                    method = method,
                    error = %e,
                    "API request failed"
                );
                Err(e)
            }
        }
    }
}
```

**Retry Handler Logging:**
```rust
// src/http_retry/handler.rs

use tracing::{info, warn, debug};

impl RateLimitHandler {
    pub fn can_retry(&self, state: &RetryState, response: &HttpResponse) -> bool {
        if response.status_code == 429 {
            let retry_after = response.headers
                .get("Retry-After")
                .and_then(|v| v.parse::<u64>().ok())
                .unwrap_or(1);

            info!(
                attempt = state.current_attempt,
                retry_after_secs = retry_after,
                "Rate limited, will retry after delay"
            );

            true
        } else {
            false
        }
    }

    pub fn prepare_for_next_attempt(&mut self, state: &mut RetryState) {
        let interval = self.calculate_interval(state);

        info!(
            attempt = state.current_attempt + 1,
            wait_secs = interval,
            "Going to retry the same request"
        );

        state.next_attempt_requested_at = Instant::now() + Duration::from_secs(interval);
    }
}
```

**Error Logging:**
```rust
// src/error.rs

use tracing::error;

impl SlackError {
    /// Log the error with context
    pub fn log(&self) {
        match self {
            SlackError::Api(api_error) => {
                error!(
                    error_code = api_error.error,
                    status_code = api_error.status_code,
                    "Slack API returned error"
                );
            }
            SlackError::Http(http_error) => {
                error!(
                    error = %http_error,
                    "HTTP request failed"
                );
            }
            _ => {
                error!(error = %self, "Slack SDK error");
            }
        }
    }
}
```

**Usage in Applications:**
```rust
use slack_rs::{WebClient, logging::init_logging};
use tracing::info;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging (reads RUST_LOG env var)
    init_logging();

    info!("Starting Slack bot");

    let client = WebClient::new(std::env::var("SLACK_BOT_TOKEN")?)?;

    // All API calls will be logged automatically
    let response = client
        .chat_post_message()
        .channel("C1234567890")
        .text("Hello!")
        .send()
        .await?;

    info!(message_ts = %response.ts, "Message sent successfully");

    Ok(())
}
```

**Environment Configuration:**
```bash
# Set log level for entire application
export RUST_LOG=info

# Set log level for slack_rs only
export RUST_LOG=slack_rs=debug

# Set specific module log levels
export RUST_LOG=slack_rs::web=debug,slack_rs::http_retry=info

# Enable JSON logging for production
export RUST_LOG_FORMAT=json
```

**Log Output Examples:**

```
# INFO level (default)
2025-11-16T10:30:45.123Z  INFO slack_rs::web::client: Sending API request method="chat.postMessage"
2025-11-16T10:30:45.456Z  INFO slack_rs::web::client: Received API response method="chat.postMessage" status=200

# DEBUG level (detailed)
2025-11-16T10:30:45.123Z DEBUG slack_rs::web::client: Sending API request method="chat.postMessage" url="https://slack.com/api/chat.postMessage"
2025-11-16T10:30:45.234Z DEBUG slack_rs::http_retry: Request attempt attempt=1 method="POST"
2025-11-16T10:30:45.456Z DEBUG slack_rs::web::client: Received API response method="chat.postMessage" status=200 body_size=1234

# ERROR level
2025-11-16T10:30:45.789Z ERROR slack_rs::web::client: API request failed method="chat.postMessage" error="channel_not_found"
```

**Testing with Logging:**
```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tracing_subscriber::fmt::format::FmtSpan;

    #[test]
    fn test_with_logging() {
        // Initialize test logging
        let _ = tracing_subscriber::fmt()
            .with_test_writer()
            .with_max_level(tracing::Level::DEBUG)
            .with_span_events(FmtSpan::CLOSE)
            .try_init();

        // Test code - logs will appear in test output with --nocapture
    }
}
```

### Logging Guidelines

1. **Always use structured fields:** `info!(user_id = %id, "Event")` not `info!("Event for {}", id)`
2. **Use appropriate levels:**
   - `error!`: Failures that prevent operation
   - `warn!`: Unexpected but recoverable situations
   - `info!`: Important state changes, retry attempts
   - `debug!`: Request/response details, internal state
   - `trace!`: Very verbose debugging
3. **Include context:** method names, status codes, error types
4. **Span instrumentation:** Use `#[instrument]` on public methods
5. **Avoid logging sensitive data:** Never log full tokens (mask them: `xoxb-***`)

---

## API Design Principles

### Rust Idioms
- Use `Result<T, E>` for error handling (no exceptions)
- Builder pattern for complex types
- Iterator traits for pagination
- Async/await for async operations
- Strong type safety (leverage Rust's type system)

### Naming Conventions
- Follow Rust naming conventions (snake_case for functions/modules)
- Keep Python SDK naming where sensible
- Use Rust terminology (e.g., `Error` instead of `Exception`)

### Error Handling
```rust
#[derive(Debug, thiserror::Error)]
pub enum SlackError {
    #[error("API error: {0}")]
    Api(SlackApiError),

    #[error("HTTP error: {0}")]
    Http(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("Invalid signature")]
    InvalidSignature,
}
```

### Example API Usage
```rust
use slack_rs::{WebClient, models::blocks::SectionBlock};

#[tokio::main]
async fn main() -> Result<(), slack_rs::Error> {
    let client = WebClient::new("xoxb-token")?;

    let response = client
        .chat_post_message()
        .channel("C1234567890")
        .text("Hello from Rust!")
        .blocks(vec![
            SectionBlock::builder()
                .text("Hello *world*")
                .build()?
        ])
        .send()
        .await?;

    println!("Message sent: {}", response.ts);
    Ok(())
}
```

---

## Performance Considerations

### Optimization Opportunities
- Zero-copy JSON parsing where possible
- Connection pooling in HTTP client
- Async I/O for all network operations
- Lazy evaluation for pagination
- Compile-time validation of Block Kit structures

### Benchmarking
- Block Kit serialization speed
- HTTP client throughput
- WebSocket message processing
- OAuth flow performance

---

## Compatibility & Migration

### Python SDK Compatibility
- Maintain similar API structure where Rust idioms allow
- Provide migration guide for Python users
- Document differences in behavior

### Versioning Strategy
- Semantic versioning (0.1.0 � 1.0.0)
- Beta releases during development
- Stable 1.0.0 after all tests pass and community feedback

---

## Risk Assessment

### High Risk Areas
1. **API Coverage** - 260+ methods to implement correctly
2. **Block Kit Complexity** - Large type system with validation
3. **Async Testing** - Ensuring robust async behavior
4. **OAuth Storage** - Multiple backends to maintain
5. **WebSocket Stability** - Reconnection logic and state management

### Mitigation Strategies
1. TDD approach ensures correctness
2. Code generation for repetitive API methods
3. Comprehensive integration testing
4. Community beta testing before 1.0
5. Extensive documentation and examples

---

## Success Criteria

### Functional Requirements
- [ ] All 900+ migrated tests passing
- [ ] All 260+ Web API methods implemented
- [ ] Complete Block Kit model coverage
- [ ] OAuth flow with multiple storage backends
- [ ] Socket Mode with auto-reconnection
- [ ] Signature verification
- [ ] HTTP retry with exponential backoff

### Non-Functional Requirements
- [ ] 80%+ code coverage
- [ ] Comprehensive documentation
- [ ] Performance benchmarks
- [ ] Security audit passed
- [ ] Published to crates.io
- [ ] CI/CD pipeline operational

### Community Goals
- [ ] 10+ example applications
- [ ] Active community contributions
- [ ] Used in production by early adopters
- [ ] Featured in Slack API documentation

---

## Timeline Summary

| Phase | Duration | Description |
|-------|----------|-------------|
| 0. Setup | Completed | Project initialization |
| 1. HTTP Retry | 2 weeks | Retry logic foundation |
| 2. Block Kit | 3 weeks | All UI models |
| 3. Signature | 1 week | Request verification |
| 4. Webhooks | 1 week | Webhook client |
| 5. Web API Core | 4 weeks | Essential API methods |
| 6. Web API Complete | 3 weeks | All 260+ methods |
| 7. OAuth Core | 3 weeks | OAuth with file/SQLite |
| 8. OAuth Backends | 1 week | Additional storage |
| 9. Socket Mode | 3 weeks | WebSocket client |
| 10. SCIM | 1 week | Enterprise provisioning |
| 11. Audit Logs | 1 week | Enterprise audit logs |
| 12. RTM | Skipped | Deprecated API |
| 13. Polish | 2 weeks | Docs and examples |
| 14. Integration | 2 weeks | Final validation |
| **Total** | **27 weeks** | **~6.75 months** |

---

## References

- [Python Slack SDK Repository](https://github.com/slackapi/python-slack-sdk)
- [Slack API Documentation](https://api.slack.com/)
- [Block Kit Documentation](https://api.slack.com/block-kit)
- [Slack API Methods](https://api.slack.com/methods)
- [OAuth Guide](https://api.slack.com/authentication/oauth-v2)
- [Socket Mode Guide](https://api.slack.com/apis/connections/socket)

---

## CI/CD Configuration

### GitHub Actions Workflow

Create `.github/workflows/ci.yml`:

```yaml
name: CI

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main, develop]

env:
  RUST_BACKTRACE: 1
  CARGO_TERM_COLOR: always

jobs:
  # Formatting check
  fmt:
    name: Format Check
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: rustfmt
      - name: Check formatting
        run: cargo fmt --all -- --check

  # Linting with Clippy
  clippy:
    name: Clippy Lints
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy
      - uses: Swatinem/rust-cache@v2
      - name: Run Clippy
        run: cargo clippy --all-targets --all-features -- -D warnings

  # Build and test on multiple platforms
  test:
    name: Test Suite
    runs-on: ${{ matrix.os }}
    strategy:
      matrix:
        os: [ubuntu-latest, windows-latest, macos-latest]
        rust: [stable, beta]
        exclude:
          # Only test beta on Linux to save CI time
          - os: windows-latest
            rust: beta
          - os: macos-latest
            rust: beta
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@master
        with:
          toolchain: ${{ matrix.rust }}
      - uses: Swatinem/rust-cache@v2
      - name: Build
        run: cargo build --all-features --verbose
      - name: Run tests
        run: cargo test --all-features --workspace --verbose

  # Test with minimum supported Rust version
  msrv:
    name: MSRV Check (Rust 1.75)
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@1.75
      - uses: Swatinem/rust-cache@v2
      - name: Check MSRV
        run: cargo check --all-features

  # Code coverage
  coverage:
    name: Code Coverage
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: llvm-tools-preview
      - uses: Swatinem/rust-cache@v2
      - name: Install cargo-llvm-cov
        uses: taiki-e/install-action@cargo-llvm-cov
      - name: Generate coverage
        run: cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
      - name: Upload coverage to Codecov
        uses: codecov/codecov-action@v4
        with:
          files: lcov.info
          fail_ci_if_error: true

  # Security audit
  security:
    name: Security Audit
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: rustsec/audit-check@v1
        with:
          token: ${{ secrets.GITHUB_TOKEN }}

  # Documentation build
  docs:
    name: Documentation
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - name: Build documentation
        run: cargo doc --all-features --no-deps --document-private-items
        env:
          RUSTDOCFLAGS: -D warnings

  # Benchmarks (only on main branch)
  bench:
    name: Benchmarks
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2
      - name: Run benchmarks
        run: cargo bench --all-features -- --output-format bencher | tee output.txt

  # All checks must pass
  ci-success:
    name: CI Success
    needs: [fmt, clippy, test, msrv, coverage, security, docs]
    runs-on: ubuntu-latest
    steps:
      - name: Mark CI as successful
        run: echo "All CI checks passed!"
```

### Release Workflow

Create `.github/workflows/release.yml`:

```yaml
name: Release

on:
  push:
    tags:
      - 'v*'

jobs:
  release:
    name: Create Release
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: Swatinem/rust-cache@v2

      - name: Run tests
        run: cargo test --all-features --workspace --release

      - name: Publish to crates.io
        run: cargo publish --token ${{ secrets.CRATES_IO_TOKEN }}

      - name: Create GitHub Release
        uses: softprops/action-gh-release@v1
        with:
          generate_release_notes: true
          draft: false
          prerelease: ${{ contains(github.ref, 'beta') || contains(github.ref, 'alpha') }}
```

### Dependabot Configuration

Create `.github/dependabot.yml`:

```yaml
version: 2
updates:
  - package-ecosystem: "cargo"
    directory: "/"
    schedule:
      interval: "weekly"
    open-pull-requests-limit: 10
    reviewers:
      - "maintainer-team"
    labels:
      - "dependencies"
      - "rust"
```

### Pre-commit Hooks

Create `.git/hooks/pre-commit` (optional for local development):

```bash
#!/bin/bash
set -e

echo "Running pre-commit checks..."

# Format check
echo "Checking formatting..."
cargo fmt -- --check

# Clippy
echo "Running clippy..."
cargo clippy --all-targets --all-features -- -D warnings

# Tests
echo "Running tests..."
cargo test --all-features --workspace

echo "All checks passed!"
```

### Development Workflow

```bash
# Before committing
cargo fmt                                                    # Format code
cargo clippy --all-targets --all-features -- -D warnings   # Lint
cargo test --all-features --workspace                       # Test

# Full CI check locally
cargo fmt -- --check && \
  cargo clippy --all-targets --all-features -- -D warnings && \
  cargo test --all-features --workspace && \
  cargo doc --all-features --no-deps

# Generate coverage report locally
cargo install cargo-llvm-cov
cargo llvm-cov --all-features --workspace --html
open target/llvm-cov/html/index.html

# Security audit
cargo install cargo-audit
cargo audit

# Check for outdated dependencies
cargo install cargo-outdated
cargo outdated

# Update dependencies
cargo update
```

---

## Notes

- This plan assumes a single developer working full-time
- Adjust timeline based on team size and availability
- Prioritize phases based on user needs
- Consider early releases of core functionality (Web API + Block Kit)
- Maintain backward compatibility within major versions
- Engage with Slack developer community for feedback
- **All code must pass:** `cargo fmt --check`, `cargo clippy -- -D warnings`, and `cargo test`
- **Minimum 80% code coverage** enforced by CI
- **100% coverage required** for public APIs, error handling, and validation logic
- **Unit tests required** for ALL modules (see Testing Strategy section)
- **Security audits** run on every PR
- **MSRV policy:** Rust 1.75+ (update as needed for dependencies)
- **Git commits:** Do NOT mention Claude or AI tools in commit messages

---

**Last Updated:** 2025-11-16
**Python SDK Version:** 3.38.0
**Target Rust Edition:** 2021
**Minimum Supported Rust Version (MSRV):** 1.75+
**Code Quality:** All code must be formatted (rustfmt) and linted (clippy) with zero warnings
