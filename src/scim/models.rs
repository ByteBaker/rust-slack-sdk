//! SCIM 2.0 data models.
//!
//! This module provides data models for SCIM (System for Cross-domain Identity Management)
//! 2.0 protocol, which is used for user and group provisioning in Slack.

use serde::{Deserialize, Serialize};

/// SCIM User representation.
///
/// Represents a user in the SCIM 2.0 format for Slack.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct User {
    /// SCIM schemas
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schemas: Option<Vec<String>>,

    /// User ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// External ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub external_id: Option<String>,

    /// Metadata about the user
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<UserMeta>,

    /// User name (typically email or username)
    #[serde(rename = "userName", skip_serializing_if = "Option::is_none")]
    pub user_name: Option<String>,

    /// Nick name
    #[serde(rename = "nickName", skip_serializing_if = "Option::is_none")]
    pub nick_name: Option<String>,

    /// User's name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<UserName>,

    /// Display name
    #[serde(rename = "displayName", skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Profile URL
    #[serde(rename = "profileUrl", skip_serializing_if = "Option::is_none")]
    pub profile_url: Option<String>,

    /// Job title
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Timezone
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timezone: Option<String>,

    /// Whether the user is active
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active: Option<bool>,

    /// Email addresses
    #[serde(skip_serializing_if = "Option::is_none")]
    pub emails: Option<Vec<UserEmail>>,

    /// Phone numbers
    #[serde(rename = "phoneNumbers", skip_serializing_if = "Option::is_none")]
    pub phone_numbers: Option<Vec<UserPhoneNumber>>,

    /// Photos
    #[serde(skip_serializing_if = "Option::is_none")]
    pub photos: Option<Vec<UserPhoto>>,

    /// Addresses
    #[serde(skip_serializing_if = "Option::is_none")]
    pub addresses: Option<Vec<UserAddress>>,

    /// Groups
    #[serde(skip_serializing_if = "Option::is_none")]
    pub groups: Option<Vec<UserGroup>>,

    /// Roles
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<UserRole>>,
}

impl User {
    /// Creates a new User with minimal required fields.
    pub fn new() -> Self {
        Self {
            schemas: Some(vec!["urn:scim:schemas:core:1.0".to_string()]),
            id: None,
            external_id: None,
            meta: None,
            user_name: None,
            nick_name: None,
            name: None,
            display_name: None,
            profile_url: None,
            title: None,
            timezone: None,
            active: None,
            emails: None,
            phone_numbers: None,
            photos: None,
            addresses: None,
            groups: None,
            roles: None,
        }
    }

    /// Sets the user name.
    pub fn with_user_name(mut self, user_name: impl Into<String>) -> Self {
        self.user_name = Some(user_name.into());
        self
    }

    /// Sets the display name.
    pub fn with_display_name(mut self, display_name: impl Into<String>) -> Self {
        self.display_name = Some(display_name.into());
        self
    }

    /// Sets the active status.
    pub fn with_active(mut self, active: bool) -> Self {
        self.active = Some(active);
        self
    }

    /// Sets the email addresses.
    pub fn with_emails(mut self, emails: Vec<UserEmail>) -> Self {
        self.emails = Some(emails);
        self
    }
}

impl Default for User {
    fn default() -> Self {
        Self::new()
    }
}

/// User metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserMeta {
    /// Creation timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,

    /// Resource location
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
}

/// User name components.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserName {
    /// Family name (last name)
    #[serde(rename = "familyName", skip_serializing_if = "Option::is_none")]
    pub family_name: Option<String>,

    /// Given name (first name)
    #[serde(rename = "givenName", skip_serializing_if = "Option::is_none")]
    pub given_name: Option<String>,
}

/// User email address.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserEmail {
    /// Email value
    pub value: String,

    /// Email type (e.g., "work", "home")
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub email_type: Option<String>,

    /// Whether this is the primary email
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary: Option<bool>,
}

