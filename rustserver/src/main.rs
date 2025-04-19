use std::fmt::{Debug, Display};
use axum::extract::{ConnectInfo, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Error, Router, ServiceExt};
use axum::routing::{any, get, post};
use axum_extra::TypedHeader;
use ratatui::backend::{Backend, ClearType, WindowSize};
use ratatui::buffer::Cell;
use ratatui::{DefaultTerminal, Terminal};
use ratatui::layout::{Position, Size};
use core::net::SocketAddr;
use std::sync::{Arc, Mutex, MutexGuard};
use axum::extract::ws::{CloseFrame, Message, Utf8Bytes, WebSocket};
use futures::{StreamExt, SinkExt};
use futures::stream::{SplitSink, SplitStream};
use ratatui::crossterm::queue;
use ratatui::prelude::{Color, Modifier};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::select;
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
        .route("/", post(print_json))
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
    let terminal_backend = Arc::new(Mutex::new(CCTweakedMonitorBackend {
        event_writer: event_writer.clone(),
        size: None,
    }));
    let (socket_sender, socket_receiver) = socket.split();
    
    let input_handler = MonitorInputHandler {
        socket_reader: socket_receiver,
        terminal_backend: terminal_backend.clone(),
    };
    tokio::spawn(async move {
        input_handler.handle_inbound().await;
    });
    
    let output_handler = MonitorOutputHandler {
        socket_writer: socket_sender,
        event_receiver,
    };
    tokio::spawn(async move {
        output_handler.handle_outbound().await;
    });
}

/// MonitorInputHandler is responsible for receiving inbound events from minecraft entities
struct MonitorInputHandler {
    socket_reader: SplitStream<WebSocket>,
    terminal_backend: Arc<Mutex<CCTweakedMonitorBackend>>
}

impl MonitorInputHandler {

    async fn handle_inbound(mut self) {
        loop {
            let msg = self.socket_reader.next().await;
            let Some(msg) = msg else {
                info!("WebSocket connection closed");
                return;
            };
            let Ok(msg) = msg.map_err(|e| {
                error!("Axum failed receiving message: {}", e);
            }) else {
                continue;
            };
            match msg {
                Message::Text(text) => {
                    info!("Received text message: {}", text);
                    let Ok(event) = serde_json::from_str::<CCTweakedMonitorInputEvent>(&text).map_err(|e| {
                        error!("Failed to deserialize message: {}", e);
                    }) else {
                        continue;
                    };
                    match event {
                        CCTweakedMonitorInputEvent::MonitorResize(size) => {
                            info!("Received monitor resize event: {:?}", size);
                            let Ok(mut guard) = self.terminal_backend.lock()
                                .map_err(|e| error!("Poisoned thread: {}", e
                            )) else {
                                return;
                            };
                            guard.size = Some(size);
                        }
                    }
                }
                Message::Binary(data) => {
                    error!("Unable to handle binary message: {:?}", data);
                    continue
                }
                Message::Close(frame) => {
                    info!("WebSocket connection closed: {:?}", frame);
                    return;
                }
                _ => {}
            }
        }

    }
}

/// MonitorOutputHandler is responsible for sending events to the minecraft entities
struct MonitorOutputHandler {
    // sends events to the terminal via websocket
    socket_writer: SplitSink<WebSocket, Message>,
    event_receiver: tokio::sync::mpsc::UnboundedReceiver<CCTweakedMonitorBackendEvent>,
}


impl MonitorOutputHandler {
    async fn handle_outbound(mut self) {
        loop {
            let Some(event) = self.event_receiver.recv().await else {
                info!("Monitor Backend Connection closed");
                return;
            };
            let Ok(data) = serde_json::to_string(&event).map_err(|e| {
                error!("Failed to serialize event: {}", e);
            }) else {
                continue;
            };
            if !data.is_ascii() {
                error!("Non-ASCII data generated: {:?}", data);
                continue;
            }
            let Ok(()) = self.socket_writer.send(Message::Text(Utf8Bytes::from(data))).await.map_err(|e| {
                error!("Failed to send event: {}", e);
            }) else {
                continue;
            };
        }
    }

}




/// Messages sent from the monitor to the server
#[derive(Debug, Clone, Serialize, Deserialize)]
enum CCTweakedMonitorInputEvent {
    #[serde(rename = "monitor_resize")]
    MonitorResize(Size)
}


/// Messages sent from the server to the monitor
#[derive(Debug, Clone, Serialize, Deserialize)]
enum CCTweakedMonitorBackendEvent {
    HideCursor,
    ShowCursor,
    ClearLine,
    ClearScreen,
    SetCursorPosition(Position),
    SetTextColor(CCTweakedColor),
    SetBackgroundColor(CCTweakedColor),
    WriteText(String),
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
enum CCTweakedColor {
    White,
    Orange,
    Magenta,
    LightBlue,
    Yellow,
    Lime,
    Pink,
    Gray,
    Cyan,
    Purple,
    Blue,
    Brown,
    Green,
    Red,
    Black
}

#[derive(Debug, Clone, Copy, Error)]
struct CCTweakedColorConversionError(Color);

impl Display for CCTweakedColorConversionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed to convert color: {:?}", self.0)
    }
}

