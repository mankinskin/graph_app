//! Refactoring steps and dependency management
//!
//! This module defines the individual refactoring steps that can be performed
//! and manages their dependencies to ensure they are executed in the correct order
//! and only when needed.

use crate::analysis::crates::CrateNames;
use anyhow::{bail, Result};
use std::collections::{HashMap, HashSet};

/// Individual refactoring steps that can be performed
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RefactorStep {
    /// Parse imports from source/target crates
    ParseImports,
    /// Analyze and categorize the parsed imports
    AnalyzeImports,
    /// Normalize super:: imports to crate:: format
    NormalizeSuperImports,
    /// Generate pub use statements in source crate's lib.rs
    GenerateExports,
    /// Replace import statements in target files
    ReplaceImports,
    /// Validate compilation after changes
    ValidateCompilation,
}

impl RefactorStep {
    /// Get a human-readable description of this step
    pub fn description(&self) -> &'static str {
        match self {
            RefactorStep::ParseImports => {
                "Parse import statements from crate files"
            },
            RefactorStep::AnalyzeImports => "Analyze and categorize imports",
            RefactorStep::NormalizeSuperImports => {
                "Normalize super:: imports to crate:: format"
            },
            RefactorStep::GenerateExports => {
                "Generate pub use statements in lib.rs"
            },
            RefactorStep::ReplaceImports => "Replace imports in target files",
            RefactorStep::ValidateCompilation => {
                "Validate compilation after changes"
            },
        }
    }

    /// Get the dependencies for this step
    pub fn dependencies(&self) -> Vec<RefactorStep> {
        match self {
            RefactorStep::ParseImports => vec![],
            RefactorStep::AnalyzeImports => vec![RefactorStep::ParseImports],
            RefactorStep::NormalizeSuperImports => {
                vec![RefactorStep::ParseImports]
            },
            RefactorStep::GenerateExports => {
                vec![RefactorStep::ParseImports, RefactorStep::AnalyzeImports]
            },
            RefactorStep::ReplaceImports => {
                vec![RefactorStep::ParseImports, RefactorStep::AnalyzeImports]
            },
            RefactorStep::ValidateCompilation => vec![
                RefactorStep::GenerateExports,
                RefactorStep::ReplaceImports,
            ],
        }
    }
}

/// Configuration that determines which refactoring steps should be executed
#[derive(Debug, Clone)]
pub struct RefactorStepsConfig {
    /// Whether to normalize super:: imports (default: true)
    pub normalize_super: bool,
    /// Whether to generate exports (default: true)
    pub generate_exports: bool,
    /// Whether to replace imports (default: true)
    pub replace_imports: bool,
    /// Whether to validate compilation (default: true for non-dry-run)
    pub validate_compilation: bool,
    /// The type of refactoring being performed
    pub crate_names: CrateNames,
}

impl RefactorStepsConfig {
    /// Create a new configuration from CLI flags
    pub fn from_flags(
        keep_super: bool,
        keep_exports: bool,
        dry_run: bool,
        crate_names: CrateNames,
    ) -> Self {
        Self {
            normalize_super: !keep_super,    // Invert the flag
            generate_exports: !keep_exports, // Invert the flag
            replace_imports: !keep_exports, // If keeping exports, don't replace imports either
            validate_compilation: !dry_run, // Only validate if not in dry run
            crate_names,
        }
    }

    /// Determine which steps are requested based on the configuration
    pub fn requested_steps(&self) -> Vec<RefactorStep> {
        let mut steps = Vec::new();

        // Check if any actual refactoring work is needed
        let has_refactoring_work = self.normalize_super
            || self.generate_exports
            || self.replace_imports;

        if !has_refactoring_work {
            // True no-op mode - don't even parse imports
            return steps;
        }

        // Parse and analyze imports (needed for refactoring)
        steps.push(RefactorStep::ParseImports);
        steps.push(RefactorStep::AnalyzeImports);

        // Add conditional steps based on configuration
        if self.normalize_super {
            steps.push(RefactorStep::NormalizeSuperImports);
        }

        if self.generate_exports {
            steps.push(RefactorStep::GenerateExports);
        }

        if self.replace_imports {
            steps.push(RefactorStep::ReplaceImports);
        }

        if self.validate_compilation
            && (self.generate_exports || self.replace_imports)
        {
            steps.push(RefactorStep::ValidateCompilation);
        }

        steps
    }
}

