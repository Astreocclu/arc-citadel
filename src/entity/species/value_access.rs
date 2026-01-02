//! Runtime value access for species-specific value structs

/// Trait for runtime access to species values by field name
pub trait ValueAccessor {
    /// Get a value by field name, returns None if field doesn't exist
    fn get_value(&self, field_name: &str) -> Option<f32>;

    /// Set a value by field name, returns false if field doesn't exist
    fn set_value(&mut self, field_name: &str, value: f32) -> bool;

    /// List all field names for validation
    fn field_names() -> &'static [&'static str];
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::entity::species::gnoll::GnollValues;

    #[test]
    fn test_value_accessor_get() {
        let mut values = GnollValues::default();
        values.bloodlust = 0.75;

        assert_eq!(values.get_value("bloodlust"), Some(0.75));
        assert_eq!(values.get_value("nonexistent"), None);
    }

    #[test]
    fn test_value_accessor_set() {
        let mut values = GnollValues::default();

        assert!(values.set_value("bloodlust", 0.9));
        assert_eq!(values.bloodlust, 0.9);

        assert!(!values.set_value("nonexistent", 0.5));
    }

    #[test]
    fn test_field_names() {
        let names = GnollValues::field_names();
        assert!(names.contains(&"bloodlust"));
        assert!(names.contains(&"pack_instinct"));
        assert!(!names.contains(&"nonexistent"));
    }
}
