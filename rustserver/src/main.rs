use std::fmt::Debug;
use axum::http::StatusCode;
use axum::Router;
use axum::routing::{get, post};
use tracing::{error, info};



#[derive(Debug, serde::Deserialize, serde::Serialize, Clone, PartialEq, Eq)]
struct InventoryReport {
    common_name: String,
    computer_id: i64,
    inventory: Vec<InventoryItem>,
    peripheral_name: String,
    inventory_type: InventoryType,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone, PartialEq, Eq)]
struct InventoryItem {
    slot: i64,
    name: String,
    count: i64,
}

#[derive(Debug, serde::Deserialize, serde::Serialize, Clone, PartialEq, Eq)]
enum InventoryType {
    #[serde(rename = "input")]
    Input{
        destination: String,
    },
    #[serde(rename = "output")]
    Output{
        source: String,
    },
    #[serde(rename = "storage")]
    Storage
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let app = Router::new()
        .route("/", get(|| async {"hello world"}))
        .route("/", post(print_json));

    let listener =  tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap_or_else(|e| {
        error!("Failed to bind to address: {}", e);
        std::process::exit(1);
    });
    info!("Starting server on {}", listener.local_addr().expect("Failed to get local address we bound to"));
    axum::serve(listener, app).await.unwrap_or_else(|e| {
        error!("Failed to start server: {}", e);
    });
}

async fn print_json(axum::Json(json): axum::Json<serde_json::Value>) -> (StatusCode, String) {
    match serde_json::from_value::<InventoryReport>(json.clone()) {
        Ok(report) => {
            info!("deserialized report: {:?}", report);
            (StatusCode::OK, "Ok".to_string())
        }
        Err(e) => {
            error!("Failed to deserialize JSON: {}", e);
            let pretty = serde_json::to_string_pretty(&json).unwrap_or_else(|e| {
                error!("Failed to serialize JSON: {}", e);
                "Failed to serialize JSON".to_string()
            });
            error!("Failed to deserialize JSON: {}", pretty);
            (StatusCode::BAD_REQUEST, format!("Failed to serialize JSON: {}", e))
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize() {
        let report = InventoryReport {
            common_name: "Test Computer".to_string(),
            computer_id: 12345,
            inventory: vec![
                InventoryItem {
                    slot: 1,
                    name: "Test Item".to_string(),
                    count: 10,
                },
            ],
            peripheral_name: "Test Peripheral".to_string(),
            inventory_type: InventoryType::Input {
                destination: "Test Destination".to_string(),
            },
        };

        let serialized = serde_json::to_string(&report).unwrap();
        assert_eq!(
            serialized,
            r#"{"common_name":"Test Computer","computer_id":12345,"inventory":[{"slot":1,"name":"Test Item","count":10}],"peripheral_name":"Test Peripheral","inventory_type":{"input":{"destination":"Test Destination"}}}"#
        );

        let report = InventoryReport {
            common_name: "Test Computer".to_string(),
            computer_id: 12345,
            inventory: vec![
                InventoryItem {
                    slot: 1,
                    name: "Test Item".to_string(),
                    count: 10,
                },
            ],
            peripheral_name: "Test Peripheral".to_string(),
            inventory_type: InventoryType::Storage
        };

        let serialized = serde_json::to_string(&report).unwrap();
        assert_eq!(
            serialized,
            r#"{"common_name":"Test Computer","computer_id":12345,"inventory":[{"slot":1,"name":"Test Item","count":10}],"peripheral_name":"Test Peripheral","inventory_type":"storage"}"#
        );
    }
}