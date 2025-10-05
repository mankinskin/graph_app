use std::{
    collections::{
        HashMap,
        HashSet,
    },
    path::{
        Path,
        PathBuf,
    },
};

use super::common::{
    format_relative_path,
    print_file_location,
    print_file_location_with_info,
};

/// Function signature analysis for detecting similar patterns
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FunctionSignature {
    pub name: String,
    pub parameter_count: usize,
    pub parameter_types: Vec<String>,
    pub return_type: Option<String>,
    pub is_public: bool,
}

/// Method pattern for identifying similar logic structures
#[derive(Debug, Clone)]
pub struct MethodPattern {
    pub signature: FunctionSignature,
    pub file_path: PathBuf,
    pub line_number: usize,
    pub complexity_score: u32,
    pub code_hash: u64,
}

/// Duplication analysis result
#[derive(Debug)]
pub struct DuplicationAnalysis {
    pub identical_functions: Vec<Vec<MethodPattern>>,
    pub similar_functions: Vec<Vec<MethodPattern>>,
    pub repeated_patterns: HashMap<String, Vec<MethodPattern>>,
    pub potential_utilities: Vec<RefactoringOpportunity>,
}

/// Refactoring opportunity suggestion
#[derive(Debug)]
pub struct RefactoringOpportunity {
    pub opportunity_type: OpportunityType,
    pub description: String,
    pub affected_functions: Vec<MethodPattern>,
    pub suggested_location: PathBuf,
    pub estimated_lines_saved: usize,
    pub confidence: f32,
}

#[derive(Debug)]
pub enum OpportunityType {
    ExtractUtilityFunction,
    ParameterizeFunction,
    CreateTrait,
    MergeModules,
    ExtractCommonStruct,
}

/// Codebase analyzer for detecting duplication patterns
pub struct CodebaseDuplicationAnalyzer {
    workspace_root: PathBuf,
    patterns: Vec<MethodPattern>,
    analysis_complete: bool,
    config: AnalysisConfig,
}

/// Configuration for duplication analysis
#[derive(Debug, Clone)]
pub struct AnalysisConfig {
    pub min_complexity_threshold: u32,
    pub similarity_threshold: f32,
    pub min_function_length: usize,
    pub exclude_patterns: Vec<String>,
    pub max_files_to_scan: Option<usize>,
}

impl Default for AnalysisConfig {
    fn default() -> Self {
        Self {
            min_complexity_threshold: 5,
            similarity_threshold: 0.8,
            min_function_length: 3,
            exclude_patterns: vec![
                "test".to_string(),
                "tests".to_string(),
                "target".to_string(),
                ".git".to_string(),
            ],
            max_files_to_scan: None,
        }
    }
}

impl CodebaseDuplicationAnalyzer {
    pub fn new(workspace_root: &Path) -> Self {
        Self::with_config(workspace_root, AnalysisConfig::default())
    }

    pub fn with_config(
        workspace_root: &Path,
        config: AnalysisConfig,
    ) -> Self {
        Self {
            workspace_root: workspace_root.to_path_buf(),
            patterns: Vec::new(),
            analysis_complete: false,
            config,
        }
    }

    /// Analyze the entire codebase for duplication patterns
    pub fn analyze_codebase(
        &mut self
    ) -> Result<DuplicationAnalysis, Box<dyn std::error::Error>> {
        println!("🔍 Scanning Rust files for duplication patterns...");

        // Scan all Rust files in the workspace
        self.scan_rust_files()?;

        // Analyze patterns for duplications
        let analysis = self.detect_duplications();
        self.analysis_complete = true;

        println!(
            "✅ Analysis completed! Scanned {} functions across {} files.",
            self.patterns.len(),
            self.count_unique_files()
        );
        Ok(analysis)
    }

    /// Scan all Rust files and extract method patterns
    fn scan_rust_files(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let mut files_scanned = 0;

        // Check if workspace has standard Rust structure
        let src_path = self.workspace_root.join("src");
        if src_path.exists() {
            self.scan_directory_recursive(&src_path, &mut files_scanned)?;
        }

        // Also scan individual crate directories (for workspaces)
        for entry in std::fs::read_dir(&self.workspace_root)? {
            let entry = entry?;
            let path = entry.path();

            if self.should_scan_directory(&path) {
                let crate_src = path.join("src");
                if crate_src.exists() {
                    self.scan_directory_recursive(
                        &crate_src,
                        &mut files_scanned,
                    )?;
                }
            }

            // Check file limit
            if let Some(max_files) = self.config.max_files_to_scan {
                if files_scanned >= max_files {
                    println!("⚠️  Reached file limit of {}. Use config to scan more files.", max_files);
                    break;
                }
            }
        }

        println!("📁 Scanned {} Rust files", files_scanned);
        Ok(())
    }

