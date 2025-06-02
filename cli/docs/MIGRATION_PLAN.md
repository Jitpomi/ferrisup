# Migration Plan for FerrisUp Architecture

This document outlines the step-by-step process to migrate from the current implementation to the new handler-based architecture.

## Migration Phases

### Phase 1: Parallel Implementation (Current)

1. ✅ Create the project_handlers directory with the core interfaces
2. ✅ Implement CLI handlers for Embassy, Dioxus, and Tauri 
3. ✅ Implement template handlers for various template types
4. ✅ Create new_with_handlers.rs as a parallel implementation
5. ✅ Add documentation for the new architecture

This phase allows testing the new implementation without affecting existing code.

### Phase 2: Testing and Validation

1. Add unit tests for the handlers system
2. Test each CLI handler with various parameters
3. Test each template handler with different templates
4. Ensure all existing templates work with the new system
5. Verify that the data science template correctly handles all data formats (CSV, JSON, Parquet)

### Phase 3: Gradual Adoption

1. Create feature flags to enable the new architecture:
   ```rust
   #[cfg(feature = "new_architecture")]
   use project_handlers;
   
   #[cfg(feature = "new_architecture")]
   fn execute(...) {
       // New implementation
   }
   
   #[cfg(not(feature = "new_architecture"))]
   fn execute(...) {
       // Existing implementation
   }
   ```

2. Enable the new architecture for specific templates first:
   ```rust
   if ["embedded-embassy", "dioxus", "tauri"].contains(&template.as_str()) {
       return execute_with_handlers(name, template, git, build, no_interactive, _project_type);
   } else {
       return execute_original(name, template, git, build, no_interactive, _project_type);
   }
   ```

3. Gradually replace the template-specific logic in the original code with calls to handlers

### Phase 4: Complete Migration

1. Replace new.rs with the new_with_handlers.rs implementation
2. Remove any redundant code from the original implementation
3. Update tests to work with the new system
4. Update documentation to reflect the new architecture

## Compatibility Safeguards

To ensure a smooth transition:

1. **Maintain Variable Format**: Ensure the variables collected by the new system match what templates expect
2. **Preserve Next Steps Logic**: Keep the next steps display consistent with the existing system
3. **Respect Existing Templates**: Don't modify template.json files except to fix specific bugs
4. **Handle Edge Cases**: Ensure the system properly handles all scenarios (no-interactive, custom variables, etc.)

## Key Files to Update

1. `src/commands/new.rs` - Replace with the new handler-based implementation
2. `src/lib.rs` - Add the project_handlers module
3. `Cargo.toml` - Add feature flags for gradual migration

## Benefits of This Approach

- **Non-disruptive**: Existing code continues to work during migration
- **Incremental**: Each template can be migrated individually
- **Testable**: Each migration step can be verified independently
- **Reversible**: Feature flags allow quick rollback if issues are found

## Timeline

- **Week 1**: Complete Phase 1 (implementation) and Phase 2 (testing)
- **Week 2**: Begin Phase 3 with migration of Embassy and Dioxus templates
- **Week 3**: Continue Phase 3 with migration of template-based projects
- **Week 4**: Complete Phase 4 with full migration and documentation

## Special Considerations

### Data Science Templates

Special care should be taken with the data science templates to maintain the following improvements:
- Support for CSV, JSON, and Parquet data formats
- Dynamic format selection in variables
- Correct template file selection based on format
- Proper next steps display with the selected format

This will ensure that the recent fixes to the data science templates are preserved during migration.