/// User phone number.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserPhoneNumber {
    /// Phone number value
    pub value: String,

    /// Phone type (e.g., "work", "mobile")
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub phone_type: Option<String>,

    /// Whether this is the primary phone
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary: Option<bool>,
}

/// User photo.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserPhoto {
    /// Photo URL value
    pub value: String,

    /// Photo type
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub photo_type: Option<String>,
}

/// User address.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserAddress {
    /// Street address
    #[serde(rename = "streetAddress", skip_serializing_if = "Option::is_none")]
    pub street_address: Option<String>,

    /// Locality (city)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub locality: Option<String>,

    /// Region (state/province)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub region: Option<String>,

    /// Postal code
    #[serde(rename = "postalCode", skip_serializing_if = "Option::is_none")]
    pub postal_code: Option<String>,

    /// Country
    #[serde(skip_serializing_if = "Option::is_none")]
    pub country: Option<String>,

    /// Whether this is the primary address
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary: Option<bool>,
}

/// User group membership.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserGroup {
    /// Group value (ID)
    pub value: String,

    /// Group display name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<String>,
}

/// User role.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct UserRole {
    /// Role value
    pub value: String,

    /// Role type
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub role_type: Option<String>,

    /// Whether this is the primary role
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary: Option<bool>,
}

/// SCIM Group representation.
///
/// Represents a group in the SCIM 2.0 format for Slack.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Group {
    /// SCIM schemas
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schemas: Option<Vec<String>>,

    /// Group ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,

    /// Metadata about the group
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<GroupMeta>,

    /// Display name
    #[serde(rename = "displayName", skip_serializing_if = "Option::is_none")]
    pub display_name: Option<String>,

    /// Group members
    #[serde(skip_serializing_if = "Option::is_none")]
    pub members: Option<Vec<GroupMember>>,
}

impl Group {
    /// Creates a new Group.
    pub fn new() -> Self {
        Self {
            schemas: Some(vec!["urn:scim:schemas:core:1.0".to_string()]),
            id: None,
            meta: None,
            display_name: None,
            members: None,
        }
    }

    /// Sets the display name.
    pub fn with_display_name(mut self, display_name: impl Into<String>) -> Self {
        self.display_name = Some(display_name.into());
        self
    }

    /// Sets the members.
    pub fn with_members(mut self, members: Vec<GroupMember>) -> Self {
        self.members = Some(members);
        self
    }
}

impl Default for Group {
    fn default() -> Self {
        Self::new()
    }
}

/// Group metadata.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GroupMeta {
    /// Creation timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,

    /// Resource location
    #[serde(skip_serializing_if = "Option::is_none")]
    pub location: Option<String>,
}

/// Group member.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GroupMember {
    /// Member value (user ID)
    pub value: String,

    /// Member display name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub display: Option<String>,
}

/// SCIM response with pagination support.
///
/// Used for list operations that may return multiple pages of results.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScimResponse<T> {
    /// SCIM schemas
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schemas: Option<Vec<String>>,

    /// Total results available
    #[serde(rename = "totalResults", skip_serializing_if = "Option::is_none")]
    pub total_results: Option<u32>,

    /// Starting index (1-based)
    #[serde(rename = "startIndex", skip_serializing_if = "Option::is_none")]
    pub start_index: Option<u32>,

    /// Items per page
    #[serde(rename = "itemsPerPage", skip_serializing_if = "Option::is_none")]
    pub items_per_page: Option<u32>,

    /// Resources in this page
    #[serde(rename = "Resources", skip_serializing_if = "Option::is_none")]
    pub resources: Option<Vec<T>>,
}

/// SCIM error response.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ScimError {
    /// SCIM schemas
    #[serde(skip_serializing_if = "Option::is_none")]
    pub schemas: Option<Vec<String>>,

    /// Error detail message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,

    /// HTTP status code
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

