//! Project initialization workflow (`git claw init`).
//!
//! Scans repository for environment files, compose files, and port variables,
//! and generates a tailored `.git-claw.toml`.

use crate::cli::output::print_success;
use crate::core::config::DockerConfig;
use crate::infra::docker::parse_compose_services;
use crate::infra::git::resolve_toplevel;
use crate::workflow::error::WorkflowError;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Options for repository initialization.
pub struct InitOptions<'a> {
    pub yes: bool,
    pub repo_root: Option<&'a Path>,
}

/// Recursively scans directory up to depth 3 for `.env` files.
fn scan_env_files(root: &Path) -> Vec<String> {
    let mut results = Vec::new();

    fn visit_dirs(root: &Path, dir: &Path, depth: usize, results: &mut Vec<String>) {
        if depth > 3 {
            return;
        }
        let entries = match fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };

        for entry in entries.flatten() {
            let path = entry.path();
            let file_name = entry.file_name();
            let name_str = file_name.to_string_lossy();

            if name_str.starts_with('.') && name_str != ".env" {
                continue;
            }
            if name_str == "target" || name_str == "node_modules" || name_str == "vendor" {
                continue;
            }

            if path.is_dir() {
                if name_str == "media" || name_str == "uploads" || name_str == "storage" {
                    if let Ok(rel) = path.strip_prefix(root) {
                        results.push(rel.to_string_lossy().to_string());
                    }
                } else {
                    visit_dirs(root, &path, depth + 1, results);
                }
            } else if path.is_file() && (name_str == ".env" || name_str.ends_with(".env")) {
                if let Ok(rel) = path.strip_prefix(root) {
                    results.push(rel.to_string_lossy().to_string());
                }
            }
        }
    }

    visit_dirs(root, root, 0, &mut results);
    results.sort();
    results
}

/// Scans for standard Compose file locations.
fn scan_compose_file(root: &Path) -> Option<String> {
    let candidates = [
        "docker-compose.yml",
        "docker-compose.yaml",
        "compose.yml",
        "compose.yaml",
        "deploy/docker-compose.yml",
        "docker/docker-compose.yml",
    ];

    for c in &candidates {
        if root.join(c).is_file() {
            return Some((*c).to_string());
        }
    }
    None
}

/// Returns true if a key name is a client connection port to a database/message broker/mail service,
/// which must NOT be shifted by slot offsets.
fn is_client_service_port(key: &str) -> bool {
    let upper = key.to_ascii_uppercase();
    upper.contains("DATABASE")
        || upper.contains("DB_")
        || upper.starts_with("DB")
        || upper.contains("POSTGRES")
        || upper.contains("PGPORT")
        || upper.contains("MYSQL")
        || upper.contains("MARIADB")
        || upper.contains("MONGO")
        || upper.contains("REDIS")
        || upper.contains("EMAIL")
        || upper.contains("SMTP")
        || upper.contains("MAIL")
        || upper.contains("AMQP")
        || upper.contains("RABBITMQ")
}

/// Extracts port definitions from `.env` files.
fn extract_ports_from_env_files(root: &Path, env_files: &[String]) -> BTreeMap<String, u16> {
    let mut ports = BTreeMap::new();

    for rel_path in env_files {
        let full_path = root.join(rel_path);
        let content = match fs::read_to_string(&full_path) {
            Ok(c) => c,
            Err(_) => continue,
        };

        for line in content.lines() {
            let trimmed = line.trim();
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }

            if let Some((key, val)) = trimmed.split_once('=') {
                let key = key.trim();
                let val = val.trim().trim_matches(|c| c == '\'' || c == '"');

                if (key == "PORT" || key.contains("PORT")) && !is_client_service_port(key) {
                    if let Ok(port) = val.parse::<u16>() {
                        if port > 0 {
                            ports.insert(key.to_string(), port);
                        }
                    }
                }
            }
        }
    }

    ports
}

/// Scans Compose file for shared services and port definitions.
fn scan_compose_details(
    root: &Path,
    compose_file: &str,
    ports: &mut BTreeMap<String, u16>,
) -> DockerConfig {
    let mut docker = DockerConfig {
        compose_file: Some(compose_file.to_string()),
        network: None,
        shared_services: Vec::new(),
    };

    let content = match fs::read_to_string(root.join(compose_file)) {
        Ok(c) => c,
        Err(_) => return docker,
    };

    let services = parse_compose_services(&content);
    let common_shared = [
        "postgres",
        "postgresql",
        "db",
        "database",
        "redis",
        "mysql",
        "mariadb",
        "mongo",
        "mongodb",
        "rabbitmq",
    ];

    for svc in services {
        if common_shared.contains(&svc.name.as_str()) {
            docker.shared_services.push(svc.name);
        }
    }

    // Scan for port interpolation syntax in compose: ${PORT_VAR:-8000}
    for line in content.lines() {
        if let Some(start_idx) = line.find("${") {
            let rest = &line[start_idx + 2..];
            if let Some(end_idx) = rest.find('}') {
                let var_expr = &rest[..end_idx];
                if let Some((var_name, default_val)) = var_expr.split_once(":-") {
                    if (var_name == "PORT" || var_name.contains("PORT"))
                        && !is_client_service_port(var_name)
                        && !ports.contains_key(var_name)
                    {
                        if let Ok(port) = default_val.parse::<u16>() {
                            ports.insert(var_name.to_string(), port);
                        }
                    }
                }
            }
        }
    }

    docker.network = scan_compose_network(&content);

    docker
}

