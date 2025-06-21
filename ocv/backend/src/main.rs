use tiny_http::{Server, Response, Header};
use serde::{Serialize, Deserialize};
use std::str::FromStr;
use std::env;

#[derive(Serialize, Deserialize)]
struct HealthResponse {
    status: String,
    version: String,
    mem0_configured: bool,
}

fn main() {
    // Get the bind address from environment variable or use default
    let bind_address = env::var("BIND_ADDRESS").unwrap_or_else(|_| "0.0.0.0:8000".to_string());
    
    // Check if mem0 API key is configured
    let mem0_api_key = env::var("BASIC_MEM0_API_KEY").unwrap_or_default();
    let mem0_configured = !mem0_api_key.is_empty();
    
    // Create a server listening on the configured port
    let server = Server::http(&bind_address).unwrap();
    println!("Server running at http://{}/", bind_address);
    println!("mem0 API: {}", if mem0_configured { "Configured" } else { "Not configured" });

    for request in server.incoming_requests() {
        let url = request.url();
        
        match url {
            "/" => {
                let response = Response::from_string("Open Context Vault API")
                    .with_header(Header::from_str("Content-Type: text/plain").unwrap())
                    .with_header(Header::from_str("Access-Control-Allow-Origin: *").unwrap());
                request.respond(response).unwrap();
            },
            "/api/health" => {
                let health = HealthResponse {
                    status: "ok".to_string(),
                    version: env!("CARGO_PKG_VERSION").to_string(),
                    mem0_configured,
                };
                
                let json = serde_json::to_string(&health).unwrap();
                let response = Response::from_string(json)
                    .with_header(Header::from_str("Content-Type: application/json").unwrap())
                    .with_header(Header::from_str("Access-Control-Allow-Origin: *").unwrap());
                request.respond(response).unwrap();
            },
            _ => {
                let response = Response::from_string("Not Found")
                    .with_status_code(404)
                    .with_header(Header::from_str("Access-Control-Allow-Origin: *").unwrap());
                request.respond(response).unwrap();
            }
        }
    }
}
