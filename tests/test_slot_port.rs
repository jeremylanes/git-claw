use git_claw::core::error::PortError;
use git_claw::core::port::{calculate_effective_ports, format_port_env_key};
use git_claw::core::slot::allocate_lowest_slot;
use std::collections::BTreeMap;

#[test]
fn test_slot_allocation_empty_and_contiguous() {
    assert_eq!(allocate_lowest_slot([]), 1);
    assert_eq!(allocate_lowest_slot([1, 2, 3]), 4);
    assert_eq!(allocate_lowest_slot([2, 3]), 1);
}

#[test]
fn test_slot_allocation_gap_recycling() {
    assert_eq!(allocate_lowest_slot([1, 3]), 2);
    assert_eq!(allocate_lowest_slot([1, 2, 4, 5]), 3);
}

#[test]
fn test_format_port_env_keys() {
    assert_eq!(format_port_env_key("http"), "PORT_HTTP");
    assert_eq!(format_port_env_key("web"), "PORT_WEB");
    assert_eq!(format_port_env_key("APP_PORT"), "APP_PORT");
    assert_eq!(format_port_env_key("PORT"), "PORT");
    assert_eq!(
        format_port_env_key("backend-service"),
        "PORT_BACKEND_SERVICE"
    );
}

#[test]
fn test_port_offset_calculation() {
    let mut ports = BTreeMap::new();
    ports.insert("web".to_string(), 3000);
    ports.insert("api".to_string(), 8000);

    let effective = calculate_effective_ports(&ports, 5).expect("calculates ports");
    assert_eq!(effective.get("PORT_WEB"), Some(&3005));
    assert_eq!(effective.get("PORT_API"), Some(&8005));
}

#[test]
fn test_port_offset_overflow() {
    let mut ports = BTreeMap::new();
    ports.insert("overflow".to_string(), 65535);

    let err = calculate_effective_ports(&ports, 1).unwrap_err();
    assert_eq!(
        err,
        PortError::PortOverflow {
            key: "overflow".to_string(),
            base_port: 65535,
            slot_id: 1,
        }
    );
}
