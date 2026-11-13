//! Deterministic port calculation and environment variable key formatting.

use crate::core::error::PortError;
use std::collections::BTreeMap;

/// Converts a port key name to an uppercase environment variable name.
///
/// Keys already ending with `_PORT`, starting with `PORT_`, or equal to `PORT`
/// are preserved as-is. Otherwise, `PORT_` is prepended.
///
/// # Example
///
/// ```
/// use git_claw::core::port::format_port_env_key;
///
/// assert_eq!(format_port_env_key("APP_PORT"), "APP_PORT");
/// assert_eq!(format_port_env_key("web"), "PORT_WEB");
/// assert_eq!(format_port_env_key("PORT"), "PORT");
/// ```
pub fn format_port_env_key(key: &str) -> String {
    let sanitized: String = key
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect();
    let upper = sanitized.to_uppercase();
    if upper == "PORT" || upper.starts_with("PORT_") || upper.ends_with("_PORT") {
        upper
    } else {
        format!("PORT_{}", upper)
    }
}

/// Filters configured ports to exclude client ports for shared Docker services.
///
/// Client ports targeting shared containers (such as Postgres or Redis) must remain
/// at their standard port numbers rather than shifting per worktree slot.
///
/// # Examples
///
/// ```
/// use git_claw::core::port::filter_shiftable_ports;
/// use std::collections::BTreeMap;
///
/// let mut ports = BTreeMap::new();
/// ports.insert("web".to_string(), 8000);
/// ports.insert("DATABASE_PORT".to_string(), 5432);
/// let shared = vec!["postgres".to_string()];
/// let shiftable = filter_shiftable_ports(&ports, &shared);
/// assert_eq!(shiftable.get("web"), Some(&8000));
/// assert!(!shiftable.contains_key("DATABASE_PORT"));
/// ```
pub fn filter_shiftable_ports(
    base_ports: &BTreeMap<String, u16>,
    shared_services: &[String],
) -> BTreeMap<String, u16> {
    base_ports
        .iter()
        .filter(|(k, _)| !is_shared_service_port(k, shared_services))
        .map(|(k, &v)| (k.clone(), v))
        .collect()
}

/// Calculates effective ports: `Effective Port = Base Port + Slot ID`.
/// Returns a map of environment key -> effective port, or PortError on overflow.
pub fn calculate_effective_ports(
    base_ports: &BTreeMap<String, u16>,
    slot_id: u32,
) -> Result<BTreeMap<String, u16>, PortError> {
    let mut result = BTreeMap::new();
    for (key, &base) in base_ports {
        let total = (base as u32)
            .checked_add(slot_id)
            .ok_or_else(|| PortError::PortOverflow {
                key: key.clone(),
                base_port: base,
                slot_id,
            })?;

        if total > u16::MAX as u32 {
            return Err(PortError::PortOverflow {
                key: key.clone(),
                base_port: base,
                slot_id,
            });
        }

        result.insert(format_port_env_key(key), total as u16);
    }
    Ok(result)
}

/// Returns true if a port key represents a client connection to a shared service.
///
/// Ports for shared services (e.g. `DATABASE_PORT` for a shared `postgres` container)
/// must connect to the shared container on its standard port and not be shifted by slot ID.
///
/// # Example
///
/// ```
/// use git_claw::core::port::is_shared_service_port;
///
/// let shared = vec!["postgres".to_string(), "redis".to_string()];
/// assert!(is_shared_service_port("DATABASE_PORT", &shared));
/// assert!(is_shared_service_port("REDIS_PORT", &shared));
/// assert!(!is_shared_service_port("APP_PORT", &shared));
/// ```
pub fn is_shared_service_port(port_name: &str, shared_services: &[String]) -> bool {
    let upper = port_name.to_ascii_uppercase();
    for svc in shared_services {
        let svc_upper = svc.to_ascii_uppercase();
        if (svc_upper == "POSTGRES"
            || svc_upper == "POSTGRESQL"
            || svc_upper == "DB"
            || svc_upper == "DATABASE")
            && (upper.contains("DATABASE")
                || upper.contains("POSTGRES")
                || upper.contains("PGPORT")
                || upper.contains("DB_")
                || upper.starts_with("DB"))
        {
            return true;
        }
        if (svc_upper == "REDIS") && upper.contains("REDIS") {
            return true;
        }
        if (svc_upper == "MYSQL" || svc_upper == "MARIADB")
            && (upper.contains("MYSQL") || upper.contains("MARIADB"))
        {
            return true;
        }
        if (svc_upper == "MONGO" || svc_upper == "MONGODB") && upper.contains("MONGO") {
            return true;
        }
        if (svc_upper == "RABBITMQ" || svc_upper == "AMQP")
            && (upper.contains("RABBITMQ") || upper.contains("AMQP"))
        {
            return true;
        }
    }
    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_port_env_key() {
        assert_eq!(format_port_env_key("web"), "PORT_WEB");
        assert_eq!(format_port_env_key("APP_PORT"), "APP_PORT");
        assert_eq!(format_port_env_key("PORT"), "PORT");
        assert_eq!(format_port_env_key("port_http"), "PORT_HTTP");
        assert_eq!(format_port_env_key("frontend-app"), "PORT_FRONTEND_APP");
        assert_eq!(format_port_env_key("db_primary"), "PORT_DB_PRIMARY");
    }

    #[test]
    fn test_calculate_effective_ports_ok() {
        let mut base_ports = BTreeMap::new();
        base_ports.insert("web".to_string(), 3000);
        base_ports.insert("api".to_string(), 8000);

        let effective = calculate_effective_ports(&base_ports, 2).expect("calculates ports");
        assert_eq!(effective.get("PORT_WEB"), Some(&3002));
        assert_eq!(effective.get("PORT_API"), Some(&8002));
    }

    #[test]
    fn test_calculate_effective_ports_overflow() {
        let mut base_ports = BTreeMap::new();
        base_ports.insert("overflow".to_string(), 65535);

        let result = calculate_effective_ports(&base_ports, 1);
        assert!(matches!(result, Err(PortError::PortOverflow { .. })));
    }

    #[test]
    fn test_is_shared_service_port() {
        let shared = vec!["postgres".to_string(), "redis".to_string()];
        assert!(is_shared_service_port("DATABASE_PORT", &shared));
        assert!(is_shared_service_port("TEST_DATABASE_PORT", &shared));
        assert!(is_shared_service_port("DB_PORT", &shared));
        assert!(is_shared_service_port("POSTGRES_PORT", &shared));
        assert!(is_shared_service_port("PGPORT", &shared));
        assert!(is_shared_service_port("REDIS_PORT", &shared));
        assert!(!is_shared_service_port("APP_PORT", &shared));
        assert!(!is_shared_service_port("FLOWER_PORT", &shared));
        assert!(!is_shared_service_port("HTTP_PORT", &shared));
    }
}
