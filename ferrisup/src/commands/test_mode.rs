use std::sync::atomic::{AtomicBool, Ordering};

static TEST_MODE_OVERRIDE: AtomicBool = AtomicBool::new(false);

/// Detects if the application is running in test mode
///
/// Returns true if the FERRISUP_TEST_MODE environment variable is set to any value
pub fn is_test_mode() -> bool {
    TEST_MODE_OVERRIDE.load(Ordering::Relaxed) || std::env::var_os("FERRISUP_TEST_MODE").is_some()
}

/// Enables non-interactive behavior for the current process.
///
/// This is primarily intended for integration tests. It is deliberately one-way so
/// concurrent tests cannot race by toggling process-global state on and off.
#[doc(hidden)]
pub fn enable_test_mode() {
    TEST_MODE_OVERRIDE.store(true, Ordering::Relaxed);
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
    if is_test_mode() { Ok(default) } else { f() }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_test_mode() {
        enable_test_mode();
        assert!(is_test_mode());
    }

    #[test]
    fn test_test_mode_or() {
        enable_test_mode();
        let result = test_mode_or("default".to_string(), || Ok("interactive".to_string()));
        assert_eq!(result.unwrap(), "default");
    }
}
