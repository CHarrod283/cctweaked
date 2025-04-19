use std::fmt::Display;
use std::io::BufWriter;
use ratatui::backend::{Backend, ClearType, WindowSize};
use ratatui::buffer::Cell;
use ratatui::layout::{Position, Size};
use ratatui::prelude::Color;
use tracing::{error, info};
use crate::{InventoryReport};
use std::io::Write;
use std::sync::Arc;
use axum::extract::ws::{Message, Utf8Bytes, WebSocket};
use futures::stream::{SplitSink, SplitStream};
use tokio::sync::mpsc::{UnboundedSender, UnboundedReceiver};
use futures::{SinkExt, StreamExt};
use ratatui::Terminal;
use serde::{Deserialize, Serialize};
use thiserror::Error;
use tokio::sync::Mutex;

pub struct CCTweakedMonitorBackend {
    event_writer: UnboundedSender<CCTweakedMonitorBackendEvent>,
    size: Size,
    current_word: Option<BufWriter<Vec<u8>>>
}

impl CCTweakedMonitorBackend {
    pub fn new(event_writer: UnboundedSender<CCTweakedMonitorBackendEvent>, size: Size) -> Self {
        CCTweakedMonitorBackend {
            event_writer,
            size,
            current_word: None
        }
    }
    
    pub fn set_size(&mut self, size: Size) {
        self.size = size;
    }
    
    fn flush_word(&mut self) -> std::io::Result<()> {
        if let Some(word) = self.current_word.take() {
            let bytes = word.into_inner()?;
            let word = String::from_utf8(bytes).map_err(|e| {
                std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to convert bytes to string: {}", e))
            })?;
            info!("Flushing word: \"{}\"", word);
            self.event_writer.send(CCTweakedMonitorBackendEvent::WriteText(word)).map_err(|e| {
                std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to send event: {}", e))
            })?;
        }
        Ok(())
    }
}


/// MonitorOutputHandler is responsible for receiving events from the CCTweakedMonitorBackend and
/// sending them to the actual minecraft terminal via websocket.
pub struct MonitorOutputHandler {
    // sends events to the terminal via websocket
    socket_writer: SplitSink<WebSocket, Message>,
    event_receiver: UnboundedReceiver<CCTweakedMonitorBackendEvent>,
}


impl MonitorOutputHandler {
    pub fn new(event_receiver: UnboundedReceiver<CCTweakedMonitorBackendEvent>, socket_writer: SplitSink<WebSocket, Message>) -> Self {
        MonitorOutputHandler {
            socket_writer,
            event_receiver,
        }
    }
    
    pub async fn handle_outbound(mut self) {
        loop {
            let Some(event) = self.event_receiver.recv().await else {
                info!("Monitor Backend Connection closed");
                return;
            };
            info!("Sending event: {:?}", event);
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
                let message = format!("{}", e);
                if message.contains("closed connection") {
                    return
                }
                error!("Failed to send event: {}", e);
            }) else {
                return;
            };
        }
    }

}


impl Backend for CCTweakedMonitorBackend {