/// Manages the execution order of refactoring steps and their dependencies
#[derive(Debug)]
pub struct RefactorStepsManager {
    config: RefactorStepsConfig,
    execution_plan: Vec<RefactorStep>,
}

impl RefactorStepsManager {
    /// Create a new steps manager with the given configuration
    pub fn new(config: RefactorStepsConfig) -> Result<Self> {
        let requested_steps = config.requested_steps();

        if requested_steps.is_empty() {
            return Ok(Self {
                config,
                execution_plan: Vec::new(),
            });
        }

        let execution_plan = Self::resolve_dependencies(&requested_steps)?;

        Ok(Self {
            config,
            execution_plan,
        })
    }

    /// Check if any refactoring work is requested
    pub fn has_work(&self) -> bool {
        !self.execution_plan.is_empty()
    }

    /// Get the steps that will be executed
    pub fn execution_plan(&self) -> &[RefactorStep] {
        &self.execution_plan
    }

    /// Check if a specific step is in the execution plan
    pub fn will_execute(
        &self,
        step: RefactorStep,
    ) -> bool {
        self.execution_plan.contains(&step)
    }

    /// Get a summary of what will be done
    pub fn summary(&self) -> RefactorSummary {
        let steps = &self.execution_plan;

        RefactorSummary {
            will_parse_imports: steps.contains(&RefactorStep::ParseImports),
            will_normalize_super: steps
                .contains(&RefactorStep::NormalizeSuperImports),
            will_generate_exports: steps
                .contains(&RefactorStep::GenerateExports),
            will_replace_imports: steps.contains(&RefactorStep::ReplaceImports),
            will_validate: steps.contains(&RefactorStep::ValidateCompilation),
            total_steps: steps.len(),
        }
    }

    /// Resolve dependencies and create an execution order
    fn resolve_dependencies(
        requested_steps: &[RefactorStep]
    ) -> Result<Vec<RefactorStep>> {
        let mut resolved = Vec::new();
        let mut visited = HashSet::new();
        let mut visiting = HashSet::new();

        // Build a map of all steps and their dependencies
        let mut all_steps = HashSet::new();
        let mut step_deps = HashMap::new();

        for &step in requested_steps {
            Self::collect_dependencies(step, &mut all_steps, &mut step_deps);
        }

        // Topological sort to resolve dependencies
        for &step in requested_steps {
            Self::visit_step(
                step,
                &step_deps,
                &mut resolved,
                &mut visited,
                &mut visiting,
            )?;
        }

        Ok(resolved)
    }

    /// Recursively collect all dependencies for a step
    fn collect_dependencies(
        step: RefactorStep,
        all_steps: &mut HashSet<RefactorStep>,
        step_deps: &mut HashMap<RefactorStep, Vec<RefactorStep>>,
    ) {
        if all_steps.contains(&step) {
            return;
        }

        all_steps.insert(step);
        let deps = step.dependencies();
        step_deps.insert(step, deps.clone());

        for dep in deps {
            Self::collect_dependencies(dep, all_steps, step_deps);
        }
    }

    /// Visit a step during topological sorting
    fn visit_step(
        step: RefactorStep,
        step_deps: &HashMap<RefactorStep, Vec<RefactorStep>>,
        resolved: &mut Vec<RefactorStep>,
        visited: &mut HashSet<RefactorStep>,
        visiting: &mut HashSet<RefactorStep>,
    ) -> Result<()> {
        if visited.contains(&step) {
            return Ok(());
        }

        if visiting.contains(&step) {
            bail!("Circular dependency detected involving step: {:?}", step);
        }

        visiting.insert(step);

        // Visit all dependencies first
        if let Some(deps) = step_deps.get(&step) {
            for &dep in deps {
                Self::visit_step(dep, step_deps, resolved, visited, visiting)?;
            }
        }

        visiting.remove(&step);
        visited.insert(step);

        // Only add to resolved if not already there
        if !resolved.contains(&step) {
            resolved.push(step);
        }

        Ok(())
    }
}

