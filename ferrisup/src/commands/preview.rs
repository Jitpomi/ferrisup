use crate::project::templates::{find_template_directory, list_templates};
use anyhow::{Result, bail};
use colored::Colorize;
use dialoguer::Select;
use serde_json::Value;
use std::fs;
use std::path::{Path, PathBuf};
use walkdir::WalkDir;

pub fn execute(
    component_type: Option<&str>,
    framework: Option<&str>,
    provider: Option<&str>,
    application_type: Option<&str>,
) -> Result<()> {
    let component = select_component(component_type)?;
    let selection = PreviewSelection {
        component: component.clone(),
        framework,
        provider,
        application_type,
    };
    let template_name = selection.template_name()?;
    let template_dir = find_template_directory(&template_name)?;
    let metadata = read_metadata(&template_dir)?;

    println!("{}", "FerrisUp template preview".bold().green());
    println!("Component: {}", component.cyan());
    if template_name != component {
        println!("Template: {}", template_name.cyan());
    }
    if let Some(description) = metadata.get("description").and_then(Value::as_str) {
        println!("Description: {description}");
    }

    println!("\n{}", "Files:".bold());
    for path in template_files(&template_dir)? {
        println!("  {}", display_target_path(&path));
    }

    if let Some(steps) = metadata.get("next_steps").and_then(Value::as_array) {
        println!("\n{}", "Template next steps:".bold());
        for step in steps.iter().filter_map(Value::as_str) {
            println!("  - {step}");
        }
    }

    println!("\nPreview reads the bundled template and does not create or modify project files.");
    Ok(())
}

struct PreviewSelection<'a> {
    component: String,
    framework: Option<&'a str>,
    provider: Option<&'a str>,
    application_type: Option<&'a str>,
}

impl PreviewSelection<'_> {
    fn template_name(&self) -> Result<String> {
        let name = match self.component.as_str() {
            "server" => format!("server/{}", required("--framework", self.framework)?),
            "serverless" => {
                format!("serverless/{}", required("--provider", self.provider)?)
            }
            "data-science" => match required("--framework", self.framework)? {
                "polars" | "polars-cli" => "data-science/polars-cli".to_string(),
                "linfa" | "linfa-examples" => "data-science/linfa-examples".to_string(),
                value => bail!("Unsupported data-science framework '{value}'"),
            },
            "client" => match required("--framework", self.framework)? {
                "leptos" => "client/leptos".to_string(),
                "dioxus" | "tauri" => bail!(
                    "{} projects are created by their official CLI and have no bundled template to preview",
                    self.framework.unwrap_or_default()
                ),
                value => bail!("Unsupported client framework '{value}'"),
            },
            "edge" => {
                let application = required("--application-type", self.application_type)?;
                let provider = required("--provider", self.provider)?;
                format!("edge/{application}/{provider}")
            }
            "shared" => bail!(
                "Shared components are created from workspace context and have no standalone template"
            ),
            _ => self.component.clone(),
        };
        Ok(name)
    }
}

fn required<'a>(flag: &str, value: Option<&'a str>) -> Result<&'a str> {
    value.ok_or_else(|| anyhow::anyhow!("{flag} is required for this component preview"))
}

fn select_component(component_type: Option<&str>) -> Result<String> {
    if let Some(component) = component_type {
        return Ok(component.to_string());
    }

    let templates = list_templates()?;
    let labels: Vec<String> = templates
        .iter()
        .map(|(name, description)| format!("{name} - {description}"))
        .collect();
    let index = Select::new()
        .with_prompt("Select a component type to preview")
        .items(&labels)
        .default(0)
        .interact()?;
    Ok(templates[index].0.clone())
}

fn read_metadata(template_dir: &Path) -> Result<Value> {
    let path = template_dir.join("template.json");
    let content = fs::read_to_string(&path)?;
    Ok(serde_json::from_str(&content)?)
}

fn template_files(template_dir: &Path) -> Result<Vec<PathBuf>> {
    let mut files = WalkDir::new(template_dir)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .filter(|entry| entry.file_type().is_file())
        .filter_map(|entry| {
            entry
                .path()
                .strip_prefix(template_dir)
                .ok()
                .map(Path::to_path_buf)
        })
        .filter(|path| path != Path::new("template.json"))
        .collect::<Vec<_>>();
    files.sort();
    Ok(files)
}

fn display_target_path(path: &Path) -> String {
    let value = path.to_string_lossy();
    value
        .strip_suffix(".template")
        .unwrap_or(&value)
        .to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_framework_and_provider_templates() {
        let server = PreviewSelection {
            component: "server".into(),
            framework: Some("axum"),
            provider: None,
            application_type: None,
        };
        assert_eq!(server.template_name().unwrap(), "server/axum");

        let edge = PreviewSelection {
            component: "edge".into(),
            framework: None,
            provider: Some("cloudflare-workers"),
            application_type: Some("api-function"),
        };
        assert_eq!(
            edge.template_name().unwrap(),
            "edge/api-function/cloudflare-workers"
        );
    }

    #[test]
    fn rejects_external_cli_preview() {
        let selection = PreviewSelection {
            component: "client".into(),
            framework: Some("dioxus"),
            provider: None,
            application_type: None,
        };
        assert!(selection.template_name().is_err());
    }

    #[test]
    fn removes_template_suffix_from_display_path() {
        assert_eq!(
            display_target_path(Path::new("Cargo.toml.template")),
            "Cargo.toml"
        );
    }
}
