//! Embedded canonical specification registry.

use crate::error::ConformanceError;
use crate::model::{ChecklistTable, ProfileKind, SpecId};
use crate::parser::markdown::parse_checklist;
use std::path::PathBuf;

// Embedded canonical checklist files:

const OPENAPI_3_2_0_CLIENT_SDK: &str = include_str!("../openapi-3.2.0/client-sdk.md");
const OPENAPI_3_2_0_CLI: &str = include_str!("../openapi-3.2.0/client-sdk-cli.md");
const OPENAPI_3_2_0_SERVERS: &str = include_str!("../openapi-3.2.0/servers.md");

const OPENAPI_3_1_1_CLIENT_SDK: &str = include_str!("../openapi-3.1.1/client-sdk.md");
const OPENAPI_3_1_1_CLI: &str = include_str!("../openapi-3.1.1/client-sdk-cli.md");
const OPENAPI_3_1_1_SERVERS: &str = include_str!("../openapi-3.1.1/servers.md");

const OPENAPI_3_1_0_CLIENT_SDK: &str = include_str!("../openapi-3.1.0/client-sdk.md");
const OPENAPI_3_1_0_CLI: &str = include_str!("../openapi-3.1.0/client-sdk-cli.md");
const OPENAPI_3_1_0_SERVERS: &str = include_str!("../openapi-3.1.0/servers.md");

const OPENAPI_3_0_0_CLIENT_SDK: &str = include_str!("../openapi-3.0.0/client-sdk.md");
const OPENAPI_3_0_0_CLI: &str = include_str!("../openapi-3.0.0/client-sdk-cli.md");
const OPENAPI_3_0_0_SERVERS: &str = include_str!("../openapi-3.0.0/servers.md");

const SWAGGER_2_0_CLIENT_SDK: &str = include_str!("../swagger-2.0/client-sdk.md");
const SWAGGER_2_0_CLI: &str = include_str!("../swagger-2.0/client-sdk-cli.md");
const SWAGGER_2_0_SERVERS: &str = include_str!("../swagger-2.0/servers.md");

const ARAZZO_1_1_0_CLIENT_SDK: &str = include_str!("../arazzo-1.1.0/client-sdk.md");
const ARAZZO_1_1_0_CLI: &str = include_str!("../arazzo-1.1.0/client-sdk-cli.md");
const ARAZZO_1_1_0_SERVERS: &str = include_str!("../arazzo-1.1.0/servers.md");

const MCP_1_0_0_CLIENT_SDK: &str = include_str!("../mcp-1.0.0/client-sdk.md");
const MCP_1_0_0_CLI: &str = include_str!("../mcp-1.0.0/client-sdk-cli.md");
const MCP_1_0_0_SERVERS: &str = include_str!("../mcp-1.0.0/servers.md");

const MCP_2026_07_28_CLIENT_SDK: &str = include_str!("../mcp-2026-07-28/client-sdk.md");
const MCP_2026_07_28_CLI: &str = include_str!("../mcp-2026-07-28/client-sdk-cli.md");
const MCP_2026_07_28_SERVERS: &str = include_str!("../mcp-2026-07-28/servers.md");

const MOCK_SERVER_PLAN: &str = include_str!("../mock-server/mock-server-plan.md");

/// Canonical specification registry providing reference checklists.
pub struct CanonicalRegistry {
    custom_root: Option<PathBuf>,
}

impl CanonicalRegistry {
    /// Creates a new `CanonicalRegistry` utilizing embedded canonical specifications.
    #[must_use]
    pub const fn new() -> Self {
        Self { custom_root: None }
    }

    /// Creates a new `CanonicalRegistry` with an explicit directory root.
    #[must_use]
    pub fn with_root(root: impl Into<PathBuf>) -> Self {
        Self {
            custom_root: Some(root.into()),
        }
    }