/// Scans Compose file for defined network name.
fn scan_compose_network(content: &str) -> Option<String> {
    let mut in_networks = false;
    let mut networks_indent = 0;
    let mut current_net: Option<String> = None;
    let mut net_indent_level: Option<usize> = None;

    for line in content.lines() {
        let trimmed_end = line.trim_end();
        if trimmed_end.is_empty() || trimmed_end.trim_start().starts_with('#') {
            continue;
        }
        let indent = line.len() - line.trim_start().len();
        let trimmed = trimmed_end.trim();

        if !in_networks {
            if trimmed == "networks:" || trimmed.starts_with("networks:") {
                in_networks = true;
                networks_indent = indent;
            }
            continue;
        }

        if indent <= networks_indent && !trimmed.is_empty() {
            break;
        }

        if let Some(expected_level) = net_indent_level {
            if indent == expected_level && trimmed.ends_with(':') {
                if let Some(net) = current_net {
                    if net != "default" {
                        return Some(net);
                    }
                }
                let name = trimmed
                    .trim_end_matches(':')
                    .trim()
                    .trim_matches(|c| c == '\'' || c == '"');
                current_net = Some(name.to_string());
                continue;
            } else if indent > expected_level {
                if let Some(stripped) = trimmed.strip_prefix("name:") {
                    let val = stripped.trim().trim_matches(|c| c == '\'' || c == '"');
                    if !val.is_empty() {
                        return Some(val.to_string());
                    }
                }
                continue;
            }
        }

        if indent > networks_indent && trimmed.ends_with(':') {
            net_indent_level = Some(indent);
            let name = trimmed
                .trim_end_matches(':')
                .trim()
                .trim_matches(|c| c == '\'' || c == '"');
            current_net = Some(name.to_string());
        }
    }

    if let Some(net) = current_net {
        if net != "default" {
            return Some(net);
        }
    }

    None
}

/// Generates tailored `.git-claw.toml` content.
pub fn generate_init_toml(
    main_branch: &str,
    ports: &BTreeMap<String, u16>,
    files: &[String],
    docker: &DockerConfig,
) -> String {
    let mut toml = String::new();
    toml.push_str("[project]\n");
    toml.push_str(&format!("main_branch = \"{}\"\n", main_branch));

    if !ports.is_empty() {
        toml.push_str("\n[ports]\n");
        for (k, v) in ports {
            toml.push_str(&format!("{} = {}\n", k, v));
        }
    }

    if !files.is_empty() {
        toml.push_str("\n[files]\n");
        toml.push_str("copy = [\n");
        for f in files {
            toml.push_str(&format!("    \"{}\",\n", f));
        }
        toml.push_str("]\n");
    }

    if docker.is_enabled() {
        toml.push_str("\n[docker]\n");
        if let Some(ref cf) = docker.compose_file {
            toml.push_str(&format!("compose_file = \"{}\"\n", cf));
        }
        if let Some(ref net) = docker.network {
            toml.push_str(&format!("network = \"{}\"\n", net));
        }
        if !docker.shared_services.is_empty() {
            let svcs = docker
                .shared_services
                .iter()
                .map(|s| format!("\"{}\"", s))
                .collect::<Vec<_>>()
                .join(", ");
            toml.push_str(&format!("shared_services = [{}]\n", svcs));
        }
    }

    toml
}

/// Runs the repository initialization workflow.
///
/// # Example
///
/// ```
/// use git_claw::workflow::init::{run_init_workflow, InitOptions};
/// use tempfile::tempdir;
///
/// let dir = tempdir().unwrap();
/// let opts = InitOptions { yes: true, repo_root: Some(dir.path()) };
/// let _ = run_init_workflow(opts);
/// ```
pub fn run_init_workflow(options: InitOptions) -> Result<PathBuf, WorkflowError> {
    let root = resolve_toplevel(options.repo_root)?;
    let target = root.join(".git-claw.toml");

    let env_files = scan_env_files(&root);
    let mut ports = extract_ports_from_env_files(&root, &env_files);

    let docker = if let Some(compose_file) = scan_compose_file(&root) {
        scan_compose_details(&root, &compose_file, &mut ports)
    } else {
        DockerConfig::default()
    };

    let content = generate_init_toml("main", &ports, &env_files, &docker);
    fs::write(&target, content)?;

    print_success(format!(
        "Initialized .git-claw.toml configuration at {}",
        target.display()
    ));

    Ok(target)
}
