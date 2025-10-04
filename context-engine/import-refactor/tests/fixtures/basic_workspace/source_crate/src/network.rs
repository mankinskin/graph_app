pub fn ping(host: &str) -> bool {
    // Mock implementation
    !host.is_empty()
}

pub struct Connection {
    pub host: String,
    pub port: u16,
    pub secure: bool,
}

// Nested modules for network functionality
pub mod http {
    pub fn get(url: &str) -> String {
        format!("GET {}", url)
    }

    pub fn post(url: &str, data: &str) -> String {
        format!("POST {} with {}", url, data)
    }

    // Nested HTTP utilities
    pub mod headers {
        pub fn content_type_json() -> &'static str {
            "application/json"
        }

        pub fn authorization_bearer(token: &str) -> String {
            format!("Bearer {}", token)
        }

        pub struct HeaderBuilder {
            pub headers: std::collections::HashMap<String, String>,
        }

        // Deep nesting for specialized headers
        pub mod security {
            pub fn cors_headers() -> Vec<(&'static str, &'static str)> {
                vec![
                    ("Access-Control-Allow-Origin", "*"),
                    ("Access-Control-Allow-Methods", "GET, POST, PUT, DELETE"),
                ]
            }

            pub struct SecurityPolicy {
                pub strict_transport_security: bool,
                pub content_security_policy: String,
            }
        }
    }

    pub mod status {
        pub fn is_success(code: u16) -> bool {
            (200..300).contains(&code)
        }

        pub fn is_client_error(code: u16) -> bool {
            (400..500).contains(&code)
        }

        pub struct StatusCode {
            pub code: u16,
            pub message: String,
        }
    }
}

pub mod tcp {
    pub fn connect(host: &str, port: u16) -> bool {
        // Mock implementation
        !host.is_empty() && port > 0
    }

    pub struct TcpStream {
        pub address: String,
        pub connected: bool,
    }

    // Nested TCP functionality
    pub mod listener {
        pub fn bind(address: &str) -> bool {
            !address.is_empty()
        }

        pub struct TcpListener {
            pub address: String,
            pub backlog: usize,
        }
    }

    pub mod buffer {
        pub fn create_buffer(size: usize) -> Vec<u8> {
            vec![0; size]
        }

        pub struct RingBuffer {
            pub data: Vec<u8>,
            pub head: usize,
            pub tail: usize,
        }
    }
}

// Another top-level nested module
pub mod protocols {
    pub mod websocket {
        pub fn upgrade_request() -> String {
            "Upgrade: websocket".to_string()
        }

        pub struct WebSocketFrame {
            pub opcode: u8,
            pub payload: Vec<u8>,
        }

        pub mod extensions {
            pub fn compression_enabled() -> bool {
                true
            }

            pub struct PerMessageDeflate {
                pub server_max_window_bits: u8,
            }
        }
    }

    pub mod tls {
        pub fn handshake() -> bool {
            true
        }

        pub struct Certificate {
            pub subject: String,
            pub issuer: String,
            pub valid_until: String,
        }

        pub mod cipher {
            pub fn default_suite() -> &'static str {
                "TLS_AES_256_GCM_SHA384"
            }

            pub struct CipherSuite {
                pub name: String,
                pub key_size: u16,
            }
        }
    }
}