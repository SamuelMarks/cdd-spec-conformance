//! Embedded canonical specification registry.

use crate::error::ConformanceError;
use crate::model::{ChecklistTable, ProfileKind, SpecId};
use crate::parser::markdown::parse_checklist;
use std::path::PathBuf;

/// Embedded canonical `OpenAPI` 3.2.0 Client SDK checklist.
const OAS_320_SDK: &str = include_str!("../openapi-3.2.0/client-sdk.md");
/// Embedded canonical `OpenAPI` 3.2.0 Client CLI checklist.
const OAS_320_CLI: &str = include_str!("../openapi-3.2.0/client-sdk-cli.md");
/// Embedded canonical `OpenAPI` 3.2.0 Servers checklist.
const OAS_320_SRV: &str = include_str!("../openapi-3.2.0/servers.md");

/// Embedded canonical `OpenAPI` 3.1.1 Client SDK checklist.
const OAS_311_SDK: &str = include_str!("../openapi-3.1.1/client-sdk.md");
/// Embedded canonical `OpenAPI` 3.1.1 Client CLI checklist.
const OAS_311_CLI: &str = include_str!("../openapi-3.1.1/client-sdk-cli.md");
/// Embedded canonical `OpenAPI` 3.1.1 Servers checklist.
const OAS_311_SRV: &str = include_str!("../openapi-3.1.1/servers.md");

/// Embedded canonical `OpenAPI` 3.1.0 Client SDK checklist.
const OAS_310_SDK: &str = include_str!("../openapi-3.1.0/client-sdk.md");
/// Embedded canonical `OpenAPI` 3.1.0 Client CLI checklist.
const OAS_310_CLI: &str = include_str!("../openapi-3.1.0/client-sdk-cli.md");
/// Embedded canonical `OpenAPI` 3.1.0 Servers checklist.
const OAS_310_SRV: &str = include_str!("../openapi-3.1.0/servers.md");

/// Embedded canonical `OpenAPI` 3.0.0 Client SDK checklist.
const OAS_300_SDK: &str = include_str!("../openapi-3.0.0/client-sdk.md");
/// Embedded canonical `OpenAPI` 3.0.0 Client CLI checklist.
const OAS_300_CLI: &str = include_str!("../openapi-3.0.0/client-sdk-cli.md");
/// Embedded canonical `OpenAPI` 3.0.0 Servers checklist.
const OAS_300_SRV: &str = include_str!("../openapi-3.0.0/servers.md");

/// Embedded canonical Swagger 2.0 Client SDK checklist.
const SWAGGER_20_SDK: &str = include_str!("../swagger-2.0/client-sdk.md");
/// Embedded canonical Swagger 2.0 Client CLI checklist.
const SWAGGER_20_CLI: &str = include_str!("../swagger-2.0/client-sdk-cli.md");
/// Embedded canonical Swagger 2.0 Servers checklist.
const SWAGGER_20_SRV: &str = include_str!("../swagger-2.0/servers.md");

/// Embedded canonical Arazzo 1.1.0 Client SDK checklist.
const ARAZZO_110_SDK: &str = include_str!("../arazzo-1.1.0/client-sdk.md");
/// Embedded canonical Arazzo 1.1.0 Client CLI checklist.
const ARAZZO_110_CLI: &str = include_str!("../arazzo-1.1.0/client-sdk-cli.md");
/// Embedded canonical Arazzo 1.1.0 Servers checklist.
const ARAZZO_110_SRV: &str = include_str!("../arazzo-1.1.0/servers.md");

/// Embedded canonical MCP 1.0.0 Client SDK checklist.
const MCP_100_SDK: &str = include_str!("../mcp-1.0.0/client-sdk.md");
/// Embedded canonical MCP 1.0.0 Client CLI checklist.
const MCP_100_CLI: &str = include_str!("../mcp-1.0.0/client-sdk-cli.md");
/// Embedded canonical MCP 1.0.0 Servers checklist.
const MCP_100_SRV: &str = include_str!("../mcp-1.0.0/servers.md");

/// Embedded canonical MCP 2026-07-28 Client SDK checklist.
const MCP_2026_SDK: &str = include_str!("../mcp-2026-07-28/client-sdk.md");
/// Embedded canonical MCP 2026-07-28 Client CLI checklist.
const MCP_2026_CLI: &str = include_str!("../mcp-2026-07-28/client-sdk-cli.md");
/// Embedded canonical MCP 2026-07-28 Servers checklist.
const MCP_2026_SRV: &str = include_str!("../mcp-2026-07-28/servers.md");

