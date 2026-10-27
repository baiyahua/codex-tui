use crossterm::event::{self, Event::Key, KeyCode, KeyEventKind};
use ratatui::DefaultTerminal;

fn run(terminal: &mut DefaultTerminal) -> anyhow::Result<()> {
    loop {
        terminal.draw(|frame| {
            frame.render_widget("Hello World (press q to quit)", frame.area());
        })?;

        if let Key(key_event) = event::read()? {
            if key_event.kind == KeyEventKind::Press && key_event.code == KeyCode::Char('q') {
                break Ok(());
            }
        };
    }
}

fn main() -> anyhow::Result<()> {
    ratatui::run(run)
}
