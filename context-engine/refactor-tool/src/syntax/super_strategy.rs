use crate::{
    common::format::format_relative_path,
    syntax::parser::ImportInfo,
    syntax::transformer::{ImportReplacementStrategy, ReplacementAction},
};
use std::path::{Path, PathBuf};

/// Super normalization strategy: super:: -> crate::
pub struct SuperNormalizationStrategy {
    pub crate_root: PathBuf,
}

impl ImportReplacementStrategy for SuperNormalizationStrategy {
    fn create_replacement(
        &self,
        import: &ImportInfo,
    ) -> Option<String> {
        use crate::core::path::is_super_import;

        // Only replace if it's actually a super import
        if !is_super_import(&import.import_path) {
            return None; // Don't change non-super imports
        }

        // Create a mutable copy and normalize it
        let mut normalized_import = import.clone();
        if let Err(_) =
            normalized_import.normalize_super_imports(&self.crate_root)
        {
            return None; // If normalization fails, don't change anything
        }

        // Return the normalized import statement
        Some(format!(
            "use {};",
            if normalized_import.imported_items.is_empty() {
                // Simple import without items: use crate::path;
                normalized_import.import_path
            } else {
                // Import with items: use crate::path::{Item1, Item2};
                format!(
                    "{}::{{{}}}",
                    normalized_import.import_path,
                    normalized_import.imported_items.join(", ")
                )
            }
        ))
    }

    fn get_description(&self) -> &str {
        "Normalize super:: imports to crate:: format"
    }

    fn format_verbose_message(
        &self,
        _import: &ImportInfo,
        action: ReplacementAction,
        file_path: &Path,
        _workspace_root: &Path,
    ) -> String {
        match action {
            ReplacementAction::Replaced { from, to } => {
                format!(
                    "  Normalized super:: import '{}' -> '{}' in {}",
                    from,
                    to,
                    format_relative_path(file_path)
                )
            },
            ReplacementAction::NotFound { searched_for } => {
                format!(
                    "  Warning: Could not find super:: import to normalize: {} in {}",
                    searched_for, format_relative_path(file_path)
                )
            },
            _ => format!(
                "Unexpected action for super normalization: {:?}",
                action
            ),
        }
    }
}
