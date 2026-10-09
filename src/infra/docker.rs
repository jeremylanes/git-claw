//! Docker Compose override generator and parser for worktree interoperability.
//!
//! Generates `docker-compose.claw.override.yml` to prevent container name
//! collisions and configure shared service/network interop.

use crate::core::config::DockerConfig;
use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

/// A service parsed from a Compose YAML file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParsedService {
    pub name: String,
    pub container_name: Option<String>,
}

/// Parses service names and their container_name attributes from Compose YAML content.
///
/// # Example
///
/// ```
/// use git_claw::infra::docker::parse_compose_services;
///
/// let yaml = "services:\n  web:\n    container_name: my_web\n";
/// let services = parse_compose_services(yaml);
/// assert_eq!(services.len(), 1);
/// assert_eq!(services[0].name, "web");
/// assert_eq!(services[0].container_name.as_deref(), Some("my_web"));
/// ```
pub fn parse_compose_services(compose_content: &str) -> Vec<ParsedService> {
    let mut services = Vec::new();
    let mut in_services = false;
    let mut services_indent = 0;
    let mut current_service: Option<ParsedService> = None;
    let mut service_indent_level: Option<usize> = None;

    for line in compose_content.lines() {
        let trimmed_end = line.trim_end();
        if trimmed_end.is_empty() {
            continue;
        }
        let trimmed = trimmed_end.trim();
        if trimmed.starts_with('#') {
            continue;
        }

        let indent = line.len() - line.trim_start().len();

        if !in_services {
            if trimmed == "services:" || trimmed.starts_with("services:") {
                in_services = true;
                services_indent = indent;
            }
            continue;
        }

        // Check if we exited the services block
        if indent <= services_indent && !trimmed.is_empty() {
            in_services = false;
            if let Some(svc) = current_service.take() {
                services.push(svc);
            }
            continue;
        }

        // Inside services block:
        if let Some(expected_level) = service_indent_level {
            if indent == expected_level && trimmed.ends_with(':') {
                if let Some(svc) = current_service.take() {
                    services.push(svc);
                }
                let name = trimmed
                    .trim_end_matches(':')
                    .trim()
                    .trim_matches(|c| c == '\'' || c == '"');
                current_service = Some(ParsedService {
                    name: name.to_string(),
                    container_name: None,
                });
                continue;
            } else if indent > expected_level {
                if let Some(stripped) = trimmed.strip_prefix("container_name:") {
                    let val = stripped.trim().trim_matches(|c| c == '\'' || c == '"');
                    if let Some(ref mut svc) = current_service {
                        svc.container_name = Some(val.to_string());
                    }
                }
                continue;
            }
        }

        // First service encountered
        if indent > services_indent && trimmed.ends_with(':') {
            service_indent_level = Some(indent);
            let name = trimmed
                .trim_end_matches(':')
                .trim()
                .trim_matches(|c| c == '\'' || c == '"');
            current_service = Some(ParsedService {
                name: name.to_string(),
                container_name: None,
            });
        }
    }

    if let Some(svc) = current_service {
        services.push(svc);
    }

    services
}

/// Generates the content of `docker-compose.claw.override.yml`.
///
/// # Example
///
/// ```
/// use git_claw::core::config::DockerConfig;
/// use git_claw::infra::docker::{generate_override_content, ParsedService};
///
/// let config = DockerConfig {
///     compose_file: None,
///     network: Some("shared_net".to_string()),
///     shared_services: vec!["db".to_string()],
/// };
/// let services = vec![ParsedService {
///     name: "web".to_string(),
///     container_name: Some("web_app".to_string()),
/// }];
/// let content = generate_override_content(&services, &config);
/// assert!(content.contains("external: true"));
/// assert!(content.contains("container_name: !reset"));
/// ```
/// Parses items under a top-level YAML section (e.g. `networks:` or `volumes:`).
/// Excludes items that are already marked `external: true`.
fn parse_top_level_items(compose_content: &str, section_header: &str) -> Vec<String> {
    let mut items = Vec::new();
    let mut in_section = false;
    let mut section_indent = 0;
    let mut current_item: Option<String> = None;
    let mut current_item_is_external = false;
    let mut item_indent_level: Option<usize> = None;

    for line in compose_content.lines() {
        let trimmed_end = line.trim_end();
        if trimmed_end.is_empty() {
            continue;
        }
        let trimmed = trimmed_end.trim();
        if trimmed.starts_with('#') {
            continue;
        }

        let indent = line.len() - line.trim_start().len();

        if !in_section {
            if trimmed == section_header || trimmed.starts_with(section_header) {
                in_section = true;
                section_indent = indent;
            }
            continue;
        }

        // Check if we exited the section
        if indent <= section_indent && !trimmed.is_empty() {
            if let Some(item) = current_item.take() {
                if !current_item_is_external {
                    items.push(item);
                }
            }
            break;
        }

        // Inside section
        if let Some(expected_level) = item_indent_level {
            if indent == expected_level && trimmed.ends_with(':') {
                if let Some(item) = current_item.take() {
                    if !current_item_is_external {
                        items.push(item);
                    }
                }
                let name = trimmed
                    .trim_end_matches(':')
                    .trim()
                    .trim_matches(|c| c == '\'' || c == '"');
                current_item = Some(name.to_string());
                current_item_is_external = false;
                continue;
            } else if indent > expected_level {
                if trimmed.contains("external: true") || trimmed == "external: true" {
                    current_item_is_external = true;
                }
                continue;
            }
        }

        if indent > section_indent && trimmed.ends_with(':') {
            item_indent_level = Some(indent);
            let name = trimmed
                .trim_end_matches(':')
                .trim()
                .trim_matches(|c| c == '\'' || c == '"');
            current_item = Some(name.to_string());
            current_item_is_external = false;
        }
    }

    if let Some(item) = current_item {
        if !current_item_is_external {
            items.push(item);
        }
    }

    items
}

