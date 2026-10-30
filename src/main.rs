use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyModifiers, MouseEventKind,
    },
    execute,
};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Layout, Position, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Span},
    widgets::{
        Block, Padding, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Widget, Wrap,
    },
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
    scroll_from_bottom: u16,
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
        if self.messages.is_empty() {
            return;
        }

        let mut y = area.y;

        let heights: Vec<u16> = self
            .messages
            .iter()
            .map(|msg| {
                let width = Line::from(msg.content.as_str()).width() as u16;
                width.div_ceil(area.width - 2).max(1) + 2
            })
            .collect();

        let total_height = heights.iter().sum::<u16>();

        let canvas = Rect {
            x: 0,
            y: 0,
            width: area.width,
            height: total_height,
        };
        let mut buffer = Buffer::empty(canvas);

        for (height, msg) in heights.iter().zip(self.messages.iter()) {
            let (prefix, color) = match msg.role {
                Role::User => ("> ", Color::Rgb(72, 39, 62)),
                Role::Assistant => ("* ", Color::Rgb(48, 10, 36)),
            };

            let rect = Rect {
                x: 0,
                y: y,
                width: area.width,
                height: *height,
            };

            let block = Block::default().padding(Padding::vertical(1)).bg(color);
            let inner = block.inner(rect);
            block.render(rect, &mut buffer);

            let [prefix_area, content_area] = inner.layout(&Layout::horizontal(vec![
                Constraint::Length(2),
                Constraint::Fill(1),
            ]));

            Paragraph::new(prefix).render(prefix_area, &mut buffer);
            Paragraph::new(msg.content.as_str())
                .wrap(Wrap { trim: false })
                .render(content_area, &mut buffer);

            y += height;
        }

        let view_height = area.height.min(total_height);
        let max_scroll_height = total_height.saturating_sub(view_height);
        let begin = max_scroll_height.saturating_sub(self.scroll_from_bottom);
        let target = frame.buffer_mut();

        for y in 0..view_height {
            for x in 0..area.width {
                let source = buffer[(x, begin + y)].clone();
                target[(x, y)] = source;
            }
        }

        if total_height > view_height {
            let mut scrollbar_state = ScrollbarState::new(total_height as usize)
                .position(begin as usize)
                .viewport_content_length(view_height as usize);

            frame.render_stateful_widget(
                Scrollbar::new(ScrollbarOrientation::VerticalRight),
                area,
                &mut scrollbar_state,
            );
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
        match event::read()? {
            Event::Mouse(mouse) => match mouse.kind {
                MouseEventKind::ScrollUp => {
                    self.scroll_from_bottom = self.scroll_from_bottom.saturating_add(2);
                }
                MouseEventKind::ScrollDown => {
                    self.scroll_from_bottom = self.scroll_from_bottom.saturating_sub(2);
                }
                _ => {}
            },
            Event::Key(key) => match key.code {
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
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
            },
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

        self.messages.push(Message {
            role: Role::Assistant,
            content: "this is a testing data1".to_string(),
        });
    }
}

struct MouseCaptureGuard;

impl MouseCaptureGuard {
    fn enable() -> Self {
        let _ = execute!(std::io::stdout(), EnableMouseCapture);
        Self
    }
}

impl Drop for MouseCaptureGuard {
    fn drop(&mut self) {
        let _ = execute!(std::io::stdout(), DisableMouseCapture);
    }
}

fn main() -> anyhow::Result<()> {
    let mut app = App::default();
    app.running = true;
    let _guard = MouseCaptureGuard::enable();
    ratatui::run(|terminal| app.run(terminal))
}
