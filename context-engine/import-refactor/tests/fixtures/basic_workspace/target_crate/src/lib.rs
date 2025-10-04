use source_crate::utils::validate_input;
use source_crate::utils::string_ops::reverse_string;
use source_crate::utils::file_ops::metadata::get_size_category;
use source_crate::network::tcp::buffer::create_buffer;
use source_crate::network::protocols::tls::cipher::default_suite;
use source_crate::GLOBAL_STATE;

pub fn helper_function() -> bool {
    validate_input(GLOBAL_STATE)
}

pub fn string_helper() -> String {
    reverse_string("hello")
}

pub fn categorize_file_size(size: u64) -> &'static str {
    get_size_category(size)
}

pub fn network_buffer_helper() -> Vec<u8> {
    create_buffer(1024)
}

pub fn crypto_helper() -> &'static str {
    default_suite()
}