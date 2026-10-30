//! Implementation of the `report` and `matrix` subcommands and `README.md` matrix updating.

use crate::diff::compare_checklists_with_path;
use crate::discovery::{scan_all_repositories, scan_repository};
use crate::error::ConformanceError;
use crate::model::SpecId;
use crate::registry::CanonicalRegistry;
use comfy_table::presets::UTF8_FULL;
use comfy_table::{Cell, Color, Row, Table};
use std::fmt::Write as FmtWrite;
use std::path::Path;

/// Computes the canonical-aware `ProfileScore` for a target `ChecklistTable`.
///
/// # Arguments
///
/// * `target` - Parsed target `ChecklistTable`.
/// * `path` - File path of the target checklist.
/// * `registry` - Reference to the canonical specification registry.
///
/// # Returns
///
/// Evaluated `ProfileScore`.
///
/// # Errors
///
/// Returns `ConformanceError` if the canonical template cannot be loaded.
fn canonical_score(
    target: &crate::model::ChecklistTable,
    path: &Path,
    registry: &CanonicalRegistry,
) -> Result<crate::model::ProfileScore, ConformanceError> {
    let canonical = registry.get_table(target.spec_id, target.profile)?;
    Ok(compare_checklists_with_path(target, &canonical, path).score)
}

/// Executes the `report` subcommand with an optional specification filter.
///
/// # Arguments
///
/// * `repo_path` - Path to the target repository.
/// * `filter_spec` - Optional specification identifier filter.
/// * `json` - Whether to output JSON.
/// * `markdown` - Whether to output Markdown.
/// * `registry` - Reference to the canonical specification registry.
///
/// # Returns
///
/// Formatted report string.
///
/// # Errors
///
/// Returns `ConformanceError` if repository scanning or checklist parsing fails.
pub(crate) fn run_report_filtered(
    repo_path: &Path,
    filter_spec: Option<SpecId>,
    json: bool,
    markdown: bool,
    registry: &CanonicalRegistry,
) -> Result<String, ConformanceError> {
    let project = scan_repository(repo_path)?;
    let tables = project.parse_all_checklists()?;

    let mut filtered = Vec::new();
    for (t, c) in tables.iter().zip(project.checklists.iter()) {
        if filter_spec.is_none_or(|f| t.spec_id == f) {
            filtered.push((t, c));
        }
    }

    if json {
        let mut scores = Vec::new();
        for (t, c) in &filtered {
            let score = canonical_score(t, &c.path, registry)?;
            scores.push(serde_json::json!({
                "spec_id": t.spec_id.as_str(),
                "profile": t.profile.as_str(),
                "title": t.title,
                "score": score,
            }));
        }
        let mut out = crate::error::to_pretty_json(&scores, "report").unwrap_or_default();
        let _ = writeln!(out);
        return Ok(out);
    }

    if markdown {
        let mut out = String::new();
        let _ = writeln!(
            out,
            "# Conformance Report for `{}`
",
            project.language_name
        );
        let _ = writeln!(
            out,
            "| Specification | Profile | To Coverage | From Coverage | Overall | Status |"
        );
        let _ = writeln!(out, "| :--- | :--- | :---: | :---: | :---: | :---: |");
        for (t, c) in &filtered {
            let score = canonical_score(t, &c.path, registry)?;
            let icon = if score.total_percentage() >= 80.0 {
                "✅"
            } else {
                "⚠️"
            };
            let _ = writeln!(
                out,
                "| {} | {} | {:.1}% | {:.1}% | {:.1}% | {} |",
                t.spec_id,
                t.profile,
                score.to_percentage(),
                score.from_percentage(),
                score.total_percentage(),
                icon
            );
        }
        return Ok(out);
    }

    let mut table = Table::new();
    table.load_preset(UTF8_FULL);
    table.set_header(vec![
        "Specification",
        "Profile",
        "To %",
        "From %",
        "Overall %",
        "Items",
    ]);

    for (t, c) in &filtered {
        let score = canonical_score(t, &c.path, registry)?;
        table.add_row(Row::from(vec![
            Cell::new(t.spec_id.as_str()).fg(Color::Cyan),
            Cell::new(t.profile.as_str()),
            Cell::new(format!("{:.1}%", score.to_percentage())),
            Cell::new(format!("{:.1}%", score.from_percentage())),
            Cell::new(format!("{:.1}%", score.total_percentage())).fg(
                if score.total_percentage() >= 80.0 {
                    Color::Green
                } else {
                    Color::Yellow
                },
            ),
            Cell::new(format!(
                "{}/{}",
                score.implemented_to + score.implemented_from,
                score.total_items * 2
            )),
        ]));
    }

    Ok(format!(
        "{table}
"
    ))
}

/// Formats a language name with a GitHub repository hyperlink (e.g. `c ([cdd-c](https://github.com/SamuelMarks/cdd-c))`).
///
/// # Arguments
///
/// * `language_name` - The short language identifier or `cdd-*` repository name.
///
/// # Returns
///
/// Formatted Markdown link string.
#[must_use]
pub fn format_language_link(language_name: &str) -> String {
    let repo_name = if language_name.starts_with("cdd-") {
        language_name.to_string()
    } else {
        format!("cdd-{language_name}")
    };
    format!("{language_name} ([{repo_name}](https://github.com/SamuelMarks/{repo_name}))")
}