    fn draw<'a, I>(&mut self, content: I) -> std::io::Result<()>
    where
        I: Iterator<Item=(u16, u16, &'a Cell)>
    {
        let mut fg = Color::White;
        let mut bg = Color::Black;
        let mut last_pos: Option<Position> = None;
        for (x, y, cell) in content {
            // Move the cursor if the previous location was not (x - 1, y)
            if !matches!(last_pos, Some(p) if x == p.x + 1 && y == p.y) {
                self.flush_word()?;
                self.event_writer.send(CCTweakedMonitorBackendEvent::SetCursorPosition(Position { x, y })).map_err(|e| {
                    std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to send event: {}", e))
                })?;
            }
            last_pos = Some(Position { x, y });
            let cell_fg = if cell.fg != Color::Reset {
                cell.fg
            } else {
                Color::White
            };
            let cell_bg = if cell.bg != Color::Reset {
                cell.bg
            } else {
                Color::Black
            };
            // Move the cursor if the previous location was not (x - 1, y)

            if cell_fg != fg || cell_bg != bg {
                self.flush_word()?;
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
            if self.current_word.is_none() {
                self.current_word = Some(BufWriter::new(vec![]));
            }
            match self.current_word {
                Some(ref mut writer) => {
                    if let Err(e) = write!(writer, "{}", cell.symbol()) {
                        return Err(std::io::Error::new(std::io::ErrorKind::Other, format!("Failed to write to word: {}", e)));
                    }
                }
                None => {
                    unreachable!("current_word should never be None here");
                }
            }
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
        Ok(self.size)
    }

    fn window_size(&mut self) -> std::io::Result<WindowSize> {
        Err(std::io::Error::new(std::io::ErrorKind::Other, "Not supported by computer craft, use size() instead"))
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.flush_word()
    }
}

/// MonitorInputHandler is responsible for receiving Monitor events from the websocket and sending them to the terminal (like monitor_resize or click events).
pub struct MonitorInputHandler {
    socket_reader: SplitStream<WebSocket>,
    terminal: Arc<Mutex<Terminal<CCTweakedMonitorBackend>>>
}

impl MonitorInputHandler {
    
    pub fn new(socket_reader: SplitStream<WebSocket>, terminal: Arc<Mutex<Terminal<CCTweakedMonitorBackend>>>) -> Self {
        MonitorInputHandler {
            socket_reader,
            terminal
        }
    }

    pub async fn handle_inbound(mut self) {
        loop {
            let msg = self.socket_reader.next().await;
            let Some(msg) = msg else {
                info!("WebSocket connection closed");
                return;
            };
            let Ok(msg) = msg.map_err(|_e| {
                info!("WebSocket connection closed (reset)"); //cctweaked isnt nice when closing websockets and just sends a stream reset, causing an error
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
                            let mut guard = self.terminal.lock().await;
                            guard.backend_mut().set_size(size)
                        }
                        CCTweakedMonitorInputEvent::InventoryReport(report) => {
                            info!("Received inventory report: {:?}", report);
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




/// Messages sent from the monitor to the server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CCTweakedMonitorInputEvent {
    #[serde(rename = "monitor_resize")]
    MonitorResize(Size),
    #[serde(rename = "inventory_report")]
    InventoryReport(InventoryReport)
}


/// Messages sent from the server to the monitor
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CCTweakedMonitorBackendEvent {
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
pub enum CCTweakedColor {
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
pub struct CCTweakedColorConversionError(Color);

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
            other => {
                Err(CCTweakedColorConversionError(other))
            }
        }
    }
}




#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_size() {
        let (writer, _reader) = tokio::sync::mpsc::unbounded_channel();
        let size = Size { width: 80, height: 25 };
        let backend = CCTweakedMonitorBackend {
            event_writer: writer,
            size,
            current_word: None
        };
        assert_eq!(backend.size().unwrap(), size);
    }
    
    #[tokio::test]
    async fn test_flush() {
        let (writer, mut receiver) = tokio::sync::mpsc::unbounded_channel();
        let size = Size { width: 80, height: 25 };
        let mut current_word = Some(BufWriter::new(vec![]));
        write!(&mut current_word.as_mut().unwrap(), "Hello").unwrap();
        let mut backend = CCTweakedMonitorBackend {
            event_writer: writer,
            size,
            current_word
        };
        let result = backend.flush();
        assert!(result.is_ok());
        assert!(backend.current_word.is_none());
        // Check that the event was sent
        let event = receiver.recv().await;
        assert!(event.is_some());
        match event.unwrap() {
            CCTweakedMonitorBackendEvent::WriteText(text) => {
                assert_eq!(text, "Hello");
            }
            _ => panic!("Expected WriteText event")
        }
    }
}