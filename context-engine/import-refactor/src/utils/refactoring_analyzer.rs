use crate::utils::{
    common::{
        format_relative_path,
        print_file_location_with_info,
    },
    duplication_analyzer::{
        CodebaseDuplicationAnalyzer,
        DuplicationAnalysis,
    },
};
use std::path::Path;

/// Configuration for refactoring analysis
#[derive(Debug, Clone)]
pub struct AnalysisConfig {
    pub workspace_name: Option<String>,
    pub min_duplicate_threshold: usize,
    pub complexity_threshold: u32,
    pub similarity_threshold: f32,
    pub verbose: bool,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            workspace_name: None,
            min_duplicate_threshold: 2,
            complexity_threshold: 5,
            similarity_threshold: 0.8,
            verbose: false,
        }
    }
}

/// Generic Rust codebase refactoring analyzer
pub struct RefactoringAnalyzer {
    config: AnalysisConfig,
}

impl RefactoringAnalyzer {
    pub fn new(config: AnalysisConfig) -> Self {
        Self { config }
    }

    pub fn with_defaults() -> Self {
        Self::new(AnalysisConfig::default())
    }

    /// Perform comprehensive analysis and generate refactoring recommendations
    pub fn analyze_and_recommend(
        &self,
        workspace_root: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let workspace_name = self.get_workspace_name(workspace_root);

        println!("🚀 RUST CODEBASE ANALYSIS");
        println!("========================");
        if let Some(name) = &workspace_name {
            println!("📁 Analyzing: {}", name);
        }
        println!("📍 Path: {}\n", workspace_root.display());

        // Run duplication analysis
        let mut analyzer = CodebaseDuplicationAnalyzer::new(workspace_root);
        let analysis = analyzer.analyze_codebase()?;

        // Print detailed results
        analyzer.print_analysis_results(&analysis);

        // Generate specific recommendations
        self.print_specific_recommendations(&analysis, workspace_root);

        Ok(())
    }

