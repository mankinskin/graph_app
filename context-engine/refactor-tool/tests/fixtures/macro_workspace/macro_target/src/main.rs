use macro_source::{
    debug_function,
    debug_print,
    format_internal,
    hashmap,
    assert_msg,
    MacroHelper,
};

fn main() {
    debug_print!("Hello from macro!");

    let helper = MacroHelper::new(100);
    println!("Helper value: {}", helper.value);

    println!("{}", debug_function());

    // Use externally-defined macros
    let data = hashmap! {
        "test" => 42,
        "demo" => 99,
    };
    println!("Data: {:?}", data);

    assert_msg!(data.len() == 2, "Expected 2 items in map");

    println!("Formatted: {}", format_internal("test"));
}