    /// Returns the raw markdown string for a canonical specification checklist.
    ///
    /// # Errors
    ///
    /// Returns `ConformanceError::Io` if an explicit root is configured and file read fails,
    /// or `ConformanceError::MissingSpecDirectory` if the specification or profile is not found.
    pub fn get_raw_content(
        &self,
        spec_id: SpecId,
        profile: ProfileKind,
    ) -> Result<String, ConformanceError> {
        if let Some(ref root) = self.custom_root {
            let spec_dir = root.join(spec_id.as_str());
            let file_path = spec_dir.join(profile.filename());
            if file_path.is_file() {
                return std::fs::read_to_string(&file_path).map_err(|e| ConformanceError::Io {
                    source: e,
                    path: file_path,
                });
            }
            return Err(ConformanceError::MissingSpecDirectory {
                expected_dir: spec_dir,
            });
        }

        let content = match (spec_id, profile) {
            (SpecId::OpenApi320, ProfileKind::ClientSdk) => OPENAPI_3_2_0_CLIENT_SDK,
            (SpecId::OpenApi320, ProfileKind::ClientCli) => OPENAPI_3_2_0_CLI,
            (SpecId::OpenApi320, ProfileKind::Servers) => OPENAPI_3_2_0_SERVERS,

            (SpecId::OpenApi311, ProfileKind::ClientSdk) => OPENAPI_3_1_1_CLIENT_SDK,
            (SpecId::OpenApi311, ProfileKind::ClientCli) => OPENAPI_3_1_1_CLI,
            (SpecId::OpenApi311, ProfileKind::Servers) => OPENAPI_3_1_1_SERVERS,

            (SpecId::OpenApi310, ProfileKind::ClientSdk) => OPENAPI_3_1_0_CLIENT_SDK,
            (SpecId::OpenApi310, ProfileKind::ClientCli) => OPENAPI_3_1_0_CLI,
            (SpecId::OpenApi310, ProfileKind::Servers) => OPENAPI_3_1_0_SERVERS,

            (SpecId::OpenApi300, ProfileKind::ClientSdk) => OPENAPI_3_0_0_CLIENT_SDK,
            (SpecId::OpenApi300, ProfileKind::ClientCli) => OPENAPI_3_0_0_CLI,
            (SpecId::OpenApi300, ProfileKind::Servers) => OPENAPI_3_0_0_SERVERS,

            (SpecId::Swagger20, ProfileKind::ClientSdk) => SWAGGER_2_0_CLIENT_SDK,
            (SpecId::Swagger20, ProfileKind::ClientCli) => SWAGGER_2_0_CLI,
            (SpecId::Swagger20, ProfileKind::Servers) => SWAGGER_2_0_SERVERS,

            (SpecId::Arazzo110, ProfileKind::ClientSdk) => ARAZZO_1_1_0_CLIENT_SDK,
            (SpecId::Arazzo110, ProfileKind::ClientCli) => ARAZZO_1_1_0_CLI,
            (SpecId::Arazzo110, ProfileKind::Servers) => ARAZZO_1_1_0_SERVERS,

            (SpecId::Mcp100, ProfileKind::ClientSdk) => MCP_1_0_0_CLIENT_SDK,
            (SpecId::Mcp100, ProfileKind::ClientCli) => MCP_1_0_0_CLI,
            (SpecId::Mcp100, ProfileKind::Servers) => MCP_1_0_0_SERVERS,

            (SpecId::Mcp20260728, ProfileKind::ClientSdk) => MCP_2026_07_28_CLIENT_SDK,
            (SpecId::Mcp20260728, ProfileKind::ClientCli) => MCP_2026_07_28_CLI,
            (SpecId::Mcp20260728, ProfileKind::Servers) => MCP_2026_07_28_SERVERS,

            (SpecId::MockServer, ProfileKind::MockServerPlan) => MOCK_SERVER_PLAN,

            _ => {
                return Err(ConformanceError::MissingSpecDirectory {
                    expected_dir: PathBuf::from(format!(
                        "{}/{}",
                        spec_id.as_str(),
                        profile.filename()
                    )),
                });
            }
        };

        Ok(content.to_string())
    }

    /// Loads and parses the canonical checklist table for the given specification and profile.
    ///
    /// # Errors
    ///
    /// Returns `ConformanceError` if retrieval or parsing fails.
    pub fn get_table(
        &self,
        spec_id: SpecId,
        profile: ProfileKind,
    ) -> Result<ChecklistTable, ConformanceError> {
        let raw = self.get_raw_content(spec_id, profile)?;
        let path = PathBuf::from(format!("{}/{}", spec_id.as_str(), profile.filename()));
        parse_checklist(&raw, &path, spec_id, profile)
    }

    /// Returns a slice of all supported specifications.
    #[must_use]
    pub const fn list_specs(&self) -> &'static [SpecId] {
        SpecId::all()
    }

    /// Returns the profiles supported by a given specification.
    #[must_use]
    pub fn list_profiles(&self, spec_id: SpecId) -> Vec<ProfileKind> {
        if spec_id == SpecId::MockServer {
            vec![ProfileKind::MockServerPlan]
        } else {
            ProfileKind::standard_profiles().to_vec()
        }
    }
}

impl Default for CanonicalRegistry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::assert_is_empty, clippy::uninlined_format_args)]
mod tests {
    use super::*;

    #[test]
    fn test_all_embedded_canonical_tables_parse_successfully(
    ) -> Result<(), Box<dyn std::error::Error>> {
        let registry = CanonicalRegistry::default();
        for spec in registry.list_specs() {
            let profiles = registry.list_profiles(*spec);
            assert!(!profiles.is_empty());
            for profile in profiles {
                let table = registry.get_table(*spec, profile)?;
                assert!(
                    !table.rows.is_empty(),
                    "Canonical table for {} {} is empty",
                    spec,
                    profile
                );
            }
        }

        // Test unmapped spec / profile combination (MockServer with ClientSdk)
        assert!(registry
            .get_table(SpecId::MockServer, ProfileKind::ClientSdk)
            .is_err());
        Ok(())
    }

    #[test]
    fn test_registry_with_custom_root() -> Result<(), Box<dyn std::error::Error>> {
        let registry = CanonicalRegistry::with_root(".");
        let table = registry.get_table(SpecId::OpenApi320, ProfileKind::ClientSdk)?;
        assert!(!table.rows.is_empty());

        let bad_registry = CanonicalRegistry::with_root("nonexistent-dir-12345");
        assert!(bad_registry
            .get_table(SpecId::OpenApi320, ProfileKind::ClientSdk)
            .is_err());

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let dir = tempfile::tempdir()?;
            let spec_dir = dir.path().join("openapi-3.2.0");
            std::fs::create_dir_all(&spec_dir)?;
            let file_path = spec_dir.join("client-sdk.md");
            std::fs::write(&file_path, b"test")?;
            std::fs::set_permissions(&file_path, std::fs::Permissions::from_mode(0o000))?;

            let reg = CanonicalRegistry::with_root(dir.path());
            assert!(reg
                .get_raw_content(SpecId::OpenApi320, ProfileKind::ClientSdk)
                .is_err());

            std::fs::set_permissions(&file_path, std::fs::Permissions::from_mode(0o644))?;
        }
        Ok(())
    }
}
