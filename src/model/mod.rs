//! Strongly typed domain models for specifications, profiles, checklists, and Arazzo workflows.

/// Strongly typed Arazzo 1.1.0 specification AST and validation routines.
pub mod arazzo;

/// Checklist tables, directional flags, feature statuses, and profile scores.
pub mod checklist;

/// Strongly typed language identifiers and custom language slugs.
pub mod language;

/// Specification identifiers, profile kinds, server frameworks, and runtime verification enums.
pub mod spec;

pub use arazzo::*;
pub use checklist::*;
pub use language::*;
pub use spec::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::path::Path;
    use std::str::FromStr;

    #[test]
    fn test_spec_id_roundtrip_and_dir_resolution() {
        for spec in SpecId::all() {
            let s = spec.as_str();
            assert_eq!(SpecId::from_str(s).ok(), Some(*spec));
            assert_eq!(spec.to_string(), s);
            assert_eq!(SpecId::from_dir_name(s), Some(*spec));
        }
        assert_eq!(
            SpecId::from_dir_name("compliance-openapi-3-2-0"),
            Some(SpecId::OpenApi320)
        );
        assert_eq!(SpecId::from_dir_name("unknown-dir"), None);

        assert!(SpecId::MockServer.supports_profile(ProfileKind::MockServerPlan));
        assert!(!SpecId::MockServer.supports_profile(ProfileKind::ClientSdk));
        assert!(SpecId::OpenApi320.supports_profile(ProfileKind::ClientSdk));
        assert!(!SpecId::OpenApi320.supports_profile(ProfileKind::MockServerPlan));

        let bad_spec = SpecId::from_str("nonexistent-spec");
        assert!(bad_spec.is_err());
    }

    #[test]
    fn test_profile_kind_roundtrip_and_priority() {
        for profile in ProfileKind::standard_profiles() {
            let s = profile.as_str();
            assert_eq!(ProfileKind::from_str(s).ok(), Some(*profile));
            assert_eq!(profile.to_string(), s);
            assert_ne!(profile.filename(), "");
        }
        assert_eq!(
            ProfileKind::from_str("mock-server-plan").ok(),
            Some(ProfileKind::MockServerPlan)
        );

        let bad_prof = ProfileKind::from_str("unknown-profile");
        assert!(bad_prof.is_err());

        assert_eq!(
            ProfileKind::filename_priority(Path::new("client-sdk.md")),
            FilenamePriority::Canonical
        );
        assert_eq!(
            ProfileKind::filename_priority(Path::new("client-cli.md")),
            FilenamePriority::LegacyClientCli
        );
        assert_eq!(
            ProfileKind::filename_priority(Path::new("sdk-gen.md")),
            FilenamePriority::LegacySdkGen
        );
        assert_eq!(
            ProfileKind::filename_priority(Path::new("cli-gen.md")),
            FilenamePriority::LegacyCliGen
        );
        assert_eq!(
            ProfileKind::filename_priority(Path::new("server-gen.md")),
            FilenamePriority::LegacyServerGen
        );
        assert_eq!(
            ProfileKind::filename_priority(Path::new("other.txt")),
            FilenamePriority::Fallback
        );
        assert!(FilenamePriority::Canonical < FilenamePriority::LegacyClientCli);
    }

    #[test]
    fn test_direction_and_directional_flags() {
        assert_eq!(Direction::To.to_string(), "To");
        assert_eq!(Direction::From.to_string(), "From");
        assert_eq!(Direction::Bidirectional.to_string(), "Overall");

        let mut flags = DirectionalFlags::from([true, false]);
        assert!(flags.get(Direction::To));
        assert!(!flags.get(Direction::From));
        assert!(!flags.get(Direction::Bidirectional));
        assert_eq!(flags.to_array(), [true, false]);

        flags.set(Direction::From, true);
        assert!(flags.get(Direction::Bidirectional));
        flags.set(Direction::To, false);
        assert!(!flags.to);
        assert!(!flags.get(Direction::Bidirectional));

        let arr: [bool; 2] = flags.into();
        assert_eq!(arr, [false, true]);

        flags.set(Direction::Bidirectional, true);
        assert_eq!(flags.to_array(), [true, true]);

        let serialized = serde_json::to_string(&flags).unwrap_or_default();
        assert_eq!(serialized, "[true,true]");
        let deserialized: Option<DirectionalFlags> = serde_json::from_str(&serialized).ok();
        assert_eq!(deserialized, Some(flags));
    }

    #[test]
    fn test_feature_status_queries() {
        let mut status = FeatureStatus::new();
        assert!(status.is_unreviewed());
        assert!(status.is_to_unreviewed());
        assert!(status.is_from_unreviewed());
        assert!(status.is_unreviewed_in(Direction::To));
        assert!(status.is_unreviewed_in(Direction::From));
        assert!(status.is_unreviewed_in(Direction::Bidirectional));
        assert!(!status.has_any_contradiction());

        status.presence = DirectionalFlags::new(true, false);
        assert!(status.is_to_present());
        assert!(status.is_present(Direction::To));
        assert!(!status.is_from_present());
        assert!(!status.is_present(Direction::From));
        assert!(status.is_unreviewed());

        status.presence = DirectionalFlags::new(true, true);
        assert!(status.is_present(Direction::Bidirectional));
        assert!(!status.is_unreviewed());
    }

    #[test]
    fn test_feature_status_contradictions() {
        let abs_status = FeatureStatus::from_arrays([false, false], [true, true], [false, false]);
        assert!(abs_status.is_to_absent());
        assert!(abs_status.is_from_absent());
        assert!(abs_status.is_absent(Direction::Bidirectional));

        let skip_status = FeatureStatus::from_arrays([false, false], [false, false], [true, true]);
        assert!(skip_status.is_to_skipped());
        assert!(skip_status.is_from_skipped());
        assert!(skip_status.is_skipped(Direction::Bidirectional));

        let contra_to = FeatureStatus::from_arrays([true, false], [false, false], [true, false]);
        assert!(contra_to.has_to_contradiction());
        assert!(contra_to.has_contradiction_in(Direction::To));
        assert!(!contra_to.has_from_contradiction());
        assert!(!contra_to.has_contradiction_in(Direction::From));
        assert!(contra_to.has_any_contradiction());
        assert!(contra_to.has_contradiction_in(Direction::Bidirectional));

        let contra_from = FeatureStatus::from_arrays([false, false], [false, true], [false, true]);
        assert!(!contra_from.has_to_contradiction());
        assert!(contra_from.has_from_contradiction());
        assert!(contra_from.has_any_contradiction());
    }

    #[test]
    fn test_checklist_table_scoring_and_saturation() {
        let table = ChecklistTable {
            spec_id: SpecId::OpenApi320,
            profile: ProfileKind::ClientSdk,
            title: "Test Checklist".to_string(),
            rows: vec![
                ChecklistRow {
                    feature_name: "Feature 1".to_string(),
                    status: FeatureStatus::from_arrays(
                        [true, true],
                        [false, false],
                        [false, false],
                    ),
                    notes: None,
                    line_number: 10,
                },
                ChecklistRow {
                    feature_name: "Feature 2".to_string(),
                    status: FeatureStatus::from_arrays(
                        [false, false],
                        [true, false],
                        [false, true],
                    ),
                    notes: Some("Skip note".to_string()),
                    line_number: 11,
                },
                ChecklistRow {
                    feature_name: "Feature 3".to_string(),
                    status: FeatureStatus::new(),
                    notes: None,
                    line_number: 12,
                },
            ],
        };

        let score = table.score();
        assert_eq!(score.total_items, 3);
        assert_eq!(score.implemented_to, 1);
        assert_eq!(score.implemented_from, 1);
        assert_eq!(score.absent_to, 1);
        assert_eq!(score.absent_from, 0);
        assert_eq!(score.absent_count, 1);
        assert_eq!(score.skipped_to, 0);
        assert_eq!(score.skipped_from, 1);
        assert_eq!(score.skipped_count, 1);
        assert_eq!(score.unreviewed_to, 1);
        assert_eq!(score.unreviewed_from, 1);
        assert_eq!(score.unreviewed_count, 1);

        let expected_pct = (1.0 / 3.0) * 100.0;
        assert!((score.percentage_for(Direction::To) - expected_pct).abs() < f64::EPSILON);
        assert!((score.percentage_for(Direction::From) - expected_pct).abs() < f64::EPSILON);
        assert!(
            (score.percentage_for(Direction::Bidirectional) - expected_pct).abs() < f64::EPSILON
        );

        let empty_score = ProfileScore::default();
        assert!((empty_score.to_percentage() - 0.0).abs() < f64::EPSILON);
        assert!((empty_score.from_percentage() - 0.0).abs() < f64::EPSILON);
        assert!((empty_score.total_percentage() - 0.0).abs() < f64::EPSILON);

        assert!((usize_to_f64(42) - 42.0).abs() < f64::EPSILON);
        assert!((usize_to_f64(usize::MAX) - f64::from(u32::MAX)).abs() < f64::EPSILON);
    }

    #[test]
    #[allow(clippy::cognitive_complexity)]
    fn test_language_id_and_custom_slug() {
        assert_eq!(LanguageId::core_13().len(), 13);
        for lang in LanguageId::core_13() {
            assert!(lang.is_core());
            let s = lang.as_str();
            assert_eq!(LanguageId::from_str(s).ok().as_ref(), Some(lang));
            assert_eq!(lang.to_string(), s);
            assert_eq!(lang.repo_name(), format!("cdd-{s}"));
            let json = serde_json::to_string(lang).unwrap_or_default();
            let round: Option<LanguageId> = serde_json::from_str(&json).ok();
            assert_eq!(round.as_ref(), Some(lang));
        }

        assert_eq!(LanguageId::from_str("cdd-c++").ok(), Some(LanguageId::Cpp));
        assert_eq!(LanguageId::from_str("c#").ok(), Some(LanguageId::CSharp));
        assert_eq!(LanguageId::from_str("golang").ok(), Some(LanguageId::Go));
        assert_eq!(LanguageId::from_str("kt").ok(), Some(LanguageId::Kotlin));
        assert_eq!(LanguageId::from_str("py").ok(), Some(LanguageId::Python));
        assert_eq!(LanguageId::from_str("rb").ok(), Some(LanguageId::Ruby));
        assert_eq!(LanguageId::from_str("rs").ok(), Some(LanguageId::Rust));
        assert_eq!(LanguageId::from_str("shell").ok(), Some(LanguageId::Sh));
        assert_eq!(
            LanguageId::from_str("typescript").ok(),
            Some(LanguageId::TypeScript)
        );

        let zig = LanguageId::from_str("cdd-zig").ok();
        assert!(zig.as_ref().is_some_and(|l| !l.is_core()));
        assert_eq!(zig.as_ref().map(LanguageId::as_str), Some("zig"));
        assert_eq!(
            zig.as_ref().map(LanguageId::repo_name),
            Some("cdd-zig".to_string())
        );

        let cobol_slug = CustomLanguageSlug("cobol".to_string());
        assert_eq!(
            CustomLanguageSlug::from_str("cobol").ok(),
            Some(cobol_slug.clone())
        );
        assert_eq!(cobol_slug.as_str(), "cobol");
        assert_eq!(cobol_slug.to_string(), "cobol");
        let slug_json = serde_json::to_string(&cobol_slug).unwrap_or_default();
        let slug_back: Option<CustomLanguageSlug> = serde_json::from_str(&slug_json).ok();
        assert_eq!(slug_back.as_ref(), Some(&cobol_slug));
        assert!(serde_json::from_str::<CustomLanguageSlug>(r#""123bad""#).is_err());
        let s: String = cobol_slug.clone().into();
        assert_eq!(s, "cobol");
        let try_back = CustomLanguageSlug::try_from(s).ok();
        assert_eq!(try_back, Some(cobol_slug));

        assert!(CustomLanguageSlug::new("a1").is_ok());
        assert!(CustomLanguageSlug::new("").is_err());
        assert!(CustomLanguageSlug::new("123bad").is_err());
        assert!(CustomLanguageSlug::new("bad_underscore").is_err());
        assert!(CustomLanguageSlug::new("bad-").is_err());
        assert!(LanguageId::try_from(String::new()).is_err());
    }

    #[test]
    fn test_server_mode_wire_protocol_http_method_and_endpoint() {
        for mode in ServerOperatingMode::all() {
            let s = mode.as_str();
            assert_eq!(ServerOperatingMode::from_str(s).ok(), Some(*mode));
            assert_eq!(mode.to_string(), s);
        }
        assert_eq!(
            ServerOperatingMode::from_str("ephemeral").ok(),
            Some(ServerOperatingMode::StatefulEphemeral)
        );
        assert_eq!(
            ServerOperatingMode::from_str("seeded").ok(),
            Some(ServerOperatingMode::SeededMock)
        );
        assert_eq!(
            ServerOperatingMode::from_str("prod").ok(),
            Some(ServerOperatingMode::ProductionDb)
        );
        assert_eq!(
            ServerOperatingMode::from_str("strict").ok(),
            Some(ServerOperatingMode::StrictValidation)
        );
        assert_eq!(
            ServerOperatingMode::from_str("auth").ok(),
            Some(ServerOperatingMode::EnforceAuthMock)
        );
        assert_eq!(
            ServerOperatingMode::from_str("idp").ok(),
            Some(ServerOperatingMode::IntegratedIdp)
        );
        assert!(ServerOperatingMode::from_str("invalid-mode").is_err());

        assert_eq!(WireProtocol::HttpRest.to_string(), "http-rest");
        assert_eq!(WireProtocol::JsonRpc.to_string(), "json-rpc");
        assert_eq!(WireProtocol::McpStdio.to_string(), "mcp-stdio");
        assert_eq!(WireProtocol::McpHttp.to_string(), "mcp-http");

        assert_eq!(HttpMethod::all().len(), 9);
        for (method, expected) in [
            (HttpMethod::Get, "GET"),
            (HttpMethod::Post, "POST"),
            (HttpMethod::Put, "PUT"),
            (HttpMethod::Delete, "DELETE"),
            (HttpMethod::Patch, "PATCH"),
            (HttpMethod::Head, "HEAD"),
            (HttpMethod::Options, "OPTIONS"),
            (HttpMethod::Trace, "TRACE"),
            (HttpMethod::Query, "QUERY"),
        ] {
            assert_eq!(method.as_str(), expected);
            assert_eq!(method.to_string(), expected);
        }

        let ep1 = EndpointPath::new("/pets");
        assert_eq!(ep1.as_str(), "/pets");
        assert_eq!(ep1.to_string(), "/pets");

        let ep2 = EndpointPath::new("pets/123");
        assert_eq!(ep2.as_str(), "/pets/123");

        let code_ok = HttpStatusCode::try_new(200).ok();
        assert_eq!(code_ok.map(HttpStatusCode::as_u16), Some(200));
        assert_eq!(code_ok.map(|c| c.to_string()), Some("200".to_string()));
        assert!(HttpStatusCode::try_new(99).is_err());
        assert!(HttpStatusCode::try_new(600).is_err());

        let ptr_ok = JsonPointer::try_new("/items/0/id").ok();
        assert_eq!(
            ptr_ok.as_ref().map(JsonPointer::as_str),
            Some("/items/0/id")
        );
        assert_eq!(
            ptr_ok.as_ref().map(ToString::to_string),
            Some("/items/0/id".to_string())
        );
        assert!(JsonPointer::try_new("").is_ok());
        assert!(JsonPointer::try_new("relative/path").is_err());
    }

    #[test]
    #[allow(clippy::cognitive_complexity, clippy::too_many_lines)]
    fn test_arazzo_domain_primitives() {
        let v = ArazzoVersion::from_str("1.1.0").ok();
        assert_eq!(v.as_ref().map(ArazzoVersion::as_str), Some("1.1.0"));
        assert_eq!(
            v.as_ref().map(ToString::to_string),
            Some("1.1.0".to_string())
        );
        assert!(ArazzoVersion::from_str("1.0.0").is_err());
        assert!(ArazzoVersion::from_str("1.1.").is_err());
        assert!(ArazzoVersion::from_str("1.1.x").is_err());

        for (raw, expected) in [
            ("openapi", ArazzoSourceType::OpenApi),
            ("asyncapi", ArazzoSourceType::AsyncApi),
            ("arazzo", ArazzoSourceType::Arazzo),
        ] {
            assert_eq!(ArazzoSourceType::from_str(raw).ok(), Some(expected));
            assert_eq!(expected.as_str(), raw);
            assert_eq!(expected.to_string(), raw);
        }
        assert!(ArazzoSourceType::from_str("graphql").is_err());

        for (raw, expected) in [
            ("path", ArazzoParameterIn::Path),
            ("query", ArazzoParameterIn::Query),
            ("querystring", ArazzoParameterIn::QueryString),
            ("header", ArazzoParameterIn::Header),
            ("cookie", ArazzoParameterIn::Cookie),
        ] {
            assert_eq!(ArazzoParameterIn::from_str(raw).ok(), Some(expected));
            assert_eq!(expected.as_str(), raw);
            assert_eq!(expected.to_string(), raw);
        }
        assert!(ArazzoParameterIn::from_str("body").is_err());

        for target in [
            ArazzoStepTarget::OperationId("op1".to_string()),
            ArazzoStepTarget::OperationPath(
                "{$sourceDescriptions.pet.url}#/paths/~1pets".to_string(),
            ),
            ArazzoStepTarget::ChannelPath(
                "{$sourceDescriptions.events.url}#/channels/~1petPlaced".to_string(),
            ),
            ArazzoStepTarget::WorkflowId("subWf".to_string()),
        ] {
            assert_ne!(target.value(), "");
        }

        for (raw, expected) in [
            ("send", ArazzoMessageAction::Send),
            ("receive", ArazzoMessageAction::Receive),
        ] {
            assert_eq!(ArazzoMessageAction::from_str(raw).ok(), Some(expected));
            assert_eq!(expected.as_str(), raw);
            assert_eq!(expected.to_string(), raw);
        }
        assert!(ArazzoMessageAction::from_str("publish").is_err());

        for (raw, expected) in [
            ("end", ArazzoSuccessActionType::End),
            ("goto", ArazzoSuccessActionType::Goto),
        ] {
            assert_eq!(ArazzoSuccessActionType::from_str(raw).ok(), Some(expected));
            assert_eq!(expected.as_str(), raw);
            assert_eq!(expected.to_string(), raw);
        }
        assert!(ArazzoSuccessActionType::from_str("retry").is_err());

        for (raw, expected) in [
            ("end", ArazzoFailureActionType::End),
            ("retry", ArazzoFailureActionType::Retry),
            ("goto", ArazzoFailureActionType::Goto),
        ] {
            assert_eq!(ArazzoFailureActionType::from_str(raw).ok(), Some(expected));
            assert_eq!(expected.as_str(), raw);
            assert_eq!(expected.to_string(), raw);
        }
        assert!(ArazzoFailureActionType::from_str("ignore").is_err());

        for (val, expected) in [
            (serde_json::json!("simple"), ArazzoCriterionType::Simple),
            (serde_json::json!("regex"), ArazzoCriterionType::Regex),
            (
                serde_json::json!("jsonpath"),
                ArazzoCriterionType::JsonPathRfc9535,
            ),
            (serde_json::json!("xpath"), ArazzoCriterionType::XPath31),
            (
                serde_json::json!("jsonpointer"),
                ArazzoCriterionType::JsonPointerRfc6901,
            ),
            (
                serde_json::json!({"type": "jsonpath", "version": "rfc9535"}),
                ArazzoCriterionType::JsonPathRfc9535,
            ),
            (
                serde_json::json!({"type": "xpath", "version": "xpath-31"}),
                ArazzoCriterionType::XPath31,
            ),
            (
                serde_json::json!({"type": "xpath", "version": "xpath-20"}),
                ArazzoCriterionType::XPath20,
            ),
            (
                serde_json::json!({"type": "xpath", "version": "xpath-10"}),
                ArazzoCriterionType::XPath10,
            ),
            (
                serde_json::json!({"type": "jsonpointer", "version": "rfc6901"}),
                ArazzoCriterionType::JsonPointerRfc6901,
            ),
        ] {
            assert_eq!(
                ArazzoCriterionType::from_json_value(&val, &LanguageId::Rust, "/type").ok(),
                Some(expected)
            );
        }
        assert!(ArazzoCriterionType::from_json_value(
            &serde_json::json!("unknown"),
            &LanguageId::Rust,
            "/type"
        )
        .is_err());
        assert!(ArazzoCriterionType::from_json_value(
            &serde_json::json!({"type": "jsonpath", "version": "bad"}),
            &LanguageId::Rust,
            "/type"
        )
        .is_err());
        assert!(ArazzoCriterionType::from_json_value(
            &serde_json::json!(123),
            &LanguageId::Rust,
            "/type"
        )
        .is_err());

        let sel = ArazzoSelectorObject {
            context: "$response.body".to_string(),
            selector: "$.id".to_string(),
            selector_type: Some(ArazzoCriterionType::JsonPathRfc9535),
        };
        assert_eq!(sel.selector, "$.id");

        for (expr, expected) in [
            ("$url", ArazzoRuntimeExpressionKind::Url),
            ("$method", ArazzoRuntimeExpressionKind::Method),
            ("$statusCode", ArazzoRuntimeExpressionKind::StatusCode),
            ("$self", ArazzoRuntimeExpressionKind::SelfUri),
            (
                "$request.header.accept",
                ArazzoRuntimeExpressionKind::RequestHeader,
            ),
            (
                "$request.query.limit",
                ArazzoRuntimeExpressionKind::RequestQuery,
            ),
            ("$request.path.id", ArazzoRuntimeExpressionKind::RequestPath),
            ("$request.body", ArazzoRuntimeExpressionKind::RequestBody),
            (
                "$request.body#/id",
                ArazzoRuntimeExpressionKind::RequestBody,
            ),
            ("$request.body.id", ArazzoRuntimeExpressionKind::RequestBody),
            (
                "$response.header.x-rate",
                ArazzoRuntimeExpressionKind::ResponseHeader,
            ),
            ("$response.body", ArazzoRuntimeExpressionKind::ResponseBody),
            (
                "$response.body#/id",
                ArazzoRuntimeExpressionKind::ResponseBody,
            ),
            (
                "$response.body.id",
                ArazzoRuntimeExpressionKind::ResponseBody,
            ),
            (
                "$message.header",
                ArazzoRuntimeExpressionKind::MessageHeader,
            ),
            (
                "$message.header#/id",
                ArazzoRuntimeExpressionKind::MessageHeader,
            ),
            (
                "$message.header.id",
                ArazzoRuntimeExpressionKind::MessageHeader,
            ),
            (
                "$message.payload",
                ArazzoRuntimeExpressionKind::MessagePayload,
            ),
            (
                "$message.payload#/id",
                ArazzoRuntimeExpressionKind::MessagePayload,
            ),
            (
                "$message.payload.id",
                ArazzoRuntimeExpressionKind::MessagePayload,
            ),
            ("$inputs.petId", ArazzoRuntimeExpressionKind::Inputs),
            ("$outputs.orderId", ArazzoRuntimeExpressionKind::Outputs),
            (
                "$steps.step1.outputs.token",
                ArazzoRuntimeExpressionKind::Steps,
            ),
            (
                "$workflows.wf1.outputs.res",
                ArazzoRuntimeExpressionKind::Workflows,
            ),
            (
                "$sourceDescriptions.petStore.url",
                ArazzoRuntimeExpressionKind::SourceDescriptions,
            ),
            (
                "$components.parameters.page",
                ArazzoRuntimeExpressionKind::Components,
            ),
        ] {
            assert_eq!(
                ArazzoRuntimeExpressionKind::parse_root(expr, "/expr").ok(),
                Some(expected)
            );
        }
        assert!(ArazzoRuntimeExpressionKind::parse_root("$unknown.val", "/expr").is_err());
        assert!(ArazzoRuntimeExpressionKind::parse_root("$inputs.", "/expr").is_err());

        let step_sum = ArazzoStepSummary {
            step_id: "step1".to_string(),
            target: ArazzoStepTarget::OperationId("listPets".to_string()),
            action: Some(ArazzoMessageAction::Send),
            output_keys: vec!["petId".to_string()],
        };
        let wf_sum = ArazzoWorkflowSummary {
            workflow_id: "wf1".to_string(),
            input_properties: vec!["coupon".to_string()],
            steps: vec![step_sum],
            output_keys: vec!["resultId".to_string()],
        };
        assert_eq!(wf_sum.workflow_id, "wf1");

        let param_obj = ArazzoParameterObject {
            name: "limit".to_string(),
            in_loc: Some(ArazzoParameterIn::Query),
            value: serde_json::json!(10),
            extensions: BTreeMap::new(),
        };
        let reusable_obj = ArazzoReusableObject {
            reference: "$components.parameters.limit".to_string(),
            value: None,
        };
        let crit_obj = ArazzoCriterionObject {
            context: Some("$statusCode".to_string()),
            condition: "$statusCode == 200".to_string(),
            criterion_type: Some(ArazzoCriterionType::Simple),
            extensions: BTreeMap::new(),
        };
        let succ_act = ArazzoSuccessActionObject {
            name: "done".to_string(),
            action_type: ArazzoSuccessActionType::End,
            workflow_id: None,
            step_id: None,
            parameters: vec![ArazzoParameterOrReusable::Parameter(param_obj.clone())],
            criteria: vec![crit_obj.clone()],
            extensions: BTreeMap::new(),
        };
        let fail_act = ArazzoFailureActionObject {
            name: "retryOnce".to_string(),
            action_type: ArazzoFailureActionType::Retry,
            workflow_id: None,
            step_id: Some("step1".to_string()),
            parameters: vec![ArazzoParameterOrReusable::Reusable(reusable_obj.clone())],
            retry_after: Some(1.0),
            retry_limit: Some(3),
            criteria: vec![crit_obj.clone()],
            extensions: BTreeMap::new(),
        };
        let mut step_outputs = BTreeMap::new();
        step_outputs.insert("selectedId".to_string(), ArazzoOutputValue::Selector(sel));
        step_outputs.insert(
            "rawStatus".to_string(),
            ArazzoOutputValue::Expression("$statusCode".to_string()),
        );
        let step_ast = ArazzoStepObject {
            step_id: "step1".to_string(),
            description: Some("Find pet".to_string()),
            target: ArazzoStepTarget::OperationId("listPets".to_string()),
            parameters: vec![ArazzoParameterOrReusable::Parameter(param_obj.clone())],
            request_body: Some(ArazzoRequestBodyObject {
                content_type: Some("application/json".to_string()),
                payload: Some(serde_json::json!({"petId": 1})),
                replacements: vec![ArazzoPayloadReplacement {
                    target: "/petId".to_string(),
                    value: serde_json::json!("$inputs.petId"),
                    target_selector_type: Some(ArazzoCriterionType::JsonPointerRfc6901),
                    extensions: BTreeMap::new(),
                }],
                extensions: BTreeMap::new(),
            }),
            depends_on: Vec::new(),
            timeout: Some(5000),
            correlation_id: None,
            action: None,
            success_criteria: vec![crit_obj],
            on_success: vec![
                ArazzoSuccessActionOrReusable::Action(succ_act.clone()),
                ArazzoSuccessActionOrReusable::Reusable(reusable_obj.clone()),
            ],
            on_failure: vec![
                ArazzoFailureActionOrReusable::Action(fail_act.clone()),
                ArazzoFailureActionOrReusable::Reusable(reusable_obj),
            ],
            outputs: step_outputs.clone(),
            extensions: BTreeMap::new(),
        };
        let wf_ast = ArazzoWorkflowObject {
            workflow_id: "wf1".to_string(),
            summary: Some("Summary".to_string()),
            description: Some("Desc".to_string()),
            inputs: Some(serde_json::json!({"type": "object"})),
            depends_on: Vec::new(),
            steps: vec![step_ast],
            success_actions: Vec::new(),
            failure_actions: Vec::new(),
            outputs: step_outputs,
            parameters: Vec::new(),
            extensions: BTreeMap::new(),
        };
        let doc_ast = ArazzoDocument {
            arazzo: ArazzoVersion("1.1.0".to_string()),
            self_uri: Some("https://example.com/wf.arazzo.yaml".to_string()),
            info: ArazzoInfoObject {
                title: "Pet Workflow".to_string(),
                summary: None,
                description: None,
                version: "1.0.0".to_string(),
                extensions: BTreeMap::new(),
            },
            source_descriptions: vec![ArazzoSourceDescription {
                name: "petApi".to_string(),
                url: "https://example.com/openapi.yaml".to_string(),
                source_type: Some(ArazzoSourceType::OpenApi),
                extensions: BTreeMap::new(),
            }],
            workflows: vec![wf_ast],
            components: Some(ArazzoComponentsObject {
                inputs: BTreeMap::new(),
                parameters: BTreeMap::from([("limit".to_string(), param_obj)]),
                success_actions: BTreeMap::from([("done".to_string(), succ_act)]),
                failure_actions: BTreeMap::from([("retryOnce".to_string(), fail_act)]),
                extensions: BTreeMap::new(),
            }),
            extensions: BTreeMap::new(),
        };
        assert_eq!(doc_ast.arazzo.as_str(), "1.1.0");
    }

    #[test]
    #[allow(clippy::cognitive_complexity, clippy::too_many_lines)]
    fn test_strong_enums_and_normalized_openapi_spec() {
        assert_eq!(
            EndpointPath::try_new("/pets/1")
                .ok()
                .as_ref()
                .map(EndpointPath::as_str),
            Some("/pets/1")
        );
        assert!(EndpointPath::try_new("").is_err());
        assert!(EndpointPath::try_new("pets").is_err());
        assert!(EndpointPath::try_new("/pets with space").is_err());

        for (raw, expected) in [
            ("actix-web", ServerFramework::ActixWeb),
            ("actix", ServerFramework::ActixWeb),
            ("axum", ServerFramework::Axum),
            ("mcp-stdio", ServerFramework::McpStdio),
            ("mcp-http", ServerFramework::McpHttp),
            ("arazzo-engine", ServerFramework::ArazzoEngine),
            ("arazzo", ServerFramework::ArazzoEngine),
        ] {
            assert_eq!(ServerFramework::from_str(raw).ok(), Some(expected));
            assert_eq!(expected.to_string(), expected.as_str());
        }
        assert!(ServerFramework::from_str("flask").is_err());

        for (raw, expected) in [
            ("verified", ClientVerificationStatus::Verified),
            (
                "ready-to-generate",
                ClientVerificationStatus::ReadyToGenerate,
            ),
            ("missing-repo", ClientVerificationStatus::MissingRepo),
            (
                "failed-verification",
                ClientVerificationStatus::FailedVerification,
            ),
        ] {
            assert_eq!(ClientVerificationStatus::from_str(raw).ok(), Some(expected));
            assert_eq!(expected.to_string(), raw);
        }
        assert!(ClientVerificationStatus::from_str("unknown").is_err());

        for (raw, expected, badge) in [
            ("compatible", CompatibilityStatus::Compatible, "✅"),
            ("configured", CompatibilityStatus::Configured, "☑️"),
            ("unsupported", CompatibilityStatus::Unsupported, "⚠️"),
            ("incompatible", CompatibilityStatus::Incompatible, "❌"),
        ] {
            assert_eq!(CompatibilityStatus::from_str(raw).ok(), Some(expected));
            assert_eq!(expected.to_string(), raw);
            assert_eq!(expected.badge(), badge);
        }
        assert!(CompatibilityStatus::from_str("unknown").is_err());

        for (raw, expected) in [
            ("live-subprocess", VerificationExecutionMode::LiveSubprocess),
            (
                "repository-static-inspection",
                VerificationExecutionMode::RepositoryStaticInspection,
            ),
            (
                "canonical-contract-baseline",
                VerificationExecutionMode::CanonicalContractBaseline,
            ),
        ] {
            assert_eq!(
                VerificationExecutionMode::from_str(raw).ok(),
                Some(expected)
            );
            assert_eq!(expected.to_string(), raw);
        }
        assert!(VerificationExecutionMode::from_str("unknown").is_err());

        assert_eq!(CompilerCapability::all().len(), 10);
        for cap in CompilerCapability::all() {
            let s = cap.as_str();
            assert_eq!(CompilerCapability::from_str(s).ok(), Some(*cap));
            assert_eq!(cap.to_string(), s);
        }
        assert!(CompilerCapability::from_str("unknown_cap").is_err());

        for (raw, expected) in [
            ("struct", SymbolKind::Struct),
            ("class", SymbolKind::Struct),
            ("field", SymbolKind::Field),
            ("property", SymbolKind::Field),
            ("function", SymbolKind::Function),
            ("fn", SymbolKind::Function),
            ("method", SymbolKind::Method),
            ("arg", SymbolKind::Arg),
            ("param", SymbolKind::Arg),
            ("parameter", SymbolKind::Arg),
            ("workflow", SymbolKind::Workflow),
        ] {
            assert_eq!(SymbolKind::from_str(raw).ok(), Some(expected));
            assert_eq!(expected.to_string(), expected.as_str());
        }
        assert!(SymbolKind::from_str("unknown_sym").is_err());

        assert_eq!(ParameterSerializationStyle::all().len(), 12);
        for style in ParameterSerializationStyle::all() {
            assert_eq!(style.to_string(), style.as_str());
        }

        assert_eq!(MediaTypeKind::all().len(), 7);
        for mt in MediaTypeKind::all() {
            assert_eq!(mt.to_string(), mt.as_str());
        }

        assert_eq!(SchemaConstructKind::all().len(), 9);
        for sc in SchemaConstructKind::all() {
            assert_eq!(sc.to_string(), sc.as_str());
        }

        assert_eq!(AuthFlowKind::all().len(), 5);
        for af in AuthFlowKind::all() {
            assert_eq!(af.to_string(), af.as_str());
        }

        assert_eq!(TopologicalCategory::all().len(), 4);
        for tc in TopologicalCategory::all() {
            assert_eq!(tc.to_string(), tc.as_str());
        }

        assert_eq!(CliOutputFormat::all().len(), 4);
        for fmt in CliOutputFormat::all() {
            assert_eq!(fmt.to_string(), fmt.as_str());
        }

        assert_eq!(McpMethodName::cli_verified_methods().len(), 12);
        for m in [
            McpMethodName::Initialize,
            McpMethodName::NotificationsInitialized,
            McpMethodName::NotificationsCancelled,
            McpMethodName::NotificationsProgress,
            McpMethodName::Ping,
            McpMethodName::ToolsList,
            McpMethodName::ToolsCall,
            McpMethodName::ResourcesList,
            McpMethodName::ResourcesRead,
            McpMethodName::ResourcesTemplatesList,
            McpMethodName::PromptsList,
            McpMethodName::PromptsGet,
            McpMethodName::LoggingSetLevel,
            McpMethodName::CompletionComplete,
        ] {
            let s = m.as_str();
            assert_eq!(McpMethodName::from_str(s).ok(), Some(m));
            assert_eq!(m.to_string(), s);
        }
        assert!(McpMethodName::from_str("unknown/method").is_err());

        assert_eq!(EquivalenceDimension::all().len(), 6);
        for dim in EquivalenceDimension::all() {
            assert_eq!(dim.to_string(), dim.as_str());
        }

        let sig_cnt = SignatureCount::new(4, 12);
        assert_eq!(sig_cnt.structs, 4);
        assert_eq!(sig_cnt.functions, 12);

        let oas_val = serde_json::json!({
            "openapi": "3.2.0",
            "components": {
                "schemas": {
                    "Pet": {
                        "required": ["id"],
                        "properties": {
                            "id": { "type": "integer" },
                            "meta": {}
                        }
                    }
                }
            },
            "paths": {
                "/pets": {
                    "parameters": [{ "name": "limit" }],
                    "get": {
                        "operationId": "listPets",
                        "parameters": [{ "name": "limit" }, { "name": "offset" }],
                        "responses": { "200": {} }
                    },
                    "post": {},
                    "put": {},
                    "delete": {},
                    "patch": {},
                    "head": {},
                    "options": {},
                    "trace": {},
                    "query": {},
                    "x-ext": true
                },
                "/bad": "not-an-object"
            }
        });
        let parsed_oas = NormalizedOpenApiSpec::from_json_value(&oas_val, &LanguageId::Rust).ok();
        assert_eq!(
            parsed_oas.as_ref().map(|s| s.spec_version.as_str()),
            Some("3.2.0")
        );
        assert!(parsed_oas.as_ref().is_some_and(
            |s| s.schemas.contains_key("Pet") && s.operations.contains_key("listPets")
        ));

        let swagger_val = serde_json::json!({
            "swagger": "2.0",
            "definitions": {
                "User": { "properties": { "name": { "type": "string" } } }
            }
        });
        assert!(NormalizedOpenApiSpec::from_json_value(&swagger_val, &LanguageId::Rust).is_ok());
        assert!(
            NormalizedOpenApiSpec::from_json_value(&serde_json::json!({}), &LanguageId::Rust)
                .is_err()
        );

        let arazzo_targets = serde_json::json!({
            "arazzo": "1.1.0",
            "x-root": 1,
            "info": { "title": "T", "version": "1.0", "summary": "S", "description": "D" },
            "sourceDescriptions": [{ "name": "s1", "url": "https://example.com/a.yaml" }],
            "components": { "inputs": { "In1": { "type": "object" } } },
            "workflows": [{
                "workflowId": "wf1",
                "summary": "S",
                "description": "D",
                "steps": [
                    { "stepId": "s1", "operationPath": "/paths/~1pets", "parameters": [{ "name": "p1", "value": 1 }] },
                    { "stepId": "s2", "channelPath": "/channels/c1", "onFailure": [{ "name": "f1", "type": "end", "retryAfter": 1.0, "retryLimit": 2 }] },
                    { "stepId": "s3", "workflowId": "subWf" }
                ]
            }]
        });
        assert!(ArazzoDocument::from_json_value(&arazzo_targets, &LanguageId::Rust).is_ok());
    }
}
