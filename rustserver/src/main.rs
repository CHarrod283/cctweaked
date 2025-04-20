mod cctweaked;
pub mod inventory_manager;

use std::fmt::{Debug, Display};
use axum::extract::{ConnectInfo, State, WebSocketUpgrade};
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
use std::time::Duration;
use ratatui::crossterm::terminal::enable_raw_mode;
use ratatui::symbols::border;
use ratatui::text::Text;
use ratatui::widgets::{Block, List};
use cctweaked::CCTweakedMonitorBackend;
use crate::cctweaked::{CCTweakedMonitorBackendEvent, CCTweakedMonitorInputEvent, MonitorInputHandler, MonitorOutputHandler};
use crate::inventory_manager::{InventoryManager, InventoryManagerReport, InventoryReport};

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let (manager_sender, manager_receiver) = tokio::sync::mpsc::unbounded_channel::<InventoryReport>();
    
    let manager = Arc::new(InventoryManager::new(manager_sender));
    let manager_clone = manager.clone();
    tokio::spawn(async move { 
        manager_clone.run(manager_receiver).await;
    });
    
    let app = Router::new()
        .route("/", get(|| async {"hello world"}))
        .route("/ws/monitor", any(terminal_handler))
        .with_state(manager);

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
    State(manager): State<Arc<InventoryManager>>,
) -> impl IntoResponse {
    let user_agent = if let Some(TypedHeader(user_agent)) = user_agent {
        user_agent.to_string()
    } else {
        String::from("Unknown browser")
    };
    info!("`{user_agent}` at {addr} connected.");
    // finalize the upgrade process by returning upgrade callback.
    // we can customize the callback by sending additional info such as address.
    ws.on_upgrade(move |socket| handle_socket(socket, addr, manager))
}

async fn handle_socket(mut socket: WebSocket, addr: SocketAddr, manager: Arc<InventoryManager>) {
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
    let (computer_id, common_name, size) = match initial_monitor_size {
        CCTweakedMonitorInputEvent::InventoryRegister { size, computer_id, common_name} => {
            info!("Registering computer id {computer_id} with common name {common_name}");
            (computer_id, common_name, size)
        }
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

    // input and output handlers can see the websocket is closed, but the terminal writer cant,
    // so we need to send a hangup signal to the terminal writer when the websocket is closed to avoid
    // leaking tasks
    let (hangup_sender, hangup_receiver) = tokio::sync::oneshot::channel();

    let input_handler = MonitorInputHandler::new(socket_receiver, terminal.clone());
    let manager_sender = manager.get_sender();
    tokio::spawn(async move {
        input_handler.handle_inbound(manager_sender).await;
    });

    let output_handler = MonitorOutputHandler::new(event_receiver, socket_sender, hangup_sender);
    tokio::spawn(async move {
        output_handler.handle_outbound().await;
    });

    select! {
        _ = write_inventory_manager_rate_report_to_terminal(terminal.clone(), manager, computer_id, common_name) => {},
        _ = hangup_receiver => {
            info!("Hangup received, closing terminal");
        }
    }


}


async fn write_inventory_manager_rate_report_to_terminal(terminal: Arc<Mutex<Terminal<CCTweakedMonitorBackend>>>, manager: Arc<InventoryManager>, computer_id: i64, common_name: String) {
    let mut timer = tokio::time::interval(Duration::from_millis(1000));
    loop {
        timer.tick().await;
        let Some(report) = manager.get_report(computer_id, Duration::from_secs(5 * 60)).await else {
            // just havent received any reports yet
            continue;
        };
        let display = match report {
            InventoryManagerReport::Input(mut r) => {
                r.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal).reverse());
                List::new(r.iter().map(|item| {
                    let text = Text::raw(format!("{}: {}", item.name, item.rate_per_second));
                    text
                })).block(Block::bordered().border_set(CCTWEAKED_BORDER).title(common_name.clone()))
            }
            InventoryManagerReport::Output(mut r)  => {
                r.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal).reverse());
                List::new(r.iter().map(|item| {
                    let text = Text::raw(format!("{}: {}", item.name, item.rate_per_second));
                    text
                })).block(Block::bordered().border_set(CCTWEAKED_BORDER).title(common_name.clone()))
            }
            InventoryManagerReport::Storage(mut r) => {
                r.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal).reverse());
                List::new(r.iter().map(|item| {
                    let text = Text::raw(format!("{}: {}", item.name, item.count));
                    text
                })).block(Block::bordered().border_set(CCTWEAKED_BORDER).title(common_name.clone()))
            }
        };
        let mut guard = terminal.lock().await;
        let Ok(_frame) = guard.draw(|frame| {
            frame.render_widget(display, frame.area());
        }).map_err(|e| {
            if e.to_string().contains("channel closed") {
                return // normal disconnect
            }
            error!("Failed to draw to terminal: {}", e);
        }) else {
            return;
        };
    }
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
    fn test_render() {
        let mut terminal = Terminal::new(TestBackend::new(20, 20)).unwrap();
        terminal.draw(| f| render(f, 1)).unwrap();
        terminal.draw(|f| render(f, 2)).unwrap();
        terminal.draw(|f| render(f, 5)).unwrap();
        terminal.draw(|f| render(f, 6)).unwrap();
        println!("Done")
    }
}