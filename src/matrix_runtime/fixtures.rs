//! Canonical `OpenAPI` 3.2.0 / Arazzo 1.1.0 documents and per-language generated code snippets.

use crate::model::LanguageId;
use crate::runner::SimpleHttpResponse;

/// Returns a canonical generated source snippet for custom extension languages (`cobol`, `fortran`, `zig`).
///
/// # Arguments
///
/// * `slug` - Custom language slug string.
/// * `mutated` - If `true`, includes the synchronized `tag` field.
///
/// # Returns
///
/// Tuple of `(virtual_filename, source_code_string)`.
#[must_use]
pub(crate) fn canonical_custom_snippet(slug: &str, mutated: bool) -> (&'static str, String) {
    let extra_zig = if mutated {
        "
    /// Tag.
    tag: []const u8,"
    } else {
        ""
    };
    let extra_cobol = if mutated {
        "
       *> Tag.
       05 TAG PIC X(32)."
    } else {
        ""
    };
    let extra_fortran = if mutated {
        "
    !> Tag.
    character(len=32) :: tag"
    } else {
        ""
    };

    match slug {
        "cobol" => (
            "sdk.cbl",
            format!(
                "       *> Pet model.
       01 PET.
       *> ID.
       05 ID PIC 9(9).
       *> Name.
       05 NAME PIC X(32).{extra_cobol}
       *> List pets.
       *> @param LIMIT Max items.
       PROGRAM-ID. LIST-PETS.
       PROCEDURE DIVISION USING LIMIT.
"
            ),
        ),
        "fortran" => (
            "sdk.f90",
            format!(
                "!> Pet model.
type :: Pet
    !> ID.
    integer :: id
    !> Name.
    character(len=32) :: name{extra_fortran}
end type Pet
!> List pets.
!> @param limit Max items.
subroutine list_pets(limit)
end subroutine list_pets
"
            ),
        ),
        _ => (
            "sdk.zig",
            format!(
                "/// Pet model.
pub const Pet = struct {{
    /// ID.
    id: i64,
    /// Name.
    name: []const u8,{extra_zig}
}};
/// List pets.
/// @param limit Max items.
pub fn listPets(limit: i32) void {{}}
"
            ),
        ),
    }
}

