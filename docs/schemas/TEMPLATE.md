# Module Specification Schema Template

> Copy this file and fill in the sections for each new domain module.

```
MODULE: [PascalCase name]
CATEGORY: [Pure Transform | State Query | State Mutation]
PURPOSE: [Single sentence describing transformation]

=== FILE LOCATION ===
[Exact path, e.g., src/entity/species/orc.rs]

=== PATTERN TO FOLLOW ===
[Path to reference implementation, e.g., src/entity/species/human.rs]
Key differences from reference:
1. [Specific difference]
2. [Another difference]

=== INPUT CONTRACT ===
[TypeName]:
  - field_name: Type           // semantic meaning
  - field_name: Type           // semantic meaning
  ...

=== OUTPUT CONTRACT ===
[TypeName]:
  - field_name: Type           // semantic meaning
  ...

=== STATE ACCESS ===
READS: [None | list of world state types readable]
WRITES: [None | list of component types modified]

=== INVARIANTS ===
1. [Boolean condition that must ALWAYS hold]
2. [Another invariant]
...

=== VALIDATION SCENARIOS ===
SCENARIO: [Short name]
  GIVEN: [Setup conditions]
  INPUT: [Specific input values]
  EXPECTED: [What output should be]
  RATIONALE: [Why this validates correctness]

SCENARIO: [Second scenario]
  ...

=== INTEGRATION POINT ===
Callsite: [Where this module is called from]
```rust
// Wiring code showing imports and invocation
use crate::module::path;
// Show exact integration
```

=== TEST TEMPLATE ===
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_[scenario_name]() {
        // Arrange
        let input = ...;

        // Act
        let result = function_under_test(input);

        // Assert
        assert_eq!(result.field, expected_value);
    }
}
```

=== DEPENDENCIES ===
UPSTREAM: [List of modules that produce this module's inputs]
DOWNSTREAM: [List of modules that consume this module's outputs]

=== ANTI-PATTERNS ===
- NEVER: [Specific thing to avoid]
- NEVER: [Another antipattern]
```

---

## Category-Implicit Error Semantics

| Category | Return Type | Error Behavior |
|----------|-------------|----------------|
| Pure Transform | `Output` directly | Infallible - always succeeds |
| State Query | `Option<Output>` or `Vec<Output>` | May return None/empty, not an error |
| State Mutation | `Result<Output, ModuleError>` | May fail validation |

---

## Checklist Before Implementation

- [ ] MODULE name is PascalCase
- [ ] CATEGORY is one of: Pure Transform, State Query, State Mutation
- [ ] PURPOSE is a single sentence
- [ ] FILE LOCATION is an exact path
- [ ] PATTERN TO FOLLOW points to existing code
- [ ] INPUT CONTRACT lists all fields with types and semantics
- [ ] OUTPUT CONTRACT lists all fields with types and semantics
- [ ] INVARIANTS are boolean conditions, not prose
- [ ] VALIDATION SCENARIOS have GIVEN/INPUT/EXPECTED/RATIONALE
- [ ] INTEGRATION POINT shows actual Rust code
- [ ] TEST TEMPLATE shows Arrange/Act/Assert pattern
- [ ] ANTI-PATTERNS start with "NEVER:"
