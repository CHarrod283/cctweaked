use std::thread::sleep;
use std::time::Duration;
use ratatui::crossterm::event;
use ratatui::crossterm::event::Event;
use ratatui::{DefaultTerminal, Frame, Terminal};
use ratatui::backend::CrosstermBackend;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let terminal = Terminal::new(CrosstermBackend::new(std::io::stdout()))?;
    let result = run(terminal);
    ratatui::restore();
    result
}

fn run(mut terminal: DefaultTerminal) -> Result<(), Box<dyn std::error::Error>> {
    let mut i = 0;
    loop {
        sleep(Duration::from_millis(1000));
        terminal.draw(|frame| render(frame, i))?;
        i += 1;
    }
}

fn render(frame: &mut Frame, i: i32) {
    if i % 5 == 0 {
        frame.render_widget(format!("Woo hoo {i}"), frame.area());
    } else {
        frame.render_widget(format!("Hello world {i}"), frame.area());
    }
    
}