/// Returns a canonical generated SDK/Server source snippet for dynamic and web languages
/// (`Php`, `Python`, `Ruby`, `Sh`, `TypeScript`, and `Custom`).
///
/// # Arguments
///
/// * `lang` - Target `LanguageId`.
/// * `mutated` - If `true`, includes a synchronized `tag` field.
///
/// # Returns
///
/// Tuple of `(virtual_filename, source_code_string)`.
#[must_use]
fn canonical_dynamic_generated_snippet(lang: &LanguageId, mutated: bool) -> (&'static str, String) {
    let opt = |s: &'static str| if mutated { s } else { "" };
    match lang {
        LanguageId::Php => (
            "Sdk.php",
            format!(
                "<?php
/** Pet model. */
class Pet {{
    /** @var int ID. */
    public int $id;
    /** @var string Name. */
    public string $name;{}
    /** List pets.
     * @param int $limit Max items.
     */
    public function listPets(int $limit): void {{}}
}}
",
                opt("
    /** @var string Tag. */
    public string $tag;")
            ),
        ),
        LanguageId::Python => (
            "sdk.py",
            format!(
                r#"class Pet:
    """Pet model."""
    #: ID.
    id: int
    #: Name.
    name: str{}

def list_pets(limit: int) -> None:
    """List pets.
    :param limit: Max items.
    """
    pass
"#,
                opt("
    #: Tag.
    tag: str")
            ),
        ),
        LanguageId::Ruby => (
            "sdk.rb",
            format!(
                "# Pet model.
class Pet
  # ID.
  attr_accessor :id
  # Name.
  attr_accessor :name{}
  # List pets.
  # @param [Integer] limit Max items.
  def list_pets(limit)
  end
end
",
                opt("
  # Tag.
  attr_accessor :tag")
            ),
        ),
        LanguageId::Sh => (
            "sdk.sh",
            format!(
                r#"# @struct Pet Pet model.
# @field id ID.
# @field name Name.{}
# List pets.
# @param limit Max items.
list_pets() {{
  local limit="$1"
}}
"#,
                opt("
# @field tag Tag.")
            ),
        ),
        LanguageId::Swift => (
            "Sdk.swift",
            format!(
                "/// Pet model.\npublic struct Pet {{\n    /// ID.\n    public let id: Int64\n    /// Name.\n    public let name: String{}\n}}\n/// List pets.\n/// - Parameter limit: Max items.\npublic func listPets(limit: Int) {{}}\n",
                opt("\n    /// Tag.\n    public let tag: String")
            ),
        ),
        LanguageId::TypeScript => (
            "sdk.ts",
            format!(
                "/** Pet model. */\nexport interface Pet {{\n  /** ID. */\n  id: number;\n  /** Name. */\n  name: string;{}\n}}\n/** List pets.\n * @param limit - Max items.\n */\nexport function listPets(limit: number): void {{}}\n",
                opt("\n  /** Tag. */\n  tag: string;")
            ),
        ),
        LanguageId::Custom(slug) => canonical_custom_snippet(slug.as_str(), mutated),
        _ => (
            "sdk.rs",
            format!(
                "/// Pet model.\npub struct Pet {{\n/// ID.\npub id: i64,\n/// Name.\npub name: String,{}\n}}\n/// List pets.\n/// * `limit` - Max items.\npub fn list_pets(limit: i32) {{}}\n",
                opt("\n/// Tag.\npub tag: String,")
            ),
        ),
    }
}

/// Returns a canonical generated SDK/Server source snippet for any of the 13 core languages or custom
/// languages (`cobol`, `fortran`, `zig`, etc.) modeling `Pet` (`id`, `name`) and `listPets(limit)`.
///
/// # Arguments
///
/// * `lang` - Target `LanguageId`.
/// * `mutated` - If `true`, includes a synchronized `tag` field (`sync --truth` convergence).
///
/// # Returns
///
/// Tuple of `(virtual_filename, source_code_string)`.
#[must_use]
pub fn canonical_generated_snippet(lang: &LanguageId, mutated: bool) -> (&'static str, String) {
    let opt = |s: &'static str| if mutated { s } else { "" };
    match lang {
        LanguageId::C => (
            "sdk.h",
            format!(
                "/** Pet model. */\nstruct Pet {{\n    /** ID. */\n    long id;\n    /** Name. */\n    char *name;{}\n}};\n/** List pets.\n * @param limit Max items.\n */\nint list_pets(int limit);\n",
                opt("\n    /** Tag. */\n    char *tag;")
            ),
        ),
        LanguageId::Cpp => (
            "sdk.hpp",
            format!(
                "/** Pet model. */\nclass Pet {{\npublic:\n    /** ID. */\n    long id;\n    /** Name. */\n    std::string name;{}\n}};\n/** List pets.\n * @param limit Max items.\n */\nint list_pets(int limit);\n",
                opt("\n    /** Tag. */\n    std::string tag;")
            ),
        ),
        LanguageId::CSharp => (
            "Sdk.cs",
            format!(
                r#"/// <summary>Pet model.</summary>
public class Pet {{
    /// <summary>ID.</summary>
    public long Id {{ get; set; }}
    /// <summary>Name.</summary>
    public string Name {{ get; set; }}{}
    /// <summary>List pets.</summary>
    /// <param name="limit">Max items.</param>
    public void ListPets(int limit) {{}}
}}
"#,
                opt("
    /// <summary>Tag.</summary>
    public string Tag { get; set; }")
            ),
        ),
        LanguageId::Go => (
            "sdk.go",
            format!(
                "// Pet model.\ntype Pet struct {{\n    // ID.\n    Id int64\n    // Name.\n    Name string{}\n}}\n// ListPets lists pets for limit.\nfunc ListPets(limit int) {{}}\n",
                opt("\n    // Tag.\n    Tag string")
            ),
        ),
        LanguageId::Java => (
            "Pet.java",
            format!(
                "/** Pet model. */\npublic class Pet {{\n    /** ID. */\n    public long id;\n    /** Name. */\n    public String name;{}\n    /** List pets.\n     * @param limit Max items.\n     */\n    public void listPets(int limit) {{}}\n}}\n",
                opt("\n    /** Tag. */\n    public String tag;")
            ),
        ),
        LanguageId::Kotlin => (
            "Sdk.kt",
            format!(
                "/** Pet model.\n * @property id ID.\n * @property name Name.{}\n */\ndata class Pet(val id: Long, val name: String{})\n/** List pets.\n * @param limit Max items.\n */\nfun listPets(limit: Int) {{}}\n",
                opt("\n * @property tag Tag."),
                opt(", val tag: String")
            ),
        ),
        _ => canonical_dynamic_generated_snippet(lang, mutated),
    }
}

/// Returns a canonical generated Arazzo 1.1.0 workflow source snippet for custom extension languages.
///
/// # Arguments
///
/// * `slug` - Custom language slug string.
/// * `mutated` - If `true`, includes the synchronized `tag` field.
///
/// # Returns
///
/// Tuple of `(virtual_filename, source_code_string)`.
#[must_use]
pub(crate) fn canonical_arazzo_custom_snippet(slug: &str, mutated: bool) -> (&'static str, String) {
    let extra_zig = if mutated {
        "
    /// Tag.
    tag: []const u8,"
    } else {
        ""
    };
    let extra_cobol = if mutated {
        "
       *> Tag.
       05 TAG PIC X(32)."
    } else {
        ""
    };
    let extra_fortran = if mutated {
        "
    !> Tag.
    character(len=32) :: tag"
    } else {
        ""
    };

    match slug {
        "cobol" => (
            "workflow.cbl",
            format!(
                "       *> Purchase input.
       01 PURCHASE-INPUT.
       *> Pet ID.
       05 PET-ID PIC 9(9).
       *> Coupon code.
       05 COUPON-CODE PIC X(32).{extra_cobol}
       *> Purchase pet workflow.
       *> @param PET-ID Pet ID.
       PROGRAM-ID. PURCHASE-PET-WORKFLOW.
       PROCEDURE DIVISION USING PET-ID.
"
            ),
        ),
        "fortran" => (
            "workflow.f90",
            format!(
                "!> Purchase input.
type :: PurchaseInput
    !> Pet ID.
    integer :: pet_id
    !> Coupon code.
    character(len=32) :: coupon_code{extra_fortran}
end type PurchaseInput
!> Purchase pet workflow.
!> @param pet_id Pet ID.
subroutine purchase_pet_workflow(pet_id)
end subroutine purchase_pet_workflow
"
            ),
        ),
        _ => (
            "workflow.zig",
            format!(
                "/// Purchase input.
pub const PurchaseInput = struct {{
    /// Pet ID.
    petId: i64,
    /// Coupon code.
    couponCode: []const u8,{extra_zig}
}};
/// Purchase pet workflow.
/// @param petId Pet ID.
pub fn purchasePetWorkflow(petId: i64) void {{}}
"
            ),
        ),
    }
}

/// Returns a canonical generated Arazzo 1.1.0 workflow source snippet for dynamic and web languages
/// (`Php`, `Python`, `Ruby`, `Sh`, `TypeScript`, and `Custom`).
///
/// # Arguments
///
/// * `lang` - Target `LanguageId`.
/// * `mutated` - If `true`, includes a synchronized `tag` field.
///
/// # Returns
///
/// Tuple of `(virtual_filename, source_code_string)`.
#[must_use]
fn canonical_arazzo_dynamic_snippet(lang: &LanguageId, mutated: bool) -> (&'static str, String) {
    let opt = |s: &'static str| if mutated { s } else { "" };
    match lang {
        LanguageId::Php => (
            "Workflow.php",
            format!(
                "<?php
/** Purchase input. */
class PurchaseInput {{
    /** @var int Pet ID. */
    public int $petId;
    /** @var string Coupon code. */
    public string $couponCode;{}
    /** Purchase pet workflow.
     * @param int $petId Pet ID.
     */
    public function purchasePetWorkflow(int $petId): void {{}}
}}
",
                opt("
    /** @var string Tag. */
    public string $tag;")
            ),
        ),
        LanguageId::Python => (
            "workflow.py",
            format!(
                r#"class PurchaseInput:
    """Purchase input."""
    #: Pet ID.
    pet_id: int
    #: Coupon code.
    coupon_code: str{}

def purchase_pet_workflow(pet_id: int) -> None:
    """Purchase pet workflow.
    :param pet_id: Pet ID.
    """
    pass
"#,
                opt("
    #: Tag.
    tag: str")
            ),
        ),
        LanguageId::Ruby => (
            "workflow.rb",
            format!(
                "# Purchase input.
class PurchaseInput
  # Pet ID.
  attr_accessor :pet_id
  # Coupon code.
  attr_accessor :coupon_code{}
  # Purchase pet workflow.
  # @param [Integer] pet_id Pet ID.
  def purchase_pet_workflow(pet_id)
  end
end
",
                opt("
  # Tag.
  attr_accessor :tag")
            ),
        ),
        LanguageId::Sh => (
            "workflow.sh",
            format!(
                r#"# @struct PurchaseInput Purchase input.
# @field pet_id Pet ID.
# @field coupon_code Coupon code.{}
# Purchase pet workflow.
# @param pet_id Pet ID.
purchase_pet_workflow() {{
  local pet_id="$1"
}}
"#,
                opt("
# @field tag Tag.")
            ),
        ),
        LanguageId::Swift => (
            "Workflow.swift",
            format!(
                "/// Purchase input.\npublic struct PurchaseInput {{\n    /// Pet ID.\n    public let petId: Int64\n    /// Coupon code.\n    public let couponCode: String{}\n}}\n/// Purchase pet workflow.\n/// - Parameter petId: Pet ID.\npublic func purchasePetWorkflow(petId: Int64) {{}}\n",
                opt("\n    /// Tag.\n    public let tag: String")
            ),
        ),
        LanguageId::TypeScript => (
            "workflow.ts",
            format!(
                "/** Purchase input. */\nexport interface PurchaseInput {{\n  /** Pet ID. */\n  petId: number;\n  /** Coupon code. */\n  couponCode: string;{}\n}}\n/** Purchase pet workflow.\n * @param petId - Pet ID.\n */\nexport function purchasePetWorkflow(petId: number): void {{}}\n",
                opt("\n  /** Tag. */\n  tag: string;")
            ),
        ),
        LanguageId::Custom(slug) => canonical_arazzo_custom_snippet(slug.as_str(), mutated),
        _ => (
            "workflow.rs",
            format!(
                "/// Purchase input.\npub struct PurchaseInput {{\n/// Pet ID.\npub pet_id: i64,\n/// Coupon code.\npub coupon_code: String,{}\n}}\n/// Purchase pet workflow.\n/// * `pet_id` - Pet ID.\npub fn purchase_pet_workflow(pet_id: i64) {{}}\n",
                opt("\n/// Tag.\npub tag: String,")
            ),
        ),
    }
}

/// Returns a canonical generated Arazzo 1.1.0 workflow source snippet for any of the 13+ languages.
///
/// Models `PurchaseInput` (`petId`, `couponCode`) and `purchasePetWorkflow(petId)`.
///
/// # Arguments
///
/// * `lang` - Target `LanguageId`.
/// * `mutated` - If `true`, includes a synchronized `tag` field.
///
/// # Returns
///
/// Tuple of `(virtual_filename, source_code_string)`.
#[must_use]
pub fn canonical_arazzo_generated_snippet(
    lang: &LanguageId,
    mutated: bool,
) -> (&'static str, String) {
    let opt = |s: &'static str| if mutated { s } else { "" };
    match lang {
        LanguageId::C => (
            "workflow.h",
            format!(
                "/** Purchase input. */\nstruct PurchaseInput {{\n    /** Pet ID. */\n    long pet_id;\n    /** Coupon code. */\n    char *coupon_code;{}\n}};\n/** Purchase pet workflow.\n * @param pet_id Pet ID.\n */\nint purchase_pet_workflow(long pet_id);\n",
                opt("\n    /** Tag. */\n    char *tag;")
            ),
        ),
        LanguageId::Cpp => (
            "workflow.hpp",
            format!(
                "/** Purchase input. */\nclass PurchaseInput {{\npublic:\n    /** Pet ID. */\n    long pet_id;\n    /** Coupon code. */\n    std::string coupon_code;{}\n}};\n/** Purchase pet workflow.\n * @param pet_id Pet ID.\n */\nint purchase_pet_workflow(long pet_id);\n",
                opt("\n    /** Tag. */\n    std::string tag;")
            ),
        ),
        LanguageId::CSharp => (
            "Workflow.cs",
            format!(
                r#"/// <summary>Purchase input.</summary>
public class PurchaseInput {{
    /// <summary>Pet ID.</summary>
    public long PetId {{ get; set; }}
    /// <summary>Coupon code.</summary>
    public string CouponCode {{ get; set; }}{}
    /// <summary>Purchase pet workflow.</summary>
    /// <param name="petId">Pet ID.</param>
    public void PurchasePetWorkflow(long petId) {{}}
}}
"#,
                opt("
    /// <summary>Tag.</summary>
    public string Tag { get; set; }")
            ),
        ),
        LanguageId::Go => (
            "workflow.go",
            format!(
                "// PurchaseInput model.\ntype PurchaseInput struct {{\n    // Pet ID.\n    PetId int64\n    // Coupon code.\n    CouponCode string{}\n}}\n// PurchasePetWorkflow executes workflow for petId.\nfunc PurchasePetWorkflow(petId int64) {{}}\n",
                opt("\n    // Tag.\n    Tag string")
            ),
        ),
        LanguageId::Java => (
            "PurchaseInput.java",
            format!(
                "/** Purchase input. */\npublic class PurchaseInput {{\n    /** Pet ID. */\n    public long petId;\n    /** Coupon code. */\n    public String couponCode;{}\n    /** Purchase pet workflow.\n     * @param petId Pet ID.\n     */\n    public void purchasePetWorkflow(long petId) {{}}\n}}\n",
                opt("\n    /** Tag. */\n    public String tag;")
            ),
        ),
        LanguageId::Kotlin => (
            "Workflow.kt",
            format!(
                "/** Purchase input.\n * @property petId Pet ID.\n * @property couponCode Coupon code.{}\n */\ndata class PurchaseInput(val petId: Long, val couponCode: String{})\n/** Purchase pet workflow.\n * @param petId Pet ID.\n */\nfun purchasePetWorkflow(petId: Long) {{}}\n",
                opt("\n * @property tag Tag."),
                opt(", val tag: String")
            ),
        ),
        _ => canonical_arazzo_dynamic_snippet(lang, mutated),
    }
}

/// Returns the canonical conformance Arazzo 1.1.0 specification JSON value used for
/// cross-language workflow signature and roundtrip verification.
///
/// # Returns
///
/// Canonical Arazzo 1.1.0 `serde_json::Value`.
#[must_use]
pub fn canonical_conformance_arazzo_doc() -> serde_json::Value {
    serde_json::json!({
        "arazzo": "1.1.0",
        "$self": "https://example.com/workflows/pet-purchase.arazzo.yaml",
        "info": {
            "title": "CDD Canonical Arazzo 1.1.0 Workflow",
            "summary": "Multi-step pet lookup and purchase orchestration",
            "version": "1.0.0"
        },
        "sourceDescriptions": [
            {
                "name": "petStoreApi",
                "url": "https://example.com/openapi.yaml",
                "type": "openapi"
            },
            {
                "name": "petEvents",
                "url": "https://example.com/asyncapi.yaml",
                "type": "asyncapi"
            }
        ],
        "components": {
            "inputs": {
                "PurchaseInput": {
                    "type": "object",
                    "properties": {
                        "petId": { "type": "integer" },
                        "couponCode": { "type": "string" }
                    }
                }
            }
        },
        "workflows": [
            {
                "workflowId": "purchasePetWorkflow",
                "inputs": {
                    "type": "object",
                    "properties": {
                        "petId": { "type": "integer" }
                    }
                },
                "steps": [
                    {
                        "stepId": "findPetStep",
                        "operationId": "listPets",
                        "timeout": 5000,
                        "parameters": [
                            {
                                "name": "petId",
                                "in": "query",
                                "value": "$inputs.petId"
                            }
                        ],
                        "successCriteria": [
                            {
                                "context": "$statusCode",
                                "condition": "$statusCode == 200",
                                "type": {
                                    "type": "jsonpath",
                                    "version": "rfc9535"
                                }
                            }
                        ],
                        "outputs": {
                            "selectedPetId": {
                                "context": "$response.body",
                                "selector": "$.items[0].id",
                                "selectorType": {
                                    "type": "jsonpath",
                                    "version": "rfc9535"
                                }
                            }
                        }
                    },
                    {
                        "stepId": "emitOrderEvent",
                        "channelPath": "{$sourceDescriptions.petEvents.url}#/channels/~1orderPlaced",
                        "action": "send",
                        "dependsOn": ["findPetStep"],
                        "correlationId": "$message.header#/correlationId"
                    }
                ]
            }
        ]
    })
}

/// Returns the canonical conformance `OpenAPI` 3.2.0 specification JSON value used for
/// cross-language signature and roundtrip verification.
///
/// # Returns
///
/// Canonical `OpenAPI` 3.2.0 `serde_json::Value`.
#[must_use]
pub fn canonical_conformance_openapi_doc() -> serde_json::Value {
    serde_json::json!({
        "openapi": "3.2.0",
        "info": { "title": "CDD Canonical Conformance API", "version": "1.0.0" },
        "paths": {
            "/pets": {
                "get": {
                    "operationId": "listPets",
                    "parameters": [
                        { "name": "limit", "in": "query", "schema": { "type": "integer" } }
                    ]
                }
            }
        },
        "components": {
            "schemas": {
                "Pet": {
                    "type": "object",
                    "properties": {
                        "id": { "type": "integer" },
                        "name": { "type": "string" }
                    }
                }
            }
        }
    })
}

/// Returns the canonical deterministic HTTP request/response corpus covering status codes
/// `200`, `201`, `204`, `400`, `401`, `403`, `404`, `422`, and `501`.
///
/// # Returns
///
/// Vector of `(operation_id, SimpleHttpResponse)` pairs.
#[must_use]
pub(crate) fn canonical_http_corpus() -> Vec<(&'static str, SimpleHttpResponse)> {
    vec![
        (
            "listPets_200",
            SimpleHttpResponse {
                status_code: 200,
                headers: vec![("content-type".to_string(), "application/json".to_string())],
                body: r#"{"items":[{"id":1,"name":"doggie"}],"timestamp":"2026-10-17T00:00:00Z"}"#
                    .to_string(),
            },
        ),
        (
            "createPet_201",
            SimpleHttpResponse {
                status_code: 201,
                headers: vec![
                    ("content-type".to_string(), "application/json".to_string()),
                    ("location".to_string(), "/pets/1".to_string()),
                ],
                body: r#"{"id":1,"name":"doggie","uuid":"00000000-0000-0000-0000-000000000001"}"#
                    .to_string(),
            },
        ),
        (
            "deletePet_204",
            SimpleHttpResponse {
                status_code: 204,
                headers: vec![],
                body: String::new(),
            },
        ),
        (
            "validatePet_400",
            SimpleHttpResponse {
                status_code: 400,
                headers: vec![(
                    "content-type".to_string(),
                    "application/problem+json".to_string(),
                )],
                body: r#"{"pointer":"/name","detail":"name is required"}"#.to_string(),
            },
        ),
        (
            "unauthorized_401",
            SimpleHttpResponse {
                status_code: 401,
                headers: vec![("www-authenticate".to_string(), "Bearer".to_string())],
                body: r#"{"error":"unauthorized"}"#.to_string(),
            },
        ),
        (
            "forbidden_403",
            SimpleHttpResponse {
                status_code: 403,
                headers: vec![("content-type".to_string(), "application/json".to_string())],
                body: r#"{"error":"forbidden"}"#.to_string(),
            },
        ),
        (
            "notFound_404",
            SimpleHttpResponse {
                status_code: 404,
                headers: vec![("content-type".to_string(), "application/json".to_string())],
                body: r#"{"error":"not_found"}"#.to_string(),
            },
        ),
        (
            "unprocessable_422",
            SimpleHttpResponse {
                status_code: 422,
                headers: vec![(
                    "content-type".to_string(),
                    "application/problem+json".to_string(),
                )],
                body: r#"{"pointer":"/id","detail":"must be positive"}"#.to_string(),
            },
        ),
        (
            "stub_501",
            SimpleHttpResponse {
                status_code: 501,
                headers: vec![("content-type".to_string(), "application/json".to_string())],
                body: r#"{"error":"not_implemented"}"#.to_string(),
            },
        ),
    ]
}