    /// Check if a directory should be scanned based on configuration
    fn should_scan_directory(
        &self,
        path: &Path,
    ) -> bool {
        if !path.is_dir() {
            return false;
        }

        // Check if directory contains Cargo.toml (indicating it's a crate)
        if !path.join("Cargo.toml").exists() {
            return false;
        }

        // Check exclude patterns
        if let Some(dir_name) = path.file_name().and_then(|n| n.to_str()) {
            for pattern in &self.config.exclude_patterns {
                if dir_name.contains(pattern) {
                    return false;
                }
            }
        }

        true
    }

    /// Recursively scan a directory for Rust files
    fn scan_directory_recursive(
        &mut self,
        dir: &Path,
        files_scanned: &mut usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        for entry in std::fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.is_dir() {
                // Check exclude patterns for subdirectories
                if let Some(dir_name) =
                    path.file_name().and_then(|n| n.to_str())
                {
                    let should_skip = self
                        .config
                        .exclude_patterns
                        .iter()
                        .any(|pattern| dir_name.contains(pattern));

                    if !should_skip {
                        self.scan_directory_recursive(&path, files_scanned)?;
                    }
                }
            } else if path.extension().and_then(|s| s.to_str()) == Some("rs") {
                self.analyze_rust_file(&path)?;
                *files_scanned += 1;

                // Check file limit
                if let Some(max_files) = self.config.max_files_to_scan {
                    if *files_scanned >= max_files {
                        break;
                    }
                }
            }
        }
        Ok(())
    }

    /// Analyze a single Rust file for method patterns
    fn analyze_rust_file(
        &mut self,
        file_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(file_path)?;

        // Parse the file to extract function signatures and patterns
        if let Ok(parsed_file) = syn::parse_file(&content) {
            for (i, item) in parsed_file.items.iter().enumerate() {
                if let syn::Item::Fn(func) = item {
                    let pattern =
                        self.extract_method_pattern(func, file_path, i + 1);
                    self.patterns.push(pattern);
                }

                // Also check inside impl blocks
                if let syn::Item::Impl(impl_block) = item {
                    for (j, impl_item) in impl_block.items.iter().enumerate() {
                        if let syn::ImplItem::Fn(method) = impl_item {
                            let pattern = self.extract_impl_method_pattern(
                                method,
                                file_path,
                                i + j + 1,
                            );
                            self.patterns.push(pattern);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Extract method pattern from a function
    fn extract_method_pattern(
        &self,
        func: &syn::ItemFn,
        file_path: &Path,
        line_number: usize,
    ) -> MethodPattern {
        let signature = FunctionSignature {
            name: func.sig.ident.to_string(),
            parameter_count: func.sig.inputs.len(),
            parameter_types: func
                .sig
                .inputs
                .iter()
                .map(|arg| match arg {
                    syn::FnArg::Typed(pat_type) =>
                        quote::quote!(#pat_type).to_string(),
                    syn::FnArg::Receiver(_) => "self".to_string(),
                })
                .collect(),
            return_type: match &func.sig.output {
                syn::ReturnType::Default => None,
                syn::ReturnType::Type(_, _) => Some("ReturnType".to_string()),
            },
            is_public: matches!(func.vis, syn::Visibility::Public(_)),
        };

        let code_content = quote::quote!(#func).to_string();
        let code_hash = self.simple_hash(&code_content);
        let complexity_score = self.calculate_complexity(&func.block);

        MethodPattern {
            signature,
            file_path: file_path.to_path_buf(),
            line_number,
            complexity_score,
            code_hash,
        }
    }

    /// Extract method pattern from an impl method
    fn extract_impl_method_pattern(
        &self,
        method: &syn::ImplItemFn,
        file_path: &Path,
        line_number: usize,
    ) -> MethodPattern {
        let signature = FunctionSignature {
            name: method.sig.ident.to_string(),
            parameter_count: method.sig.inputs.len(),
            parameter_types: method
                .sig
                .inputs
                .iter()
                .map(|arg| match arg {
                    syn::FnArg::Typed(pat_type) =>
                        quote::quote!(#pat_type).to_string(),
                    syn::FnArg::Receiver(_) => "self".to_string(),
                })
                .collect(),
            return_type: match &method.sig.output {
                syn::ReturnType::Default => None,
                syn::ReturnType::Type(_, _) => Some("ReturnType".to_string()),
            },
            is_public: matches!(method.vis, syn::Visibility::Public(_)),
        };

        let code_content = quote::quote!(#method).to_string();
        let code_hash = self.simple_hash(&code_content);
        let complexity_score = self.calculate_complexity(&method.block);

        MethodPattern {
            signature,
            file_path: file_path.to_path_buf(),
            line_number,
            complexity_score,
            code_hash,
        }
    }

    /// Simple hash function for code content
    fn simple_hash(
        &self,
        content: &str,
    ) -> u64 {
        use std::{
            collections::hash_map::DefaultHasher,
            hash::{
                Hash,
                Hasher,
            },
        };

        let mut hasher = DefaultHasher::new();
        content.hash(&mut hasher);
        hasher.finish()
    }

    /// Calculate complexity score based on block structure
    fn calculate_complexity(
        &self,
        block: &syn::Block,
    ) -> u32 {
        let mut complexity = 1; // Base complexity

        // Count statements
        complexity += block.stmts.len() as u32;

        // Add complexity for control flow (simplified heuristic)
        let block_str = quote::quote!(#block).to_string();

        // Count control flow keywords
        complexity += block_str.matches("if ").count() as u32;
        complexity += block_str.matches("match ").count() as u32;
        complexity += block_str.matches("for ").count() as u32;
        complexity += block_str.matches("while ").count() as u32;
        complexity += block_str.matches("loop ").count() as u32;
        complexity += block_str.matches("fn ").count() as u32;

        // Bonus for nested complexity
        let nesting_level = block_str.matches('{').count().saturating_sub(1);
        complexity += (nesting_level as u32) / 2;

        complexity.max(1) // Minimum complexity of 1
    }

    /// Detect duplication patterns in the analyzed methods
    fn detect_duplications(&self) -> DuplicationAnalysis {
        let mut identical_functions = Vec::new();
        let mut similar_functions = Vec::new();
        let mut repeated_patterns = HashMap::new();
        let mut potential_utilities = Vec::new();

        // Group by identical code hash
        let mut hash_groups: HashMap<u64, Vec<&MethodPattern>> = HashMap::new();
        for pattern in &self.patterns {
            hash_groups
                .entry(pattern.code_hash)
                .or_default()
                .push(pattern);
        }

        // Find identical functions
        for (_, group) in hash_groups {
            if group.len() > 1 {
                identical_functions.push(group.into_iter().cloned().collect());
            }
        }

        // Group by similar signatures
        let mut signature_groups: HashMap<String, Vec<&MethodPattern>> =
            HashMap::new();
        for pattern in &self.patterns {
            let key = format!(
                "{}_{}_{}",
                pattern.signature.name,
                pattern.signature.parameter_count,
                pattern.signature.parameter_types.join(",")
            );
            signature_groups.entry(key).or_default().push(pattern);
        }

        // Find similar functions
        for (_, group) in signature_groups {
            if group.len() > 1 {
                // Check if they're not already in identical_functions
                let group_hashes: HashSet<u64> =
                    group.iter().map(|p| p.code_hash).collect();
                if group_hashes.len() > 1 {
                    similar_functions
                        .push(group.into_iter().cloned().collect());
                }
            }
        }

        // Detect repeated patterns (same function name in different files)
        let mut name_groups: HashMap<String, Vec<&MethodPattern>> =
            HashMap::new();
        for pattern in &self.patterns {
            name_groups
                .entry(pattern.signature.name.clone())
                .or_default()
                .push(pattern);
        }

        for (name, group) in name_groups {
            if group.len() > 1 {
                repeated_patterns
                    .insert(name, group.into_iter().cloned().collect());
            }
        }

        // Generate refactoring opportunities
        potential_utilities
            .extend(self.generate_utility_opportunities(&identical_functions));
        potential_utilities.extend(
            self.generate_parameterization_opportunities(&similar_functions),
        );

        DuplicationAnalysis {
            identical_functions,
            similar_functions,
            repeated_patterns,
            potential_utilities,
        }
    }

    /// Generate utility extraction opportunities
    fn generate_utility_opportunities(
        &self,
        identical_groups: &[Vec<MethodPattern>],
    ) -> Vec<RefactoringOpportunity> {
        let mut opportunities = Vec::new();

        for group in identical_groups {
            if group.len() >= 2
                && group[0].complexity_score
                    >= self.config.min_complexity_threshold
            {
                let estimated_lines_saved = group.len().saturating_sub(1)
                    * (group[0].complexity_score as usize);

                opportunities.push(RefactoringOpportunity {
                    opportunity_type: OpportunityType::ExtractUtilityFunction,
                    description: format!(
                        "Extract identical function '{}' ({} duplicates) to shared module",
                        group[0].signature.name,
                        group.len()
                    ),
                    affected_functions: group.clone(),
                    suggested_location: self.workspace_root.join("src").join("utils").join("extracted.rs"),
                    estimated_lines_saved,
                    confidence: 0.95,
                });
            }
        }

        opportunities
    }

    /// Generate parameterization opportunities
    fn generate_parameterization_opportunities(
        &self,
        similar_groups: &[Vec<MethodPattern>],
    ) -> Vec<RefactoringOpportunity> {
        let mut opportunities = Vec::new();

        for group in similar_groups {
            if group.len() >= 2 {
                let avg_complexity =
                    group.iter().map(|p| p.complexity_score).sum::<u32>()
                        / group.len() as u32;

                // Only suggest parameterization if complexity is worth it
                if avg_complexity >= self.config.min_complexity_threshold {
                    let estimated_lines_saved = (group.len().saturating_sub(1)
                        * avg_complexity as usize)
                        / 2;

                    opportunities.push(RefactoringOpportunity {
                        opportunity_type: OpportunityType::ParameterizeFunction,
                        description: format!(
                            "Unify similar functions '{}' ({} variants) with parameterization",
                            group[0].signature.name,
                            group.len()
                        ),
                        affected_functions: group.clone(),
                        suggested_location: self.workspace_root.join("src").join("utils").join("parameterized.rs"),
                        estimated_lines_saved,
                        confidence: 0.75,
                    });
                }
            }
        }

        opportunities
    }

    /// Print comprehensive analysis results
    pub fn print_analysis_results(
        &self,
        analysis: &DuplicationAnalysis,
    ) {
        println!("\n🎯 COMPREHENSIVE DUPLICATION ANALYSIS RESULTS");
        println!("=====================================");

        // Print identical functions
        if !analysis.identical_functions.is_empty() {
            println!(
                "\n🔄 IDENTICAL FUNCTIONS ({})",
                analysis.identical_functions.len()
            );
            println!("-------------------------------");

            for (i, group) in analysis.identical_functions.iter().enumerate() {
                println!(
                    "{}. Function '{}' - {} identical copies:",
                    i + 1,
                    group[0].signature.name,
                    group.len()
                );
                for pattern in group {
                    print_file_location(
                        &pattern.file_path,
                        &self.workspace_root,
                        pattern.line_number,
                    );
                }
                println!();
            }
        }

        // Print similar functions
        if !analysis.similar_functions.is_empty() {
            println!(
                "\n🔀 SIMILAR FUNCTIONS ({})",
                analysis.similar_functions.len()
            );
            println!("----------------------------");

            for (i, group) in analysis.similar_functions.iter().enumerate() {
                println!(
                    "{}. Function '{}' - {} similar variants:",
                    i + 1,
                    group[0].signature.name,
                    group.len()
                );
                for pattern in group {
                    print_file_location_with_info(
                        &pattern.file_path,
                        &self.workspace_root,
                        pattern.line_number,
                        format!(
                            "params: {}",
                            pattern.signature.parameter_count
                        ),
                    );
                }
                println!();
            }
        }

        // Print refactoring opportunities
        if !analysis.potential_utilities.is_empty() {
            println!(
                "\n💡 REFACTORING OPPORTUNITIES ({})",
                analysis.potential_utilities.len()
            );
            println!("----------------------------------");

            for (i, opportunity) in
                analysis.potential_utilities.iter().enumerate()
            {
                println!(
                    "{}. {} (Confidence: {:.0}%)",
                    i + 1,
                    opportunity.description,
                    opportunity.confidence * 100.0
                );
                println!(
                    "   💾 Estimated lines saved: {}",
                    opportunity.estimated_lines_saved
                );
                println!(
                    "   📍 Suggested location: {}",
                    format_relative_path(
                        &opportunity.suggested_location,
                        &self.workspace_root
                    )
                );
                println!();
            }
        }

        // Print summary
        let total_duplicates = analysis
            .identical_functions
            .iter()
            .map(|g| g.len().saturating_sub(1))
            .sum::<usize>();
        let total_estimated_savings = analysis
            .potential_utilities
            .iter()
            .map(|o| o.estimated_lines_saved)
            .sum::<usize>();

        println!("\n📊 SUMMARY");
        println!("----------");
        println!("• Total duplicate function instances: {}", total_duplicates);
        println!(
            "• Total refactoring opportunities: {}",
            analysis.potential_utilities.len()
        );
        println!(
            "• Estimated lines that could be saved: {}",
            total_estimated_savings
        );
        println!("• Files analyzed: {}", self.count_unique_files());
        println!();
    }

    /// Count unique files that were analyzed
    fn count_unique_files(&self) -> usize {
        let unique_files: HashSet<&PathBuf> =
            self.patterns.iter().map(|p| &p.file_path).collect();
        unique_files.len()
    }
}