/// SCIM patch operation.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PatchOperation {
    /// Operation type (add, remove, replace)
    pub op: String,

    /// Path to the attribute
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,

    /// Value to set
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>,
}

/// SCIM patch request.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PatchRequest {
    /// SCIM schemas
    pub schemas: Vec<String>,

    /// Operations to perform
    #[serde(rename = "Operations")]
    pub operations: Vec<PatchOperation>,
}

impl PatchRequest {
    /// Creates a new patch request.
    pub fn new(operations: Vec<PatchOperation>) -> Self {
        Self {
            schemas: vec!["urn:ietf:params:scim:api:messages:2.0:PatchOp".to_string()],
            operations,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_user_new() {
        let user = User::new();
        assert!(user.schemas.is_some());
        assert!(user.id.is_none());
    }

    #[test]
    fn test_user_builder() {
        let user = User::new()
            .with_user_name("test@example.com")
            .with_display_name("Test User")
            .with_active(true);

        assert_eq!(user.user_name, Some("test@example.com".to_string()));
        assert_eq!(user.display_name, Some("Test User".to_string()));
        assert_eq!(user.active, Some(true));
    }

    #[test]
    fn test_user_serialization() {
        let user = User::new()
            .with_user_name("john@example.com")
            .with_display_name("John Doe");

        let json = serde_json::to_string(&user).unwrap();
        let deserialized: User = serde_json::from_str(&json).unwrap();

        assert_eq!(user, deserialized);
    }

    #[test]
    fn test_user_email() {
        let email = UserEmail {
            value: "test@example.com".to_string(),
            email_type: Some("work".to_string()),
            primary: Some(true),
        };

        assert_eq!(email.value, "test@example.com");
        assert_eq!(email.email_type, Some("work".to_string()));
        assert_eq!(email.primary, Some(true));
    }

    #[test]
    fn test_group_new() {
        let group = Group::new();
        assert!(group.schemas.is_some());
        assert!(group.id.is_none());
    }

    #[test]
    fn test_group_builder() {
        let members = vec![GroupMember {
            value: "U123".to_string(),
            display: Some("John Doe".to_string()),
        }];

        let group = Group::new()
            .with_display_name("Engineers")
            .with_members(members);

        assert_eq!(group.display_name, Some("Engineers".to_string()));
        assert!(group.members.is_some());
        assert_eq!(group.members.as_ref().unwrap().len(), 1);
    }

    #[test]
    fn test_group_serialization() {
        let group = Group::new().with_display_name("Test Group");

        let json = serde_json::to_string(&group).unwrap();
        let deserialized: Group = serde_json::from_str(&json).unwrap();

        assert_eq!(group, deserialized);
    }

    #[test]
    fn test_scim_response() {
        let response: ScimResponse<User> = ScimResponse {
            schemas: Some(vec![
                "urn:ietf:params:scim:api:messages:2.0:ListResponse".to_string()
            ]),
            total_results: Some(10),
            start_index: Some(1),
            items_per_page: Some(5),
            resources: Some(vec![User::new()]),
        };

        assert_eq!(response.total_results, Some(10));
        assert_eq!(response.items_per_page, Some(5));
    }

    #[test]
    fn test_patch_operation() {
        let op = PatchOperation {
            op: "replace".to_string(),
            path: Some("active".to_string()),
            value: Some(json!(false)),
        };

        assert_eq!(op.op, "replace");
        assert_eq!(op.path, Some("active".to_string()));
    }

    #[test]
    fn test_patch_request() {
        let operations = vec![PatchOperation {
            op: "replace".to_string(),
            path: Some("displayName".to_string()),
            value: Some(json!("New Name")),
        }];

        let request = PatchRequest::new(operations);

        assert_eq!(request.operations.len(), 1);
        assert!(request
            .schemas
            .contains(&"urn:ietf:params:scim:api:messages:2.0:PatchOp".to_string()));
    }
}