/// Embedded canonical Mock Server Plan checklist.
const MOCK_SERVER_PLAN: &str = include_str!("../mock-server/mock-server-plan.md");

/// Registry providing access to canonical specification checklists (embedded or via custom directory).
#[derive(Debug, Clone, Default)]
pub struct CanonicalRegistry {
    /// Optional filesystem override directory containing canonical specification folders.
    pub custom_root: Option<PathBuf>,
}

impl CanonicalRegistry {
    /// Creates a new registry backed by compile-time embedded canonical tables.
    ///
    /// # Returns
    ///
    /// A default `CanonicalRegistry`.
    #[must_use]
    pub const fn new() -> Self {
        Self { custom_root: None }
    }

    /// Creates a registry backed by a custom filesystem directory.
    ///
    /// # Arguments
    ///
    /// * `root` - Filesystem directory containing canonical specification folders.
    ///
    /// # Returns
    ///
    /// A `CanonicalRegistry` reading from `root`.
    #[must_use]
    pub const fn with_root(root: PathBuf) -> Self {
        Self {
            custom_root: Some(root),
        }
    }

    /// Retrieves the raw Markdown content for a given specification and profile.
    ///
    /// # Arguments
    ///
    /// * `spec` - Target specification identifier.
    /// * `profile` - Target profile boundary.
    ///
    /// # Errors
    ///
    /// Returns `ConformanceError::MissingSpecDirectory` if the `(spec, profile)` pair is unsupported,
    /// or `ConformanceError::Io` if reading from a custom root directory fails.
    pub fn get_raw_content(
        &self,
        spec: SpecId,
        profile: ProfileKind,
    ) -> Result<String, ConformanceError> {
        if let Some(ref root) = self.custom_root {
            let file_path = root.join(spec.as_str()).join(profile.filename());
            if file_path.is_file() {
                return std::fs::read_to_string(&file_path).map_err(|e| ConformanceError::Io {
                    source: e,
                    path: file_path,
                });
            }
        }

        let embedded = match (spec, profile) {
            (SpecId::OpenApi320, ProfileKind::ClientSdk) => Some(OAS_320_SDK),
            (SpecId::OpenApi320, ProfileKind::ClientCli) => Some(OAS_320_CLI),
            (SpecId::OpenApi320, ProfileKind::Servers) => Some(OAS_320_SRV),

            (SpecId::OpenApi311, ProfileKind::ClientSdk) => Some(OAS_311_SDK),
            (SpecId::OpenApi311, ProfileKind::ClientCli) => Some(OAS_311_CLI),
            (SpecId::OpenApi311, ProfileKind::Servers) => Some(OAS_311_SRV),

            (SpecId::OpenApi310, ProfileKind::ClientSdk) => Some(OAS_310_SDK),
            (SpecId::OpenApi310, ProfileKind::ClientCli) => Some(OAS_310_CLI),
            (SpecId::OpenApi310, ProfileKind::Servers) => Some(OAS_310_SRV),

            (SpecId::OpenApi300, ProfileKind::ClientSdk) => Some(OAS_300_SDK),
            (SpecId::OpenApi300, ProfileKind::ClientCli) => Some(OAS_300_CLI),
            (SpecId::OpenApi300, ProfileKind::Servers) => Some(OAS_300_SRV),

            (SpecId::Swagger20, ProfileKind::ClientSdk) => Some(SWAGGER_20_SDK),
            (SpecId::Swagger20, ProfileKind::ClientCli) => Some(SWAGGER_20_CLI),
            (SpecId::Swagger20, ProfileKind::Servers) => Some(SWAGGER_20_SRV),

            (SpecId::Arazzo110, ProfileKind::ClientSdk) => Some(ARAZZO_110_SDK),
            (SpecId::Arazzo110, ProfileKind::ClientCli) => Some(ARAZZO_110_CLI),
            (SpecId::Arazzo110, ProfileKind::Servers) => Some(ARAZZO_110_SRV),

            (SpecId::Mcp100, ProfileKind::ClientSdk) => Some(MCP_100_SDK),
            (SpecId::Mcp100, ProfileKind::ClientCli) => Some(MCP_100_CLI),
            (SpecId::Mcp100, ProfileKind::Servers) => Some(MCP_100_SRV),

            (SpecId::Mcp20260728, ProfileKind::ClientSdk) => Some(MCP_2026_SDK),
            (SpecId::Mcp20260728, ProfileKind::ClientCli) => Some(MCP_2026_CLI),
            (SpecId::Mcp20260728, ProfileKind::Servers) => Some(MCP_2026_SRV),

            (SpecId::MockServer, ProfileKind::MockServerPlan) => Some(MOCK_SERVER_PLAN),
            _ => None,
        };

        embedded
            .map(ToString::to_string)
            .ok_or_else(|| ConformanceError::MissingSpecDirectory {
                expected_dir: PathBuf::from(format!("{}/{}", spec.as_str(), profile.filename())),
            })
    }

