mod cctweaked;

use std::fmt::{Debug, Display};
use axum::extract::{ConnectInfo, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Error, Router, ServiceExt};
use axum::routing::{any, get, post};
use axum_extra::TypedHeader;
use ratatui::backend::{Backend, ClearType, CrosstermBackend, WindowSize};
use ratatui::buffer::Cell;
use ratatui::{DefaultTerminal, Frame, Terminal, TerminalOptions};
use ratatui::layout::{Position, Size};
use core::net::SocketAddr;
use std::sync::{Arc};
use axum::extract::ws::{CloseFrame, Message, Utf8Bytes, WebSocket};
use futures::{StreamExt, SinkExt};
use futures::stream::{SplitSink, SplitStream};
use ratatui::crossterm::queue;
use ratatui::prelude::{Color, Modifier, Widget};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::select;
use tokio::sync::Mutex;
use tracing::{error, info};
use tracing::instrument::WithSubscriber;
use std::io::{stdout, BufWriter, Stdout, Write};
use ratatui::crossterm::terminal::enable_raw_mode;
use ratatui::symbols::border;
use ratatui::text::Text;
use ratatui::widgets::{Block, List};
use cctweaked::CCTweakedMonitorBackend;
use crate::cctweaked::{CCTweakedMonitorBackendEvent, CCTweakedMonitorInputEvent, MonitorInputHandler, MonitorOutputHandler};

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
        .route("/ws/monitor", any(terminal_handler));

    let listener =  tokio::net::TcpListener::bind("127.0.0.1:3000").await.unwrap_or_else(|e| {
        error!("Failed to bind to address: {}", e);
        std::process::exit(1);
    });
    info!("Starting server on {}", listener.local_addr().expect("Failed to get local address we bound to"));
    axum::serve(listener, app.into_make_service_with_connect_info::<SocketAddr>()).await.unwrap_or_else(|e| {
        error!("Failed to start server: {}", e);
    });
}


async fn terminal_handler(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    ws: WebSocketUpgrade,
    user_agent: Option<TypedHeader<headers::UserAgent>>,
) -> impl IntoResponse {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };
    info!("`{user_agent}` at {addr} connected.");
    // finalize the upgrade process by returning upgrade callback.
    // we can customize the callback by sending additional info such as address.
    ws.on_upgrade(move |socket| handle_socket(socket, addr))
}

async fn handle_socket(mut socket: WebSocket, addr: SocketAddr) {
    // Handle the WebSocket connection here
    info!("WebSocket connection established with {addr}");
    // You can send and receive messages using the `socket` object
    // For example, you can send a message to the client:

    let (event_writer, event_receiver) = tokio::sync::mpsc::unbounded_channel::<CCTweakedMonitorBackendEvent>();
    let Some(initial_monitor_size) = socket.recv().await else {
        error!("Didnt receive initial monitor size");
        return;
    };
    let Ok(initial_monitor_size) = initial_monitor_size.map_err(|e| {
        error!("Failed to receive initial monitor size: {}", e);
    }) else {
        return;
    };
    let Ok(initial_monitor_size) = initial_monitor_size.into_text().map_err(|e| {
        error!("Failed to convert initial monitor size to text: {}", e);
    }) else {
        return;
    };
    let Ok(initial_monitor_size) = serde_json::from_str::<CCTweakedMonitorInputEvent>(initial_monitor_size.as_str()).map_err(|e| {
        error!("Failed to deserialize input event: {}", e);
    }) else {
        return;
    };
    let size = match initial_monitor_size {
        CCTweakedMonitorInputEvent::MonitorResize(size) => size,
        _ => {
            error!("Expected monitor resize event, got: {:?}", initial_monitor_size);
            return;
        }
    };
        
    let terminal_backend = CCTweakedMonitorBackend::new(event_writer, size);
    let Ok(terminal) = Terminal::new(terminal_backend).map_err(|e| {
        error!("Failed to create terminal: {}", e);
    }) else {
        return;
    };

    let (socket_sender, socket_receiver) = socket.split();

    let terminal = Arc::new(Mutex::new(terminal));

    // maybe select?
    let input_handler = MonitorInputHandler::new(socket_receiver, terminal.clone());
    tokio::spawn(async move {
        input_handler.handle_inbound().await;
    });

    let output_handler = MonitorOutputHandler::new(event_receiver, socket_sender);
    tokio::spawn(async move {
        output_handler.handle_outbound().await;
    });

    tokio::spawn(async move {
        write_hello_to_terminal(terminal.clone()).await;
    });

}

async fn write_hello_to_terminal(terminal: Arc<Mutex<Terminal<CCTweakedMonitorBackend>>>) {
    let mut i = 0;
    loop {
        tokio::time::sleep(tokio::time::Duration::from_millis(1000)).await;
        let mut guard = terminal.lock().await;
        let Ok(_frame) = guard.draw(|frame| render(frame, i) ).map_err(|e| {
            if e.to_string().contains("channel closed") {
                return // normal disconnect
            }
            error!("Failed to draw to terminal: {}", e);
        }) else {
            return;
        };
        i+=1
    }

}

fn render(frame: &mut Frame, i: i32) {
    let text = if i % 5 == 0 {
        Text::raw(format!("Woo Hoo {}", i))
    } else {
        Text::raw(format!("Hello world {}", i))
    };
    let table = List::new(vec![text]).block(Block::bordered().border_set(CCTWEAKED_BORDER));
    frame.render_widget(table, frame.area());
}

/// MonitorInputHandler is responsible for receiving inbound events from minecraft entities


pub const CCTWEAKED_BORDER: border::Set = border::Set {
    top_left: "🬕",
    top_right: "🬂",
    bottom_left: "▌",
    bottom_right: " ",
    vertical_left: "▌",
    vertical_right: " ",
    horizontal_top: "🬂",
    horizontal_bottom: " ",
};






#[cfg(test)]
mod tests {
    use ratatui::backend::TestBackend;
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

        let monitor_resize = CCTweakedMonitorInputEvent::MonitorResize(Size { width: 10, height: 20 });
        let serialized = serde_json::to_string(&monitor_resize).unwrap();
        assert_eq!(
            serialized,
            r#"{"monitor_resize":{"width":10,"height":20}}"#
        );
    }

    #[test]
    fn test_render() {
        let mut terminal = Terminal::new(TestBackend::new(20, 20)).unwrap();
        terminal.draw(| f| render(f, 1)).unwrap();
        terminal.draw(|f| render(f, 2)).unwrap();
        terminal.draw(|f| render(f, 5)).unwrap();
        terminal.draw(|f| render(f, 6)).unwrap();
        println!("Done")
    }
}