//! Shared constants used throughout the Slack SDK.

/// HTTP header names
pub mod headers {
    /// Slack request timestamp header
    pub const SLACK_REQUEST_TIMESTAMP: &str = "x-slack-request-timestamp";

    /// Slack signature header
    pub const SLACK_SIGNATURE: &str = "x-slack-signature";

    /// User-Agent header
    pub const USER_AGENT: &str = "User-Agent";

    /// Content-Type header
    pub const CONTENT_TYPE: &str = "Content-Type";

    /// Authorization header
    pub const AUTHORIZATION: &str = "Authorization";

    /// Retry-After header (for rate limiting)
    pub const RETRY_AFTER: &str = "Retry-After";
}

/// HTTP status codes
pub mod status_codes {
    /// OK
    pub const OK: u16 = 200;

    /// Too Many Requests (rate limited)
    pub const TOO_MANY_REQUESTS: u16 = 429;

    /// Internal Server Error
    pub const INTERNAL_SERVER_ERROR: u16 = 500;

    /// Service Unavailable
    pub const SERVICE_UNAVAILABLE: u16 = 503;
}

/// Time-related constants
pub mod time {
    /// Maximum age of a request timestamp in seconds (5 minutes)
    pub const MAX_REQUEST_AGE_SECS: u64 = 300;

    /// Default timeout for HTTP requests in seconds
    pub const DEFAULT_TIMEOUT_SECS: u64 = 30;
}

/// Block Kit validation limits
pub mod limits {
    /// Maximum text length for most text objects (3000 characters)
    pub const MAX_TEXT_LENGTH: usize = 3000;

    /// Maximum length for option labels (75 characters)
    pub const MAX_OPTION_LABEL_LENGTH: usize = 75;

    /// Maximum length for option values (75 characters)
    pub const MAX_OPTION_VALUE_LENGTH: usize = 75;

    /// Maximum length for confirm dialog titles (100 characters)
    pub const MAX_CONFIRM_TITLE_LENGTH: usize = 100;

    /// Maximum length for confirm dialog text (300 characters)
    pub const MAX_CONFIRM_TEXT_LENGTH: usize = 300;

    /// Maximum text length for header blocks (150 characters)
    pub const MAX_HEADER_TEXT_LENGTH: usize = 150;

    /// Maximum number of fields in a section block
    pub const MAX_SECTION_FIELDS: usize = 10;

    /// Maximum number of elements in a context block
    pub const MAX_CONTEXT_ELEMENTS: usize = 10;

    /// Maximum number of elements in an actions block
    pub const MAX_ACTIONS_ELEMENTS: usize = 25;

    /// Maximum number of blocks in a view
    pub const MAX_VIEW_BLOCKS: usize = 100;

    /// Maximum length for view titles (24 characters)
    pub const MAX_VIEW_TITLE_LENGTH: usize = 24;

    /// Maximum length for view submit/close buttons (24 characters)
    pub const MAX_VIEW_BUTTON_LENGTH: usize = 24;

    /// Maximum length for callback IDs (255 characters)
    pub const MAX_CALLBACK_ID_LENGTH: usize = 255;

    /// Maximum length for private metadata (3000 characters)
    pub const MAX_PRIVATE_METADATA_LENGTH: usize = 3000;

    /// Maximum length for button text (75 characters)
    pub const MAX_BUTTON_TEXT_LENGTH: usize = 75;

    /// Maximum length for button values (2000 characters)
    pub const MAX_BUTTON_VALUE_LENGTH: usize = 2000;

    /// Maximum length for URLs (3000 characters)
    pub const MAX_URL_LENGTH: usize = 3000;

    /// Maximum length for alt text (2000 characters)
    pub const MAX_ALT_TEXT_LENGTH: usize = 2000;

    /// Maximum length for placeholder text (150 characters)
    pub const MAX_PLACEHOLDER_LENGTH: usize = 150;

    /// Maximum length for action IDs (255 characters)
    pub const MAX_ACTION_ID_LENGTH: usize = 255;

    /// Maximum length for hint text (2000 characters)
    pub const MAX_HINT_LENGTH: usize = 2000;

    /// Maximum number of options in a select menu
    pub const MAX_SELECT_OPTIONS: usize = 100;

    /// Maximum number of initial selected options
    pub const MAX_INITIAL_OPTIONS: usize = 100;

    /// Maximum number of options in overflow menu
    pub const MAX_OVERFLOW_OPTIONS: usize = 5;

    /// Minimum number of options in overflow menu
    pub const MIN_OVERFLOW_OPTIONS: usize = 2;

    /// Maximum number of options in checkboxes/radio buttons
    pub const MAX_CHOICE_OPTIONS: usize = 10;

    /// Maximum number of option groups
    pub const MAX_OPTION_GROUPS: usize = 100;
}

/// Signature verification constants
pub mod signature {
    /// Signature version prefix
    pub const SIGNATURE_VERSION: &str = "v0";

    /// Signature format prefix (includes version)
    pub const SIGNATURE_PREFIX: &str = "v0=";
}
