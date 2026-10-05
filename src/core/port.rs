//! Deterministic port calculation and environment variable key formatting.

use crate::core::error::PortError;
use std::collections::BTreeMap;

/// Converts a port key name to an uppercase environment variable name prefixed with `PORT_`.
/// e.g. "web" -> "PORT_WEB", "api-gateway" -> "PORT_API_GATEWAY".
pub fn format_port_env_key(key: &str) -> String {
    let sanitized: String = key
        .chars()
        .map(|c| if c.is_alphanumeric() { c } else { '_' })
        .collect();
    format!("PORT_{}", sanitized.to_uppercase())
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_port_env_key() {
        assert_eq!(format_port_env_key("web"), "PORT_WEB");
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
}