    /// Detect workspace name from Cargo.toml or directory name
    fn get_workspace_name(
        &self,
        workspace_root: &Path,
    ) -> Option<String> {
        if let Some(name) = &self.config.workspace_name {
            return Some(name.clone());
        }

        // Try to read from Cargo.toml
        let cargo_toml = workspace_root.join("Cargo.toml");
        if cargo_toml.exists() {
            if let Ok(content) = std::fs::read_to_string(&cargo_toml) {
                for line in content.lines() {
                    if line.trim().starts_with("name =") {
                        if let Some(name) = line.split('"').nth(1) {
                            return Some(name.to_string());
                        }
                    }
                }
            }
        }

        // Fall back to directory name
        workspace_root
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name.to_string())
    }

    /// Print specific recommendations based on the analysis
    fn print_specific_recommendations(
        &self,
        analysis: &DuplicationAnalysis,
        workspace_root: &Path,
    ) {
        println!("🎯 REFACTORING RECOMMENDATIONS");
        println!("=============================\n");

        let total_opportunities = analysis.potential_utilities.len();
        if total_opportunities == 0 {
            println!("✅ No significant duplication detected! Codebase appears well-organized.\n");
            return;
        }

        // High Priority Actions
        if !analysis.identical_functions.is_empty() {
            println!("🔥 HIGH PRIORITY (Exact Duplicates):");
            println!("-----------------------------------");

            for (i, group) in analysis.identical_functions.iter().enumerate() {
                if group.len() >= self.config.min_duplicate_threshold {
                    let function_name = &group[0].signature.name;
                    let lines_per_function = group[0].complexity_score as usize;
                    let total_lines_saved =
                        group.len().saturating_sub(1) * lines_per_function;

                    println!(
                        "{}. Extract '{}' function:",
                        i + 1,
                        function_name
                    );
                    println!("   � {} identical copies found", group.len());
                    println!(
                        "   � Estimated lines saved: ~{}",
                        total_lines_saved
                    );
                    println!(
                        "   🏗️  Suggested: Create shared utility function"
                    );

                    if self.config.verbose {
                        println!("   � Locations:");
                        for pattern in group.iter().take(3) {
                            print_file_location_with_info(
                                &pattern.file_path,
                                workspace_root,
                                pattern.line_number,
                                "",
                            );
                        }
                        if group.len() > 3 {
                            println!(
                                "      • ... and {} more",
                                group.len() - 3
                            );
                        }
                    }
                    println!();
                }
            }
        }

        // Medium Priority Actions
        if !analysis.similar_functions.is_empty() {
            println!("⚡ MEDIUM PRIORITY (Similar Functions):");
            println!("--------------------------------------");

            for (i, group) in analysis.similar_functions.iter().enumerate() {
                if group.len() >= self.config.min_duplicate_threshold {
                    let function_name = &group[0].signature.name;
                    let avg_complexity =
                        group.iter().map(|p| p.complexity_score).sum::<u32>()
                            / group.len() as u32;

                    println!("{}. Unify '{}' variants:", i + 1, function_name);
                    println!("   📊 {} similar implementations", group.len());
                    println!("   🧮 Average complexity: {}", avg_complexity);
                    println!("   🏗️  Suggested: Parameterize with configuration or strategy pattern");

                    if self.config.verbose {
                        println!("   📍 Locations:");
                        for pattern in group.iter().take(3) {
                            print_file_location_with_info(
                                &pattern.file_path,
                                workspace_root,
                                pattern.line_number,
                                format!(
                                    "params: {}",
                                    pattern.signature.parameter_count
                                ),
                            );
                        }
                        if group.len() > 3 {
                            println!(
                                "      • ... and {} more",
                                group.len() - 3
                            );
                        }
                    }
                    println!();
                }
            }
        }

        // Patterns analysis
        self.print_pattern_analysis(
            &analysis.repeated_patterns,
            workspace_root,
        );

        // Implementation suggestions
        self.print_implementation_suggestions(&analysis, workspace_root);
    }

    /// Analyze patterns in function names and suggest improvements
    fn print_pattern_analysis(
        &self,
        repeated_patterns: &std::collections::HashMap<
            String,
            Vec<crate::utils::duplication_analyzer::MethodPattern>,
        >,
        workspace_root: &Path,
    ) {
        if repeated_patterns.is_empty() {
            return;
        }

        println!("🔍 PATTERN ANALYSIS:");
        println!("-------------------");

        let mut pattern_count = 0;
        for (name, patterns) in repeated_patterns {
            if patterns.len() >= self.config.min_duplicate_threshold {
                pattern_count += 1;
                if pattern_count <= 5 {
                    // Show top 5 patterns
                    println!(
                        "• Function name '{}' appears in {} files",
                        name,
                        patterns.len()
                    );

                    if self.config.verbose {
                        for pattern in patterns.iter().take(3) {
                            println!(
                                "  └─ {}",
                                format_relative_path(
                                    &pattern.file_path,
                                    workspace_root
                                )
                            );
                        }
                        if patterns.len() > 3 {
                            println!(
                                "  └─ ... and {} more",
                                patterns.len() - 3
                            );
                        }
                    }
                }
            }
        }

        if pattern_count > 5 {
            println!("• ... and {} more repeated patterns", pattern_count - 5);
        }

        if pattern_count > 0 {
            println!("💡 Consider: Reviewing if these represent actual duplicates or valid patterns\n");
        }
    }

    /// Print implementation suggestions based on analysis
    fn print_implementation_suggestions(
        &self,
        analysis: &DuplicationAnalysis,
        workspace_root: &Path,
    ) {
        println!("🏗️ IMPLEMENTATION SUGGESTIONS:");
        println!("------------------------------");

        //let total_duplicates = analysis.identical_functions.iter()
        //    .map(|g| g.len().saturating_sub(1))
        //    .sum::<usize>();
        //
        //let total_similar = analysis.similar_functions.iter()
        //    .map(|g| g.len().saturating_sub(1))
        //    .sum::<usize>();

        let estimated_reduction = analysis
            .potential_utilities
            .iter()
            .map(|o| o.estimated_lines_saved)
            .sum::<usize>();

        println!("📊 Impact Analysis:");
        println!(
            "   • Exact duplicates: {} function groups",
            analysis.identical_functions.len()
        );
        println!(
            "   • Similar functions: {} function groups",
            analysis.similar_functions.len()
        );
        println!("   • Estimated lines reducible: ~{}", estimated_reduction);
    }

    /// Generate a quick summary of findings
    pub fn print_quick_summary(
        &self,
        analysis: &DuplicationAnalysis,
        workspace_root: &Path,
    ) {
        let total_duplicates = analysis
            .identical_functions
            .iter()
            .map(|g| g.len().saturating_sub(1))
            .sum::<usize>();

        let total_similar = analysis
            .similar_functions
            .iter()
            .map(|g| g.len().saturating_sub(1))
            .sum::<usize>();

        let total_opportunities = analysis.potential_utilities.len();
        let workspace_name = self
            .get_workspace_name(workspace_root)
            .unwrap_or_else(|| "codebase".to_string());

        println!("📈 ANALYSIS SUMMARY");
        println!("==================");
        println!("� Project: {}", workspace_name);
        println!(
            "�🔄 Identical function groups: {}",
            analysis.identical_functions.len()
        );
        println!(
            "🔀 Similar function groups: {}",
            analysis.similar_functions.len()
        );
        println!("💡 Refactoring opportunities: {}", total_opportunities);
        println!(
            "📉 Total duplicate instances: {}",
            total_duplicates + total_similar
        );
        println!();

        if total_opportunities > 0 {
            if !analysis.identical_functions.is_empty() {
                println!(
                    "🎯 PRIORITY: Extract {} exact duplicate function groups",
                    analysis.identical_functions.len()
                );
            }
            if !analysis.similar_functions.is_empty() {
                println!(
                    "⚡ NEXT: Unify {} similar function groups",
                    analysis.similar_functions.len()
                );
            }

            let estimated_savings = analysis
                .potential_utilities
                .iter()
                .map(|o| o.estimated_lines_saved)
                .sum::<usize>();
            if estimated_savings > 0 {
                println!(
                    "💾 IMPACT: ~{} lines could be eliminated",
                    estimated_savings
                );
            }
        } else {
            println!("✅ RESULT: No significant duplication detected!");
            println!("🎉 Codebase appears well-organized.");
        }
        println!();
    }
}
