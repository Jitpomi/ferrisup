use std::sync::OnceLock;

/// Detects if the application is running in test mode
/// 
/// Returns true if the FERRISUP_TEST_MODE environment variable is set to any value
pub fn is_test_mode() -> bool {
    static TEST_MODE: OnceLock<bool> = OnceLock::new();
    *TEST_MODE.get_or_init(|| std::env::var("FERRISUP_TEST_MODE").is_ok())
}

/// Provides a default value when in test mode, or calls the provided function to get a value otherwise
/// 
/// This is useful for bypassing interactive prompts in tests while still allowing normal interactive
/// behavior in regular usage.
pub fn test_mode_or<F, T>(default: T, f: F) -> anyhow::Result<T> 
where
    F: FnOnce() -> anyhow::Result<T>,
    T: Clone,
{
    if is_test_mode() {
        Ok(default)
    } else {
        f()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_test_mode() {
        // Test with environment variable set
        std::env::set_var("FERRISUP_TEST_MODE", "1");
        assert!(is_test_mode());

        // Test with environment variable unset
        std::env::remove_var("FERRISUP_TEST_MODE");
        // Note: This will still return true because OnceLock caches the result
        // In real tests, this function would be called before any test runs
    }

    #[test]
    fn test_test_mode_or() {
        // Test with test mode enabled
        std::env::set_var("FERRISUP_TEST_MODE", "1");
        let result = test_mode_or("default".to_string(), || {
            Ok("interactive".to_string())
        });
        assert_eq!(result.unwrap(), "default");

        // Test with test mode disabled
        std::env::remove_var("FERRISUP_TEST_MODE");
        // Note: This will still use the default value because OnceLock caches the result
    }
}
