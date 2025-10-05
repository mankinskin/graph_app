// Utility macros defined in a separate module file
// These should be detected by the refactor tool when scanning the entire source crate

/// A macro for creating HashMap with initial values
#[macro_export]
macro_rules! hashmap {
    ($($key:expr => $val:expr),* $(,)?) => {
        {
            let mut map = ::std::collections::HashMap::new();
            $(
                map.insert($key, $val);
            )*
            map
        }
    };
}

/// A macro for asserting conditions with custom messages
#[macro_export]
macro_rules! assert_msg {
    ($condition:expr, $msg:expr) => {
        if !$condition {
            panic!("Assertion failed: {}", $msg);
        }
    };
}

/// A conditional macro only available with specific feature flag
#[cfg(feature = "async_support")]
#[macro_export]
macro_rules! async_block {
    ($($body:tt)*) => {
        async {
            $($body)*
        }
    };
}

// Non-exported macro (should not be detected)
macro_rules! internal_helper {
    ($x:expr) => {
        format!("Internal: {}", $x)
    };
}

/// Helper function that uses internal macro
pub fn format_internal(value: &str) -> String {
    internal_helper!(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hashmap_macro() {
        let map = hashmap! {
            "a" => 1,
            "b" => 2,
        };
        assert_eq!(map.get("a"), Some(&1));
    }

    #[test]
    fn test_assert_msg_macro() {
        assert_msg!(true, "This should pass");
    }
}