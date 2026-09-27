//! MCP resources engine exposing canonical specification checklists as read-only URIs.

use crate::error::ConformanceError;
use crate::model::{ProfileKind, SpecId};
use crate::registry::CanonicalRegistry;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Metadata for an MCP resource.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Resource {
    /// Canonical URI identifying the resource.
    pub uri: String,
    /// Human-readable title of the resource.
    pub name: String,
    /// Detailed description.
    pub description: Option<String>,
    /// MIME type of the payload (e.g. `text/markdown`, `application/json`).
    pub mime_type: Option<String>,
}

/// Dynamic URI template for resources.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceTemplate {
    /// URI template following RFC 6570.
    pub uri_template: String,
    /// Name of the template.
    pub name: String,
    /// Description of parameterized values.
    pub description: Option<String>,
    /// MIME type.
    pub mime_type: Option<String>,
}

/// Result returned in response to `resources/list`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ListResourcesResult {
    /// Available resources.
    pub resources: Vec<Resource>,
}

/// Result returned in response to `resources/templates/list`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListResourceTemplatesResult {
    /// Available resource templates.
    pub resource_templates: Vec<ResourceTemplate>,
}

/// Content payload returned by `resources/read`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceContents {
    /// URI of the read resource.
    pub uri: String,
    /// MIME type of the payload.
    pub mime_type: Option<String>,
    /// Text payload.
    pub text: Option<String>,
}

/// Result returned in response to `resources/read`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ReadResourceResult {
    /// Contents of the resource.
    pub contents: Vec<ResourceContents>,
}

/// Lists available static conformance resources.
#[must_use]
pub fn list_resources(registry: &CanonicalRegistry) -> ListResourcesResult {
    let mut resources = vec![Resource {
        uri: "conformance://specs".to_string(),
        name: "Canonical Specifications Catalog".to_string(),
        description: Some(
            "Catalog of all supported standards and specification versions.".to_string(),
        ),
        mime_type: Some("application/json".to_string()),
    }];

    for spec in registry.list_specs() {
        for profile in registry.list_profiles(*spec) {
            resources.push(Resource {
                uri: format!("conformance://specs/{}/{}", spec.as_str(), profile.as_str()),
                name: format!("Canonical {spec} ({profile}) Checklist"),
                description: Some(format!("Canonical reference table for {spec} {profile}.")),
                mime_type: Some("text/markdown".to_string()),
            });
        }
    }

    ListResourcesResult { resources }
}

/// Lists parameterized resource templates.
#[must_use]
pub fn list_resource_templates() -> ListResourceTemplatesResult {
    ListResourceTemplatesResult {
        resource_templates: vec![ResourceTemplate {
            uri_template: "conformance://specs/{spec_id}/{profile}".to_string(),
            name: "Canonical Checklist Table".to_string(),
            description: Some(
                "Reads the canonical reference table for any specification standard and profile."
                    .to_string(),
            ),
            mime_type: Some("text/markdown".to_string()),
        }],
    }
}

/// Reads the contents of a requested resource URI.
///
/// # Errors
///
/// Returns `ConformanceError` if the URI is not found or parameter parsing fails.
pub fn read_resource(
    uri: &str,
    registry: &CanonicalRegistry,
) -> Result<ReadResourceResult, ConformanceError> {
    if uri == "conformance://specs" {
        let specs: Vec<&'static str> = registry.list_specs().iter().map(|s| s.as_str()).collect();
        let json = serde_json::to_string_pretty(&specs).unwrap_or_default();
        return Ok(ReadResourceResult {
            contents: vec![ResourceContents {
                uri: uri.to_string(),
                mime_type: Some("application/json".to_string()),
                text: Some(json),
            }],
        });
    }

    if let Some(rest) = uri.strip_prefix("conformance://specs/") {
        let parts: Vec<&str> = rest.split('/').collect();
        if parts.len() == 2 {
            let spec_id = SpecId::from_str(parts[0])?;
            let profile = ProfileKind::from_str(parts[1])?;
            let content = registry.get_raw_content(spec_id, profile)?;

            return Ok(ReadResourceResult {
                contents: vec![ResourceContents {
                    uri: uri.to_string(),
                    mime_type: Some("text/markdown".to_string()),
                    text: Some(content),
                }],
            });
        }
    }

    Err(ConformanceError::JsonRpcInvalidParams {
        message: format!("Unknown resource URI: '{uri}'"),
    })
}

#[cfg(test)]
#[allow(clippy::assert_is_empty)]
mod tests {
    use super::*;

    #[test]
    fn test_list_and_read_resources() {
        let registry = CanonicalRegistry::new();
        let list = list_resources(&registry);
        assert!(!list.resources.is_empty());

        let templates = list_resource_templates();
        assert_eq!(templates.resource_templates.len(), 1);

        let catalog_res = read_resource("conformance://specs", &registry);
        assert!(catalog_res.is_ok());

        let spec_res = read_resource("conformance://specs/openapi-3.2.0/client-sdk", &registry);
        assert!(spec_res.is_ok());

        let bad_res = read_resource("conformance://invalid", &registry);
        assert!(bad_res.is_err());

        let bad_parts = read_resource("conformance://specs/one", &registry);
        assert!(bad_parts.is_err());

        let too_many_parts = read_resource("conformance://specs/a/b/c", &registry);
        assert!(too_many_parts.is_err());

        let invalid_spec =
            read_resource("conformance://specs/invalid-spec-id/client-sdk", &registry);
        assert!(invalid_spec.is_err());

        let invalid_profile = read_resource(
            "conformance://specs/openapi-3.2.0/invalid-profile",
            &registry,
        );
        assert!(invalid_profile.is_err());

        let unmapped_profile =
            read_resource("conformance://specs/mock-server/client-sdk", &registry);
        assert!(unmapped_profile.is_err());
    }
}