/// Parses network names defined under top-level `networks:` that are not already marked external.
pub fn parse_compose_internal_networks(compose_content: &str) -> Vec<String> {
    parse_top_level_items(compose_content, "networks:")
}

/// Parses volume names defined under top-level `volumes:` that are not already marked external.
pub fn parse_compose_volumes(compose_content: &str) -> Vec<String> {
    parse_top_level_items(compose_content, "volumes:")
}

/// Generates the content of `docker-compose.claw.override.yml`, including external networks and volumes.
pub fn generate_override_content_full(
    services: &[ParsedService],
    config: &DockerConfig,
    extra_networks: &[String],
    extra_volumes: &[String],
) -> String {
    let mut out = String::from("# Generated by git-claw\n");

    let shared_set: BTreeSet<&str> = config.shared_services.iter().map(|s| s.as_str()).collect();

    let mut service_lines = Vec::new();

    // 1. Shared services: neutralized without breaking depends_on
    for svc_name in &config.shared_services {
        service_lines.push(format!("  {svc_name}:"));
        service_lines.push("    container_name: !reset".to_string());
        service_lines.push("    restart: \"no\"".to_string());
        service_lines.push("    scale: 0".to_string());
        service_lines.push("    entrypoint: [\"true\"]".to_string());
    }

    // 2. Worktree-specific services that need container_name neutralization or network attachment
    for svc in services {
        if shared_set.contains(svc.name.as_str()) {
            continue;
        }

        let needs_neutralization = svc.container_name.is_some();
        let needs_network = config.network.is_some();

        if needs_neutralization || needs_network {
            service_lines.push(format!("  {}:", svc.name));
            if needs_neutralization {
                service_lines.push("    container_name: !reset".to_string());
            }
            if let Some(ref net) = config.network {
                service_lines.push("    networks:".to_string());
                service_lines.push(format!("      - {net}"));
            }
        }
    }

    if !service_lines.is_empty() {
        out.push_str("services:\n");
        for line in service_lines {
            out.push_str(&line);
            out.push('\n');
        }
    }

    // Networks
    let mut all_networks = BTreeSet::new();
    if let Some(ref net) = config.network {
        all_networks.insert(net.clone());
    }
    for net in extra_networks {
        all_networks.insert(net.clone());
    }

    if !all_networks.is_empty() {
        out.push_str("networks:\n");
        for net in &all_networks {
            out.push_str(&format!("  {net}:\n"));
            out.push_str("    external: true\n");
        }
    }

    // Volumes
    let mut all_volumes = BTreeSet::new();
    for vol in extra_volumes {
        all_volumes.insert(vol.clone());
    }

    if !all_volumes.is_empty() {
        out.push_str("volumes:\n");
        for vol in &all_volumes {
            out.push_str(&format!("  {vol}:\n"));
            out.push_str("    external: true\n");
        }
    }

    out
}

