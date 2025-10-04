// Exported macro
#[macro_export]
macro_rules! debug_print {
    ($($arg:tt)*) => {
        println!("[DEBUG] {}", format!($($arg)*));
    };
}

// Non-exported macro (should not be included in pub use)
macro_rules! private_macro {
    ($x:expr) => {
        $x * 2
    };
}

#[cfg(feature = "extra_macros")]
#[macro_export]
macro_rules! extra_debug {
    ($msg:expr) => {
        println!("[EXTRA] {}", $msg);
    };
}

pub fn use_private_macro(x: i32) -> i32 {
    private_macro!(x)
}

#[cfg(feature = "debug_mode")]
pub fn debug_function() -> String {
    "Debug mode enabled".to_string()
}

#[cfg(not(feature = "debug_mode"))]
pub fn debug_function() -> String {
    "Debug mode disabled".to_string()
}

pub struct MacroHelper {
    pub value: i32,
}

impl MacroHelper {
    pub fn new(value: i32) -> Self {
        Self { value }
    }
}

// Test function (should be detected and exported conditionally)
#[cfg(test)]
pub fn test_setup() -> MacroHelper {
    MacroHelper::new(42)
}

#[test]
fn test_macro_functionality() {
    let helper = test_setup();
    assert_eq!(helper.value, 42);
}