/// Executes the `matrix` subcommand and returns the terminal output string.
///
/// # Arguments
///
/// * `repos_dir` - Parent directory containing `cdd-*` repositories.
/// * `output` - Optional file path to write the Markdown matrix to.
/// * `readme` - Optional `README.md` path to update in place.
/// * `registry` - Reference to the canonical specification registry.
///
/// # Returns
///
/// Terminal summary output string.
///
/// # Errors
///
/// Returns `ConformanceError` if scanning, checklist parsing, or file writing fails.
pub(crate) fn run_matrix(
    repos_dir: &Path,
    output: Option<&Path>,
    readme: Option<&Path>,
    registry: &CanonicalRegistry,
) -> Result<String, ConformanceError> {
    let mut out = String::new();
    let projects = scan_all_repositories(repos_dir)?;
    let _ = writeln!(
        out,
        "Scanning {} projects in {}...",
        projects.len(),
        repos_dir.display()
    );

    let mut markdown_lines = vec![
        "# CDD Ecosystem Specification Conformance Matrix
"
        .to_string(),
        "| Language | Specification | Profile | Implemented Features | Overall Score |".to_string(),
        "| :--- | :--- | :--- | :---: | :---: |".to_string(),
    ];

    let mut term_table = Table::new();
    term_table.load_preset(UTF8_FULL);
    term_table.set_header(vec!["Language", "Specification", "Profile", "Score %"]);

    for p in &projects {
        let lang_link = format_language_link(&p.language_name);
        for checklist in &p.checklists {
            let t = checklist.parse()?;
            let score = canonical_score(&t, &checklist.path, registry)?;
            let pct = score.total_percentage();
            term_table.add_row(Row::from(vec![
                Cell::new(&p.language_name).fg(Color::Green),
                Cell::new(t.spec_id.as_str()).fg(Color::Cyan),
                Cell::new(t.profile.as_str()),
                Cell::new(format!("{pct:.1}%")).fg(if pct >= 80.0 {
                    Color::Green
                } else {
                    Color::Yellow
                }),
            ]));

            markdown_lines.push(format!(
                "| {} | {} | {} | {}/{} | **{:.1}%** |",
                lang_link,
                t.spec_id,
                t.profile,
                score.implemented_to + score.implemented_from,
                score.total_items * 2,
                pct
            ));
        }
    }

    let _ = writeln!(out, "{term_table}");

    if let Some(out_path) = output {
        let md_content = markdown_lines.join(
            "
",
        ) + "
";
        let out_err = |source: std::io::Error| ConformanceError::Io {
            source,
            path: out_path.to_path_buf(),
        };
        std::fs::write(out_path, md_content).map_err(out_err)?;
        let _ = writeln!(out, "Matrix written to {}", out_path.display());
    }

    if let Some(readme_path) = readme {
        let table_only = markdown_lines[1..].join(
            "
",
        );
        let msg = update_readme_with_table(readme_path, &table_only)?;
        out.push_str(&msg);
    }
    Ok(out)
}

/// Updates a README markdown file with the generated ecosystem conformance table.
///
/// Looks for `<!-- ECOSYSTEM_CONFORMANCE_START -->` and `<!-- ECOSYSTEM_CONFORMANCE_END -->` tags.
/// If not found (or if tags are out of order), inserts the section before `## Integration Checklists Overview`.
///
/// # Arguments
///
/// * `readme_path` - Path to the `README.md` file.
/// * `table_content` - Generated Markdown table string.
///
/// # Returns
///
/// Confirmation message string.
///
/// # Errors
///
/// Returns `ConformanceError::Io` if reading or writing `readme_path` fails.
pub(crate) fn update_readme_with_table(
    readme_path: &Path,
    table_content: &str,
) -> Result<String, ConformanceError> {
    let to_io_err = |source: std::io::Error| ConformanceError::Io {
        source,
        path: readme_path.to_path_buf(),
    };
    let content = std::fs::read_to_string(readme_path).map_err(to_io_err)?;

    let start_tag = "<!-- ECOSYSTEM_CONFORMANCE_START -->";
    let end_tag = "<!-- ECOSYSTEM_CONFORMANCE_END -->";

    let valid_tags = content
        .find(start_tag)
        .zip(content.find(end_tag))
        .filter(|(s, e)| *s + start_tag.len() <= *e);

    let new_content = if let Some((start_idx, end_idx)) = valid_tags {
        let prefix = &content[..start_idx + start_tag.len()];
        let suffix = &content[end_idx..];
        format!(
            "{prefix}

{table_content}

{suffix}"
        )
    } else {
        let section = format!(
            "## Ecosystem Conformance Matrix

The table below tracks live specification compliance across all 13 core language repositories in the CDD ecosystem (`cdd-c`, `cdd-cpp`, `cdd-csharp`, `cdd-go`, `cdd-java`, `cdd-kotlin`, `cdd-php`, `cdd-python-all`, `cdd-ruby`, `cdd-rust`, `cdd-sh`, `cdd-swift`, `cdd-ts`):

{start_tag}

{table_content}

{end_tag}

---

"
        );

        content
            .find("## Integration Checklists Overview")
            .map_or_else(
                || {
                    format!(
                        "{content}

{section}"
                    )
                },
                |target_idx| {
                    format!(
                        "{}{}{}",
                        &content[..target_idx],
                        section,
                        &content[target_idx..]
                    )
                },
            )
    };

    std::fs::write(readme_path, new_content).map_err(to_io_err)?;

    Ok(format!(
        "Updated {} with ecosystem conformance matrix.
",
        readme_path.display()
    ))
}