    /// Parses and returns the canonical `ChecklistTable` for a specification and profile.
    ///
    /// # Arguments
    ///
    /// * `spec` - Target specification identifier.
    /// * `profile` - Target profile boundary.
    ///
    /// # Errors
    ///
    /// Returns `ConformanceError` if the table does not exist or fails to parse.
    pub fn get_table(
        &self,
        spec: SpecId,
        profile: ProfileKind,
    ) -> Result<ChecklistTable, ConformanceError> {
        let raw = self.get_raw_content(spec, profile)?;
        let virtual_path = PathBuf::from(format!("{}/{}", spec.as_str(), profile.filename()));
        parse_checklist(&raw, &virtual_path, spec, profile)
    }

    /// Lists all canonical specifications available in the registry.
    ///
    /// # Returns
    ///
    /// Static slice of all supported `SpecId` variants.
    #[must_use]
    pub const fn list_specs(&self) -> &'static [SpecId] {
        SpecId::all()
    }

    /// Lists the applicable profiles for a given specification.
    ///
    /// # Arguments
    ///
    /// * `spec` - Target specification identifier.
    ///
    /// # Returns
    ///
    /// Static slice of `ProfileKind` variants applicable to `spec`.
    #[must_use]
    pub const fn list_profiles(&self, spec: SpecId) -> &'static [ProfileKind] {
        match spec {
            SpecId::MockServer => &[ProfileKind::MockServerPlan],
            _ => ProfileKind::standard_profiles(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_embedded_canonical_tables_parse_cleanly() {
        let registry = CanonicalRegistry::new();
        for spec in registry.list_specs() {
            for profile in registry.list_profiles(*spec) {
                let table = registry.get_table(*spec, *profile).ok();
                assert!(table.is_some_and(|t| !t.rows.is_empty()));
            }
        }
    }

    #[test]
    fn test_invalid_spec_profile_combination() {
        let registry = CanonicalRegistry::new();
        let res = registry.get_table(SpecId::OpenApi320, ProfileKind::MockServerPlan);
        assert!(res.is_err());
    }

    #[test]
    fn test_custom_root_registry() {
        let dir = std::env::temp_dir().join(format!("cdd_reg_t1_{}", std::process::id()));
        let oas_dir = dir.join("openapi-3.2.0");
        let _ = std::fs::create_dir_all(&oas_dir);
        let file_path = oas_dir.join("client-sdk.md");
        let _ = std::fs::write(
            &file_path,
            "# Custom Table\n| Object / Feature | Presence [To, From] |\n| :--- | :---: |\n| Custom Row | `[x]` , `[x]` |\n",
        );

        let reg = CanonicalRegistry::with_root(dir.clone());
        let table = reg
            .get_table(SpecId::OpenApi320, ProfileKind::ClientSdk)
            .ok();
        assert_eq!(
            table.map(|t| (t.rows.len(), t.rows[0].feature_name.clone())),
            Some((1, "Custom Row".to_string()))
        );

        let fallback = reg.get_table(SpecId::OpenApi320, ProfileKind::Servers).ok();
        assert!(fallback.is_some_and(|t| !t.rows.is_empty()));

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&file_path, std::fs::Permissions::from_mode(0o000));
            assert!(reg
                .get_table(SpecId::OpenApi320, ProfileKind::ClientSdk)
                .is_err());
            let _ = std::fs::set_permissions(&file_path, std::fs::Permissions::from_mode(0o644));
        }
        let _ = std::fs::remove_dir_all(&dir);
    }
}
