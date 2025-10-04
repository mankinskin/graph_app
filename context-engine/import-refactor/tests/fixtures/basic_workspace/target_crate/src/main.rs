use source_crate::{Config, main_function, MAGIC_NUMBER};
use source_crate::math::{add, subtract};
use source_crate::math::advanced::Calculator;
use source_crate::math::advanced::scientific::{power, AdvancedCalculator};
use source_crate::math::advanced::scientific::statistics::{mean, StatEngine};
use source_crate::math::advanced::geometry::{Point, area_circle};
use source_crate::math::operations::factorial;
use source_crate::math::operations::matrix::{transpose, MatrixProcessor};
use source_crate::utils::format_string;
use source_crate::utils::string_ops::{reverse_string, capitalize};
use source_crate::utils::string_ops::encoding::{base64_encode, Encoder};
use source_crate::utils::string_ops::parsing::{extract_numbers, Parser};
use source_crate::utils::file_ops::{get_extension, join_path};
use source_crate::utils::file_ops::compression::Compressor;
use source_crate::utils::file_ops::metadata::{FileInfo, get_size_category};
use source_crate::network::{ping, Connection};
use source_crate::network::http::{get, post};
use source_crate::network::http::headers::{content_type_json, HeaderBuilder};
use source_crate::network::http::headers::security::{cors_headers, SecurityPolicy};
use source_crate::network::http::status::{is_success, StatusCode};
use source_crate::network::tcp::{connect, TcpStream};
use source_crate::network::tcp::listener::{bind, TcpListener};
use source_crate::network::protocols::websocket::{upgrade_request, WebSocketFrame};
use source_crate::network::protocols::tls::{handshake, Certificate};
use source_crate::{Status, Processable};

fn main() {
    let config = Config {
        name: "test".to_string(),
        value: MAGIC_NUMBER,
    };
    
    println!("{}", main_function());
    println!("Sum: {}", add(1, 2));
    println!("Diff: {}", subtract(5, 3));
    println!("{}", format_string("hello"));
    
    let calc = Calculator { result: 0 };
    println!("Calculator result: {}", calc.result);
    
    // Test new nested functionality
    println!("Power: {}", power(2.0, 3.0));
    let advanced_calc = AdvancedCalculator { precision: 10, memory: vec![] };
    println!("Advanced calc precision: {}", advanced_calc.precision);
    
    let values = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    println!("Mean: {}", mean(&values));
    
    let stats = StatEngine { samples: values };
    println!("Stat engine samples: {:?}", stats.samples);
    
    let point = Point { x: 1.0, y: 2.0 };
    println!("Point: ({}, {})", point.x, point.y);
    println!("Circle area: {}", area_circle(5.0));
    
    println!("Factorial of 5: {}", factorial(5));
    
    let matrix = vec![vec![1.0, 2.0], vec![3.0, 4.0]];
    let transposed = transpose(&matrix);
    println!("Transposed: {:?}", transposed);
    
    let processor = MatrixProcessor { data: matrix };
    println!("Matrix processor data: {:?}", processor.data);
    
    // String operations
    println!("Reversed: {}", reverse_string("hello"));
    println!("Capitalized: {}", capitalize("hello"));
    println!("Base64: {}", base64_encode("test"));
    
    let encoder = Encoder { algorithm: "base64".to_string() };
    println!("Encoder: {}", encoder.algorithm);
    
    let numbers = extract_numbers("abc123def456");
    println!("Numbers: {:?}", numbers);
    
    let parser = Parser { delimiter: ',' };
    println!("Parser delimiter: {}", parser.delimiter);
    
    // File operations
    println!("Extension: {:?}", get_extension("file.txt"));
    println!("Path: {}", join_path(&["home", "user", "file.txt"]));
    
    let compressor = Compressor { level: 9 };
    println!("Compression level: {}", compressor.level);
    
    let file_info = FileInfo {
        size: 1024,
        created: "2023-01-01".to_string(),
        modified: "2023-01-02".to_string(),
    };
    println!("File size: {}", file_info.size);
    println!("Size category: {}", get_size_category(file_info.size));
    
    // Network operations
    println!("Ping result: {}", ping("example.com"));
    
    let connection = Connection {
        host: "example.com".to_string(),
        port: 80,
        secure: false,
    };
    println!("Connection: {}:{}", connection.host, connection.port);
    
    println!("HTTP GET: {}", get("http://example.com"));
    println!("HTTP POST: {}", post("http://example.com", "data"));
    
    println!("Content-Type: {}", content_type_json());
    
    let header_builder = HeaderBuilder {
        headers: std::collections::HashMap::new(),
    };
    println!("Header builder: {:?}", header_builder.headers);
    
    let cors = cors_headers();
    println!("CORS headers: {:?}", cors);
    
    let security_policy = SecurityPolicy {
        strict_transport_security: true,
        content_security_policy: "default-src 'self'".to_string(),
    };
    println!("Security policy: {}", security_policy.content_security_policy);
    
    println!("Status success: {}", is_success(200));
    
    let status_code = StatusCode {
        code: 200,
        message: "OK".to_string(),
    };
    println!("Status: {} {}", status_code.code, status_code.message);
    
    println!("TCP connect: {}", connect("example.com", 80));
    
    let tcp_stream = TcpStream {
        address: "example.com:80".to_string(),
        connected: true,
    };
    println!("TCP stream: {}", tcp_stream.address);
    
    println!("TCP bind: {}", bind("127.0.0.1:8080"));
    
    let tcp_listener = TcpListener {
        address: "127.0.0.1:8080".to_string(),
        backlog: 128,
    };
    println!("TCP listener: {}", tcp_listener.address);
    
    println!("WebSocket upgrade: {}", upgrade_request());
    
    let ws_frame = WebSocketFrame {
        opcode: 1,
        payload: vec![1, 2, 3],
    };
    println!("WebSocket frame opcode: {}", ws_frame.opcode);
    
    println!("TLS handshake: {}", handshake());
    
    let cert = Certificate {
        subject: "CN=example.com".to_string(),
        issuer: "CN=CA".to_string(),
        valid_until: "2024-12-31".to_string(),
    };
    println!("Certificate subject: {}", cert.subject);
    
    let status = Status::Ready;
    match status {
        Status::Ready => println!("Ready"),
        Status::Processing => println!("Processing"),
        Status::Done => println!("Done"),
    }
}