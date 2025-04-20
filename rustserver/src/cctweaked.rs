use std::fmt::Display;
use std::io::BufWriter;
use ratatui::backend::{Backend, ClearType, WindowSize};
use ratatui::buffer::Cell;
use ratatui::layout::{Position, Size};
use ratatui::prelude::Color;
use tracing::{error, info, trace};
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
            let message = match event {
                CCTweakedMonitorBackendEvent::WriteText(word) => {
                    let Ok(word) = translate_to_cctweaked(&word).map_err(|e| {
                        error!("Failed to translate word: {}", e);
                    }) else {
                        continue;
                    };
                    Message::Binary(word.into())
                }
                e => {
                    let Ok(data) = serde_json::to_string(&e).map_err(|e| {
                        error!("Failed to serialize event: {}", e);
                    }) else {
                        continue;
                    };
                    Message::Text(Utf8Bytes::from(data))
                }
            };
            
            let Ok(()) = self.socket_writer.send(message).await.map_err(|e| {
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


fn translate_to_cctweaked(word: &str) -> Result<Vec<u8>, CharTranslationError> {
    let mut result = Vec::new();
    for c in word.chars() {
        if let Some(byte) = translate_char_to_cctweaked_byte(c) {
            result.push(byte);
        } else {
            return Err(CharTranslationError(c));
        }
    }
    Ok(result)
}

fn translate_char_to_cctweaked_byte(c: char) -> Option<u8> {
    if c.is_ascii() {
        return Some(c as u8);
    }
    // Handle special characters
    match c {
        //  |0 1 2 3 4 5 6 7 8 9 A B C D E F
        // -+--------------------------------
        // 0|  ☺ ☻ ♥ ♦ ♣ ♠ ● ○     ♂ ♀   ♪ ♬
        '☺' => Some(0x01),
        '☻' => Some(0x02),
        '♥' => Some(0x03),
        '♦' => Some(0x04),
        '♣' => Some(0x05),
        '♠' => Some(0x06),
        '●' => Some(0x07),
        '○' => Some(0x08),
        '♂' => Some(0x0b),
        '♀' => Some(0x0c),
        '♪' => Some(0x0e),
        '♬' => Some(0x0f),
        // 1|▶ ◀ ↕ ‼ ¶ ░ ▬ ↨ ⬆ ⬇ ➡ ⬅ ∟ ⧺ ▲ ▼
        '▶' => Some(0x10),
        '◀' => Some(0x11),
        '↕' => Some(0x12),
        '‼' => Some(0x13),
        '¶' => Some(0x14),
        '░' => Some(0x15),
        '▬' => Some(0x16),
        '↨' => Some(0x17),
        '⬆' => Some(0x18),
        '⬇' => Some(0x19),
        '➡' => Some(0x1a),
        '⬅' => Some(0x1b),
        '∟' => Some(0x1c),
        '⧺' => Some(0x1d),
        '▲' => Some(0x1e),
        '▼' => Some(0x1f),
        // ascii
        '▒' => Some(0x7f),
        // 8|⠀ ⠁ ⠈ ⠉ ⠂ ⠃ ⠊ ⠋ ⠐ ⠑ ⠘ ⠙ ⠒ ⠓ ⠚ ⠛
        '🬀' => Some(0x81),
        '🬁' => Some(0x82),
        '🬂' => Some(0x83),
        '🬃' => Some(0x84),
        '🬄' => Some(0x85),
        '🬅' => Some(0x86),
        '🬆' => Some(0x87),
        '🬇' => Some(0x88),
        '🬈' => Some(0x89),
        '🬉' => Some(0x8a),
        '🬊' => Some(0x8b),
        '🬋' => Some(0x8c),
        '🬌' => Some(0x8d),
        '🬍' => Some(0x8e),
        '🬎' => Some(0x8f),
        // 9|⠄ ⠅ ⠌ ⠍ ⠆ ⠇ ⠎ ⠏ ⠔ ⠕ ⠜ ⠝ ⠖ ⠗ ⠞ ⠟
        '🬏' => Some(0x90),
        '🬐' => Some(0x91),
        '🬑' => Some(0x92),
        '🬒' => Some(0x93),
        '🬓' => Some(0x94),
        '▌' => Some(0x95),
        '🬔' => Some(0x96),
        '🬕' => Some(0x97),
        '🬖' => Some(0x98),
        '🬗' => Some(0x99),
        '🬘' => Some(0x9a),
        '🬙' => Some(0x9b),
        '🬚' => Some(0x9c),
        '🬛' => Some(0x9d),
        '🬜' => Some(0x9e),
        '🬝' => Some(0x9f),
        // A|▓ ¡ ¢ £ ¤ ¥ ¦ █ ¨ © ª « ¬ ­ ® ¯
        '▓' => Some(0xa0),
        '¡' => Some(0xa1),
        '¢' => Some(0xa2),
        '£' => Some(0xa3),
        '¤' => Some(0xa4),
        '¥' => Some(0xa5),
        '¦' => Some(0xa6),
        '¨' => Some(0xa8),
        '©' => Some(0xa9),
        'ª' => Some(0xaa),
        '«' => Some(0xab),
        '¬' => Some(0xac),
        '\u{AD}' => Some(0xad),
        '®' => Some(0xae),
        '¯' => Some(0xaf),
        // B|° ± ² ³ ´ µ ¶ · ¸ ¹ º » ¼ ½ ¾ ¿
        '°' => Some(0xb0),
        '±' => Some(0xb1),
        '²' => Some(0xb2),
        '³' => Some(0xb3),
        '´' => Some(0xb4),
        'µ' => Some(0xb5),
        //'¶' => Some(0xb6),
        '·' => Some(0xb7),
        '¸' => Some(0xb8),
        '¹' => Some(0xb9),
        'º' => Some(0xba),
        '»' => Some(0xbb),
        '¼' => Some(0xbc),
        '½' => Some(0xbd),
        '¾' => Some(0xbe),
        '¿' => Some(0xbf),
        // C|À Á Â Ã Ä Å Æ Ç È É Ê Ë Ì Í Î Ï
        'À' => Some(0xc0),
        'Á' => Some(0xc1),
        'Â' => Some(0xc2),
        'Ã' => Some(0xc3),
        'Ä' => Some(0xc4),
        'Å' => Some(0xc5),
        'Æ' => Some(0xc6),
        'Ç' => Some(0xc7),
        'È' => Some(0xc8),
        'É' => Some(0xc9),
        'Ê' => Some(0xca),
        'Ë' => Some(0xcb),
        'Ì' => Some(0xcc),
        'Í' => Some(0xcd),
        'Î' => Some(0xce),
        'Ï' => Some(0xcf),
        // D|Ð Ñ Ò Ó Ô Õ Ö × Ø Ù Ú Û Ü Ý Þ ß
        'Ð' => Some(0xd0),
        'Ñ' => Some(0xd1),
        'Ò' => Some(0xd2),
        'Ó' => Some(0xd3),
        'Ô' => Some(0xd4),
        'Õ' => Some(0xd5),
        'Ö' => Some(0xd6),
        '×' => Some(0xd7),
        'Ø' => Some(0xd8),
        'Ù' => Some(0xd9),
        'Ú' => Some(0xda),
        'Û' => Some(0xdb),
        'Ü' => Some(0xdc),
        'Ý' => Some(0xdd),
        'Þ' => Some(0xde),
        'ß' => Some(0xdf),
        // E|à á â ã ä å æ ç è é ê ë ì í î ï
        'à' => Some(0xe0),
        'á' => Some(0xe1),
        'â' => Some(0xe2),
        'ã' => Some(0xe3),
        'ä' => Some(0xe4),
        'å' => Some(0xe5),
        'æ' => Some(0xe6),
        'ç' => Some(0xe7),
        'è' => Some(0xe8),
        'é' => Some(0xe9),
        'ê' => Some(0xea),
        'ë' => Some(0xeb),
        'ì' => Some(0xec),
        'í' => Some(0xed),
        'î' => Some(0xee),
        'ï' => Some(0xef),
        // F|ð ñ ò ó ô õ ö ÷ ø ù ú û ü ý þ ÿ
        'ð' => Some(0xf0),
        'ñ' => Some(0xf1),
        'ò' => Some(0xf2),
        'ó' => Some(0xf3),
        'ô' => Some(0xf4),
        'õ' => Some(0xf5),
        'ö' => Some(0xf6),
        '÷' => Some(0xf7),
        'ø' => Some(0xf8),
        'ù' => Some(0xf9),
        'ú' => Some(0xfa),
        'û' => Some(0xfb),
        'ü' => Some(0xfc),
        'ý' => Some(0xfd),
        'þ' => Some(0xfe),
        'ÿ' => Some(0xff),
        _ => None,

    }
}

#[derive(Debug, Clone, Copy, Error)]
struct CharTranslationError(char);

impl Display for CharTranslationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed to translate character: {:?}", self.0)
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