/// Generates the content of `docker-compose.claw.override.yml`.
///
/// # Example
///
/// ```
/// use git_claw::core::config::DockerConfig;
/// use git_claw::infra::docker::{generate_override_content, ParsedService};
///
/// let config = DockerConfig {
///     compose_file: None,
///     network: Some("shared_net".to_string()),
///     shared_services: vec!["db".to_string()],
/// };
/// let services = vec![ParsedService {
///     name: "web".to_string(),
///     container_name: Some("web_app".to_string()),
/// }];
/// let content = generate_override_content(&services, &config);
/// assert!(content.contains("external: true"));
/// assert!(content.contains("container_name: !reset"));
/// ```
pub fn generate_override_content(services: &[ParsedService], config: &DockerConfig) -> String {
    generate_override_content_full(services, config, &[], &[])
}

/// Generates `docker-compose.claw.override.yml` in the given worktree directory
/// if Docker configuration is enabled.
///
/// Returns `Ok(Some(path))` if generated, or `Ok(None)` if Docker configuration is disabled.
///
/// # Example
///
/// ```
/// use git_claw::core::config::DockerConfig;
/// use git_claw::infra::docker::generate_docker_compose_override;
/// use tempfile::tempdir;
///
/// let dir = tempdir().unwrap();
/// let config = DockerConfig {
///     compose_file: None,
///     network: Some("net".to_string()),
///     shared_services: vec!["postgres".to_string()],
/// };
/// let result = generate_docker_compose_override(dir.path(), dir.path(), &config).unwrap();
/// assert!(result.is_some());
/// ```
pub fn generate_docker_compose_override(
    toplevel_path: &Path,
    worktree_path: &Path,
    config: &DockerConfig,
) -> Result<Option<PathBuf>, std::io::Error> {
    if !config.is_enabled() {
        return Ok(None);
    }

    let candidates = [
        "docker-compose.yml",
        "docker-compose.yaml",
        "compose.yml",
        "compose.yaml",
    ];

    let compose_path = if let Some(ref custom_file) = config.compose_file {
        let p = worktree_path.join(custom_file);
        if p.exists() {
            Some(p)
        } else {
            let top_p = toplevel_path.join(custom_file);
            if top_p.exists() {
                Some(top_p)
            } else {
                None
            }
        }
    } else {
        candidates
            .iter()
            .map(|name| worktree_path.join(name))
            .find(|p| p.exists())
            .or_else(|| {
                candidates
                    .iter()
                    .map(|name| toplevel_path.join(name))
                    .find(|p| p.exists())
            })
    };

    let (services, extra_networks, extra_volumes) = if let Some(ref p) = compose_path {
        let content = fs::read_to_string(p).unwrap_or_default();
        let svcs = parse_compose_services(&content);
        let nets = parse_compose_internal_networks(&content);
        let vols = parse_compose_volumes(&content);
        (svcs, nets, vols)
    } else {
        (Vec::new(), Vec::new(), Vec::new())
    };

    let content = generate_override_content_full(&services, config, &extra_networks, &extra_volumes);
    let target = worktree_path.join("docker-compose.override.yml");
    fs::write(&target, &content)?;

    // If custom compose_file is in a subdirectory, also write it in that subdirectory
    if let Some(ref custom_file) = config.compose_file {
        if let Some(parent) = Path::new(custom_file).parent() {
            if parent != Path::new("") {
                let sub_target = worktree_path
                    .join(parent)
                    .join("docker-compose.override.yml");
                let _ = fs::write(sub_target, &content);
            }
        }
    }

    Ok(Some(target))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_compose_networks_and_volumes() {
        let yaml = r#"
version: '3.8'

services:
  web:
    image: web
networks:
  tune_network:
    name: tune_network
  external_net:
    external: true
volumes:
  postgres_data:
    name: tune_postgres_data
  external_vol:
    external: true
"#;

        let nets = parse_compose_internal_networks(yaml);
        assert_eq!(nets, vec!["tune_network"]);

        let vols = parse_compose_volumes(yaml);
        assert_eq!(vols, vec!["postgres_data"]);
    }

    #[test]
    fn test_generate_override_content_full_with_networks_and_volumes() {
        let config = DockerConfig {
            compose_file: None,
            network: None,
            shared_services: vec!["postgres".to_string()],
        };
        let services = vec![ParsedService {
            name: "web".to_string(),
            container_name: Some("web_app".to_string()),
        }];
        let nets = vec!["custom_network".to_string()];
        let vols = vec!["custom_data".to_string()];

        let content = generate_override_content_full(&services, &config, &nets, &vols);
        assert!(content.contains("container_name: !reset"));
        assert!(content.contains("networks:\n  custom_network:\n    external: true"));
        assert!(content.contains("volumes:\n  custom_data:\n    external: true"));
    }
}