/// Summary of what refactoring operations will be performed
#[derive(Debug, Clone)]
pub struct RefactorSummary {
    pub will_parse_imports: bool,
    pub will_normalize_super: bool,
    pub will_generate_exports: bool,
    pub will_replace_imports: bool,
    pub will_validate: bool,
    pub total_steps: usize,
}

impl RefactorSummary {
    /// Check if this is a no-op (no actual changes)
    pub fn is_no_op(&self) -> bool {
        !self.will_generate_exports
            && !self.will_replace_imports
            && !self.will_normalize_super
    }

    /// Get a human-readable description of the operations
    pub fn describe(&self) -> String {
        if self.is_no_op() {
            return "No refactoring operations will be performed (analysis only)".to_string();
        }

        let mut operations = Vec::new();

        if self.will_normalize_super {
            operations.push("normalize super:: imports");
        }

        if self.will_generate_exports {
            operations.push("generate pub use statements");
        }

        if self.will_replace_imports {
            operations.push("replace import statements");
        }

        match operations.len() {
            0 => "No operations".to_string(),
            1 => format!("Will {}", operations[0]),
            2 => format!("Will {} and {}", operations[0], operations[1]),
            _ => {
                let last = operations.pop().unwrap();
                format!("Will {}, and {}", operations.join(", "), last)
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_step_dependencies() {
        // Test that ParseImports has no dependencies
        assert_eq!(RefactorStep::ParseImports.dependencies(), vec![]);

        // Test that AnalyzeImports depends on ParseImports
        assert_eq!(
            RefactorStep::AnalyzeImports.dependencies(),
            vec![RefactorStep::ParseImports]
        );

        // Test that GenerateExports has the right dependencies
        let deps = RefactorStep::GenerateExports.dependencies();
        assert!(deps.contains(&RefactorStep::ParseImports));
        assert!(deps.contains(&RefactorStep::AnalyzeImports));
    }

    #[test]
    fn test_no_op_configuration() {
        let config = RefactorStepsConfig {
            normalize_super: false,
            generate_exports: false,
            replace_imports: false,
            validate_compilation: false,
            crate_names: CrateNames::SelfRefactor {
                crate_name: "test".to_string(),
            },
        };

        let steps = config.requested_steps();
        // True no-op - no steps at all
        assert_eq!(steps.len(), 0);
    }

    #[test]
    fn test_full_configuration() {
        let config = RefactorStepsConfig {
            normalize_super: true,
            generate_exports: true,
            replace_imports: true,
            validate_compilation: true,
            crate_names: CrateNames::SelfRefactor {
                crate_name: "test".to_string(),
            },
        };

        let manager = RefactorStepsManager::new(config).unwrap();
        let plan = manager.execution_plan();

        // Should include all steps in dependency order
        assert!(plan.contains(&RefactorStep::ParseImports));
        assert!(plan.contains(&RefactorStep::AnalyzeImports));
        assert!(plan.contains(&RefactorStep::NormalizeSuperImports));
        assert!(plan.contains(&RefactorStep::GenerateExports));
        assert!(plan.contains(&RefactorStep::ReplaceImports));
        assert!(plan.contains(&RefactorStep::ValidateCompilation));

        // ParseImports should come before others
        let parse_idx = plan
            .iter()
            .position(|&s| s == RefactorStep::ParseImports)
            .unwrap();
        let analyze_idx = plan
            .iter()
            .position(|&s| s == RefactorStep::AnalyzeImports)
            .unwrap();
        assert!(parse_idx < analyze_idx);
    }

    #[test]
    fn test_summary_no_op() {
        let summary = RefactorSummary {
            will_parse_imports: true,
            will_normalize_super: false,
            will_generate_exports: false,
            will_replace_imports: false,
            will_validate: false,
            total_steps: 2,
        };

        assert!(summary.is_no_op());
        assert!(summary.describe().contains("No refactoring operations"));
    }
}
