// AST manipulation and code generation

pub use self::parser::{ImportParser, ImportInfo};
pub use self::generator::{generate_nested_pub_use, build_nested_structure};
pub use self::transformer::{replace_imports_with_strategy, CrossCrateReplacementStrategy, SelfCrateReplacementStrategy};
pub use self::visitor::{parse_existing_pub_uses, merge_pub_uses};
pub use self::item_info::ItemInfo;

pub mod parser;
pub mod generator;
pub mod transformer;
pub mod visitor;
pub mod item_info;