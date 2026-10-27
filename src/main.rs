use crossterm::event::{self, KeyCode, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout, Position, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Padding, Paragraph, Wrap},
};

enum Role {
    User,
    Assistant,
}

struct Message {
    role: Role,
    content: String,
}

#[derive(Default)]
struct App {
    messages: Vec<Message>,
    input: String,
    running: bool,
}

impl App {
    fn run(&mut self, terminal: &mut DefaultTerminal) -> anyhow::Result<()> {
        while self.running {
            terminal.draw(|frame| self.render(frame))?;
            self.read_event()?;
        }
        Ok(())
    }

    fn render(&self, frame: &mut Frame) {
        let line_count = self
            .input
            .len()
            .div_ceil((frame.area().width - 2) as usize)
            .max(1)
            + 2;

        let layout = Layout::vertical(vec![
            Constraint::Length(1),
            Constraint::Fill(1),
            Constraint::Length(line_count as u16),
            Constraint::Length(1),
        ]);

        let [title_area, output_area, input_area, status_area] = frame.area().layout(&layout);

        self.render_title(frame, title_area);
        self.render_output(frame, output_area);
        self.render_input(frame, input_area);
        self.render_status(frame, status_area);
    }

    fn render_title(&self, frame: &mut Frame, area: Rect) {
        let style = Style::new().bg(Color::Blue);
        frame.render_widget(Paragraph::new("title").style(style), area);
    }

    fn render_output(&self, frame: &mut Frame, area: Rect) {
        let mut y = area.y;

        for msg in &self.messages {
            let (prefix, color) = match msg.role {
                Role::User => ("> ", Color::Rgb(72, 39, 62)),
                Role::Assistant => ("* ", Color::Rgb(48, 10, 36)),
            };

            let line = Line::from(vec![
                Span::styled(prefix, Style::default().fg(Color::Gray)),
                Span::styled(msg.content.as_str(), Style::default().fg(Color::White)),
            ]);

            let line_count = line.width().div_ceil(area.width as usize) + 2;

            let paragraph = Paragraph::new(line)
                .block(Block::default().padding(Padding::vertical(1)))
                .wrap(Wrap { trim: false })
                .bg(color);

            let rect = Rect {
                x: area.x,
                y: y,
                width: area.width,
                height: line_count as u16,
            };

            frame.render_widget(paragraph, rect);

            y += line_count as u16;
        }
    }

    fn render_input(&self, frame: &mut Frame, area: Rect) {
        let block = Block::default()
            .padding(Padding::vertical(1))
            .bg(Color::Rgb(72, 39, 62));

        let inner = block.inner(area);

        frame.render_widget(block, area);

        let [prefix_area, input_area] = inner.layout(&Layout::horizontal(vec![
            Constraint::Length(2),
            Constraint::Fill(1),
        ]));

        let prefix_paragraph = Paragraph::new("> ");
        let input_paragraph = Paragraph::new(self.input.as_str())
            .wrap(Wrap { trim: false })
            .fg(Color::White);

        frame.render_widget(prefix_paragraph, prefix_area);
        frame.render_widget(input_paragraph, input_area);

        let cursor_offset = Line::from(self.input.as_str()).width();
        let cursor_x = cursor_offset as u16 % input_area.width;
        let cursor_y = cursor_offset as u16 / input_area.width;
        let x = input_area.x + cursor_x;
        let y = input_area.y + cursor_y;

        frame.set_cursor_position(Position { x, y });
    }

    fn render_status(&self, frame: &mut Frame, area: Rect) {
        let style = Style::new().bg(Color::Red);
        frame.render_widget(Paragraph::new("titddle").style(style), area);
    }

    fn read_event(&mut self) -> anyhow::Result<()> {
        let Some(event) = event::read()?.as_key_press_event() else {
            return Ok(());
        };

        match event.code {
            KeyCode::Char('c') if event.modifiers.contains(KeyModifiers::CONTROL) => {
                self.running = false;
            }
            KeyCode::Backspace => {
                self.input.pop();
            }
            KeyCode::Char(c) => {
                self.input.push(c);
            }
            KeyCode::Enter => {
                self.submit_input();
            }
            _ => {}
        }
        Ok(())
    }

    fn submit_input(&mut self) {
        if self.input.is_empty() {
            return;
        }

        let content = std::mem::take(&mut self.input);

        self.messages.push(Message {
            role: Role::User,
            content,
        });
    }
}

fn main() -> anyhow::Result<()> {
    let mut app = App::default();
    app.running = true;
    ratatui::run(|terminal| app.run(terminal))
}
