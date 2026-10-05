---
title: 'Deterministic slot allocation engine & port offset calculator'
type: 'feature'
ticket: 3
created: '2026-10-06'
status: 'built'
followup_review_recommended: false
baseline_revision: '68eb24820f3950ffc28102d78be771c7168787e9'
route: 'full'
route_source: 'auto'
risk: 'low'
review: 'quick'
review_source: 'pinned'
lenses_ran: ['quick']
review_loop_iteration: 0
context:
  - '_bmad-output/initiative-git-claw/architecture-git-claw/architecture-git-claw.md'
  - '_bmad-output/initiative-git-claw/prd-git-claw/prd-git-claw.md'
---

<frozen-after-approval reason="human-owned intent — do not modify unless human renegotiates">

## Intent

**Problem:** `git-claw` requires deterministic slot allocation (assigning the lowest available positive integer ID starting from 1) and calculating effective port offsets (`Effective Port = Base Port + Slot ID`) with port validation.

**Approach:** Implement `core::slot` for lowest-available slot calculation and recycling, and `core::port` for deterministic port offset mapping and environment variable key formatting (`PORT_<KEY>`), with overflow protection.

## Boundaries & Constraints

**Always:**
- Keep `core/slot.rs` and `core/port.rs` pure: zero I/O, zero side-effects.
- Slot IDs are positive non-zero integers (1, 2, 3, ...).
- Lowest unused slot ID must always be selected (e.g. if slots 1 and 3 are taken, allocate 2).
- Effective port formula: `base_port + slot_id`. Validate that port does not exceed `65535`.
- Port environment key formatting: convert key to uppercase snake_case and prefix with `PORT_`.

**Never:**
- Do not perform file access, locking, or Git calls in `core/`.
- Do not allow port overflow beyond `u16::MAX`.

## I/O & Edge-Case Matrix

| Scenario | Input / State | Expected Output / Behavior | Error Handling |
|----------|--------------|---------------------------|----------------|
| Empty slots | Occupied slots: `[]` | Returns Slot ID `1` | Ok(1) |
| Contiguous slots | Occupied slots: `[1, 2, 3]` | Returns Slot ID `4` | Ok(4) |
| Recycled slot gap | Occupied slots: `[1, 3, 4]` | Returns lowest gap Slot ID `2` | Ok(2) |
| Valid port calculation | Base port `3000`, slot ID `2` | Effective port `3002`, env key `PORT_WEB` | Ok((3002, "PORT_WEB")) |
| Port overflow | Base port `65535`, slot ID `1` | Detects overflow beyond 65535 | Err(PortError::PortOverflow) |

</frozen-after-approval>

## Code Map

- `src/core/slot.rs` -- Pure slot allocation logic: `allocate_lowest_slot(occupied: &[u32]) -> u32`.
- `src/core/port.rs` -- Port arithmetic, env key generation, overflow validation: `calculate_effective_ports(base_ports: &BTreeMap<String, u16>, slot_id: u32) -> Result<BTreeMap<String, u16>, PortError>`.
- `src/core/error.rs` -- Domain error `PortError`.
- `src/core/mod.rs` -- Re-exports `slot` and `port` modules.
- `tests/test_slot_port.rs` -- Unit and integration tests for slot allocation and port offset calculations.

## Tasks & Acceptance

**Execution:**
- [x] `src/core/error.rs` -- Add PortError to domain errors -- Handles port overflow errors.
- [x] `src/core/slot.rs` -- Implement deterministic slot allocator -- Pure function computing lowest available positive integer.
- [x] `src/core/port.rs` -- Implement port calculator and env key generator -- Computes `base_port + slot_id` and formats `PORT_<KEY>`.
- [x] `src/core/mod.rs` -- Re-export slot and port APIs -- Makes models accessible to application and integration tests.
- [x] `tests/test_slot_port.rs` -- Implement tests for slot and port logic -- Verifies slot gaps, recycling, port offset arithmetic, and overflows.

**Acceptance Criteria:**
- Given an empty list of active slots, when allocating next slot, then slot ID 1 is returned.
- Given active slots `[1, 3]`, when allocating next slot, then slot ID 2 is returned (gap recycling).
- Given base ports `{"web": 3000, "api": 8000}` and slot ID 5, when calculating ports, then `PORT_WEB=3005` and `PORT_API=8005` are computed.
- Given base port 65535 and slot ID 1, when calculating ports, then a `PortOverflow` error is returned.

## Implementation Notes
- Implemented `allocate_lowest_slot` in `src/core/slot.rs` using BTreeSet to find the lowest available positive integer >= 1.
- Implemented `format_port_env_key` and `calculate_effective_ports` in `src/core/port.rs` with overflow checking against `u16::MAX`.
- Added `PortError` to `src/core/error.rs`.
- Re-exported functions and types in `src/core/mod.rs`.
- Added unit tests in `src/core/slot.rs` and `src/core/port.rs`, and integration tests in `tests/test_slot_port.rs`.

## Plan Change Log

## Review Triage Log

### 2026-10-06 — Review pass
- verdicts: 0 findings — high 0, medium 0, low 0, false 0, maybe-false 0
- findings: []

## Verification

**Commands:**
- `cargo check` -- expected: zero errors and zero warnings
- `cargo test` -- expected: all unit and integration tests pass

## Auto Run Result

### Summary of implemented change
Implemented pure deterministic slot allocator (recycling freed IDs starting from 1) and effective port calculator (`Effective Port = Base Port + Slot ID`) with environment variable key formatting (`PORT_<KEY>`) and overflow protection.

### Files changed
- `src/core/error.rs` - Added `PortError`.
- `src/core/slot.rs` - Lowest available slot ID allocator with unit tests.
- `src/core/port.rs` - Port calculation arithmetic, env key sanitizer, and unit tests.
- `src/core/mod.rs` - Re-exports for domain models.
- `tests/test_slot_port.rs` - Integration test suite covering all matrix edge-cases.

### Review findings breakdown
- Patches applied: 0
- Items deferred: 0
- Findings rejected: 0

### Follow-up review recommendation
- `false`

### Verification performed
- `cargo test`: 23 passed, 0 failed, 0 warnings.
- `cargo check`: Passed with 0 errors and 0 warnings.

### Residual risks
- None. Pure logic layer verified with complete unit and integration tests.