impl TryFrom<Color> for CCTweakedColor {

    type Error = CCTweakedColorConversionError;

    fn try_from(value: Color) -> Result<Self, Self::Error> {
        match value {
            Color::White => Ok(CCTweakedColor::White),
            Color::Magenta => Ok(CCTweakedColor::Magenta),
            Color::LightBlue => Ok(CCTweakedColor::LightBlue),
            Color::Yellow => Ok(CCTweakedColor::Yellow),
            Color::Gray => Ok(CCTweakedColor::Gray),
            Color::Cyan => Ok(CCTweakedColor::Cyan),
            Color::Blue => Ok(CCTweakedColor::Blue),
            Color::Green => Ok(CCTweakedColor::Green),
            Color::Red => Ok(CCTweakedColor::Red),
            Color::Black => Ok(CCTweakedColor::Black),
            s => {
                Err(CCTweakedColorConversionError(s))
            }
        }
    }
}

struct CCTweakedMonitorBackend {
    event_writer: tokio::sync::mpsc::UnboundedSender<CCTweakedMonitorBackendEvent>,
    // size of the terminal, none if we haven't setup the monitor yet
    size: Option<Size>,
}


impl Backend for CCTweakedMonitorBackend {
    
    fn draw<'a, I>(&mut self, content: I) -> std::io::Result<()>
    where
        I: Iterator<Item=(u16, u16, &'a Cell)>
    {
        let mut fg = Color::White;
        let mut bg = Color::Black;
        for (x, y, cell) in content {
            if cell.skip {
                continue;
            }
            // Move the cursor if the previous location was not (x - 1, y)
            self.event_writer.send(CCTweakedMonitorBackendEvent::SetCursorPosition(Position { x, y })).map_err(|e| {
                std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to send event: {}", e))
            })?;
            if cell.fg != fg || cell.bg != bg {
                self.event_writer.send(CCTweakedMonitorBackendEvent::SetTextColor(
                    CCTweakedColor::try_from(cell.fg).unwrap_or_else(|e|{
                        error!("Failed to convert color: {}", e);
                        CCTweakedColor::White
                    })
                )).map_err(|e| {
                    std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to send event: {}", e))
                })?;
                self.event_writer.send(CCTweakedMonitorBackendEvent::SetBackgroundColor(
                    CCTweakedColor::try_from(cell.bg).unwrap_or_else(|e|{
                        error!("Failed to convert color: {}", e);
                        CCTweakedColor::Black
                    })
                )).map_err(|e| {
                    std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to send event: {}", e))
                })?;
                fg = cell.fg;
                bg = cell.bg;
            }

            if !cell.symbol().is_ascii() {
                return Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Non-ASCII character: {}", cell.symbol())));
            }
            self.event_writer.send(CCTweakedMonitorBackendEvent::WriteText(cell.symbol().to_string())).map_err(|e| {
                std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to send event: {}", e))
            })?;
        }

        Ok(())
    }

    fn hide_cursor(&mut self) -> std::io::Result<()> {
        self.event_writer.send(CCTweakedMonitorBackendEvent::HideCursor).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to send event: {}", e))
        })
    }

    fn show_cursor(&mut self) -> std::io::Result<()> {
        self.event_writer.send(CCTweakedMonitorBackendEvent::ShowCursor).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to send event: {}", e))
        })
    }

    fn get_cursor_position(&mut self) -> std::io::Result<Position> {
        todo!("maybe not needed");
    }

    fn set_cursor_position<P: Into<Position>>(&mut self, position: P) -> std::io::Result<()> {
        self.event_writer.send(CCTweakedMonitorBackendEvent::SetCursorPosition(position.into())).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to send event: {}", e))
        })
    }

    fn clear(&mut self) -> std::io::Result<()> {
        self.event_writer.send(CCTweakedMonitorBackendEvent::ClearScreen).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to send event: {}", e))
        })
    }

    fn clear_region(&mut self, clear_type: ClearType) -> std::io::Result<()> {
        match clear_type {
            ClearType::All => self.clear(),
            ClearType::CurrentLine => self.event_writer.send(CCTweakedMonitorBackendEvent::ClearLine).map_err(|e| {
                std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to send event: {}", e))
            }),
            ClearType::AfterCursor => unimplemented!("Not supported by cctweaked"),
            ClearType::UntilNewLine => unimplemented!("Not supported by cctweaked"),
            ClearType::BeforeCursor => unimplemented!("Not supported by cctweaked")
        }
    }

    fn size(&self) -> std::io::Result<Size> {
        let Some(size) = self.size else {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "Size not set"));
        };
        Ok(size)
    }

    fn window_size(&mut self) -> std::io::Result<WindowSize> {
         Err(std::io::Error::new(std::io::ErrorKind::Other, "Not supported by computer craft, use size() instead"))
    }

    fn flush(&mut self) -> std::io::Result<()> {
        // No-op since sending events in handled by websocket
        Ok(())
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
        
        let monitor_resize = CCTweakedMonitorInputEvent::MonitorResize(Size { width: 10, height: 20 });
        let serialized = serde_json::to_string(&monitor_resize).unwrap();
        assert_eq!(
            serialized,
            r#"{"monitor_resize":{"width":10,"height":20}}"#
        );
    }
}