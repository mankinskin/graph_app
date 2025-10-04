use anyhow::Result;
use std::{
    fs,
    path::Path,
};
use syn::{
    parse_file,
    Item,
    ItemUse,
    UseTree,
};

/// AST analysis results for validation
#[derive(Debug, Default)]
pub struct AstAnalysis {
    pub pub_use_items: Vec<UseAnalysis>,
    pub public_functions: Vec<String>,
    pub public_structs: Vec<String>,
    pub public_enums: Vec<String>,
    pub public_traits: Vec<String>,
    pub public_constants: Vec<String>,
    pub public_statics: Vec<String>,
    pub public_modules: Vec<String>,
    pub macro_exports: Vec<String>,
    pub conditional_items: Vec<ConditionalItem>,
}

#[derive(Debug)]
pub struct UseAnalysis {
    pub path: String,
    pub items: Vec<String>,
    pub is_nested: bool,
    pub has_conditions: bool,
}

#[derive(Debug)]
pub struct ConditionalItem {
    pub condition: String,
    pub item_type: String,
    pub name: String,
}

/// Parse a Rust source file and extract AST information
pub fn analyze_ast(file_path: &Path) -> Result<AstAnalysis> {
    let content = fs::read_to_string(file_path)?;
    let ast = parse_file(&content)?;

    let mut analysis = AstAnalysis::default();

    for item in &ast.items {
        analyze_item(item, &mut analysis, "");
    }

    Ok(analysis)
}

/// Recursively analyze AST items
pub fn analyze_item(
    item: &Item,
    analysis: &mut AstAnalysis,
    module_prefix: &str,
) {
    match item {
        Item::Use(item_use) => {
            analyze_use_item(item_use, analysis);
        },
        Item::Fn(item_fn) => {
            if matches!(item_fn.vis, syn::Visibility::Public(_)) {
                let name = format!("{}{}", module_prefix, item_fn.sig.ident);
                analysis.public_functions.push(name);
            }
        },
        Item::Struct(item_struct) => {
            if matches!(item_struct.vis, syn::Visibility::Public(_)) {
                let name = format!("{}{}", module_prefix, item_struct.ident);
                analysis.public_structs.push(name);
            }
        },
        Item::Enum(item_enum) => {
            if matches!(item_enum.vis, syn::Visibility::Public(_)) {
                let name = format!("{}{}", module_prefix, item_enum.ident);
                analysis.public_enums.push(name);
            }
        },
        Item::Trait(item_trait) => {
            if matches!(item_trait.vis, syn::Visibility::Public(_)) {
                let name = format!("{}{}", module_prefix, item_trait.ident);
                analysis.public_traits.push(name);
            }
        },
        Item::Const(item_const) => {
            if matches!(item_const.vis, syn::Visibility::Public(_)) {
                let name = format!("{}{}", module_prefix, item_const.ident);
                analysis.public_constants.push(name);
            }
        },
        Item::Static(item_static) => {
            if matches!(item_static.vis, syn::Visibility::Public(_)) {
                let name = format!("{}{}", module_prefix, item_static.ident);
                analysis.public_statics.push(name);
            }
        },
        Item::Mod(item_mod) => {
            if matches!(item_mod.vis, syn::Visibility::Public(_)) {
                let name = format!("{}{}", module_prefix, item_mod.ident);
                analysis.public_modules.push(name.clone());

                // Analyze items within the module if it's inline
                if let Some((_, items)) = &item_mod.content {
                    let new_prefix = format!("{}::", name);
                    for inner_item in items {
                        analyze_item(inner_item, analysis, &new_prefix);
                    }
                }
            }
        },
        Item::Macro(item_macro) => {
            // Check for #[macro_export] attribute
            for attr in &item_macro.attrs {
                if attr.path().is_ident("macro_export") {
                    if let Some(ident) = &item_macro.ident {
                        let name = format!("{}{}", module_prefix, ident);
                        analysis.macro_exports.push(name);
                    }
                }
            }
        },
        _ => {},
    }

    // Check for conditional compilation attributes
    let attrs = match item {
        Item::Fn(item_fn) => &item_fn.attrs,
        Item::Struct(item_struct) => &item_struct.attrs,
        Item::Enum(item_enum) => &item_enum.attrs,
        Item::Trait(item_trait) => &item_trait.attrs,
        Item::Const(item_const) => &item_const.attrs,
        Item::Static(item_static) => &item_static.attrs,
        Item::Mod(item_mod) => &item_mod.attrs,
        Item::Macro(item_macro) => &item_macro.attrs,
        _ => return,
    };

    for attr in attrs {
        if attr.path().is_ident("cfg") {
            let condition = "cfg".to_string(); // Simplified for now
            let item_type = match item {
                Item::Fn(_) => "function",
                Item::Struct(_) => "struct",
                Item::Enum(_) => "enum",
                Item::Trait(_) => "trait",
                Item::Const(_) => "const",
                Item::Static(_) => "static",
                Item::Mod(_) => "module",
                Item::Macro(_) => "macro",
                _ => "item",
            };

            let name = match item {
                Item::Fn(f) => f.sig.ident.to_string(),
                Item::Struct(s) => s.ident.to_string(),
                Item::Enum(e) => e.ident.to_string(),
                Item::Trait(t) => t.ident.to_string(),
                Item::Const(c) => c.ident.to_string(),
                Item::Static(s) => s.ident.to_string(),
                Item::Mod(m) => m.ident.to_string(),
                Item::Macro(m) =>
                    m.ident.as_ref().map(|i| i.to_string()).unwrap_or_default(),
                _ => "unknown".to_string(),
            };

            analysis.conditional_items.push(ConditionalItem {
                condition,
                item_type: item_type.to_string(),
                name: format!("{}{}", module_prefix, name),
            });
        }
    }
}

/// Analyze use statements
fn analyze_use_item(
    item_use: &ItemUse,
    analysis: &mut AstAnalysis,
) {
    let use_analysis = extract_use_info(&item_use.tree, "");
    analysis.pub_use_items.push(use_analysis);
}

/// Extract information from use tree
fn extract_use_info(
    tree: &UseTree,
    prefix: &str,
) -> UseAnalysis {
    match tree {
        UseTree::Path(path) => {
            let new_prefix = if prefix.is_empty() {
                path.ident.to_string()
            } else {
                format!("{}::{}", prefix, path.ident)
            };
            extract_use_info(&path.tree, &new_prefix)
        },
        UseTree::Name(name) => UseAnalysis {
            path: prefix.to_string(),
            items: vec![name.ident.to_string()],
            is_nested: false,
            has_conditions: false,
        },
        UseTree::Rename(rename) => UseAnalysis {
            path: prefix.to_string(),
            items: vec![rename.rename.to_string()],
            is_nested: false,
            has_conditions: false,
        },
        UseTree::Glob(_) => UseAnalysis {
            path: prefix.to_string(),
            items: vec!["*".to_string()],
            is_nested: false,
            has_conditions: false,
        },
        UseTree::Group(group) => {
            let mut items = Vec::new();
            for item in &group.items {
                let sub_analysis = extract_use_info(item, "");
                items.extend(sub_analysis.items);
            }
            UseAnalysis {
                path: prefix.to_string(),
                items,
                is_nested: true,
                has_conditions: false,
            }
        },
    }
}
