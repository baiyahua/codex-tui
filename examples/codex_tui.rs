use color_eyre::Result;
use crossterm::event::{self, KeyCode, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Layout, Position, Rect},
    style::{Color, Modifier, Style, Stylize},
    text::{Line, Span},
    widgets::{Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState, Wrap},
};

fn main() -> Result<()> {
    color_eyre::install()?;
    ratatui::run(|terminal| App::default().run(terminal))
}

#[derive(Debug)]
enum Role {
    User,
    Assistant,
}

#[derive(Debug)]
struct Message {
    role: Role,
    content: String,
}

#[derive(Debug, Default)]
struct App {
    input: String,
    messages: Vec<Message>,
    scroll: u16,
    max_scroll: u16,
    should_quit: bool,
}

impl App {
    fn run(mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.should_quit {
            terminal.draw(|frame| self.render(frame))?;
            self.handle_event()?;
        }
        Ok(())
    }

    fn handle_event(&mut self) -> Result<()> {
        let Some(key) = event::read()?.as_key_press_event() else {
            return Ok(());
        };

        match (key.modifiers, key.code) {
            (KeyModifiers::CONTROL, KeyCode::Char('c')) => self.should_quit = true,
            (_, KeyCode::Esc) => self.should_quit = true,
            (_, KeyCode::Enter) => self.submit(),
            (_, KeyCode::Backspace) => {
                self.input.pop();
            }
            (_, KeyCode::Up) => self.scroll = self.scroll.saturating_sub(1),
            (_, KeyCode::Down) => {
                self.scroll = self.scroll.saturating_add(1).min(self.max_scroll);
            }
            (_, KeyCode::PageUp) => self.scroll = self.scroll.saturating_sub(5),
            (_, KeyCode::PageDown) => {
                self.scroll = self.scroll.saturating_add(5).min(self.max_scroll);
            }
            (_, KeyCode::Home) => self.scroll = 0,
            (_, KeyCode::End) => self.scroll = self.max_scroll,
            (_, KeyCode::Char(character)) => self.input.push(character),
            _ => {}
        }

        Ok(())
    }

    fn submit(&mut self) {
        let input = self.input.trim();
        if input.is_empty() {
            return;
        }

        self.messages.push(Message {
            role: Role::User,
            content: input.to_owned(),
        });
        self.messages.push(Message {
            role: Role::Assistant,
            content: format!("这是第一阶段的模拟回复。你输入了：{input}"),
        });
        self.input.clear();
        self.scroll = u16::MAX;
    }

    fn render(&mut self, frame: &mut Frame) {
        // 左右边框占 2 列，提示符 "> " 占 2 列。
        let input_content_width = frame.area().width.saturating_sub(4).max(1) as usize;
        let input_rows = Line::from(self.input.as_str())
            .width()
            .max(1)
            .div_ceil(input_content_width);
        // 上下边框各占一行。
        let input_height = u16::try_from(input_rows)
            .unwrap_or(u16::MAX)
            .saturating_add(2);

        let [header_area, messages_area, input_area, footer_area] =
            frame.area().layout(&Layout::vertical([
                Constraint::Length(1),
                Constraint::Fill(1),
                Constraint::Length(input_height),
                Constraint::Length(1),
            ]));

        self.render_header(frame, header_area);
        self.render_messages(frame, messages_area);
        self.render_input(frame, input_area);
        self.render_footer(frame, footer_area);
    }

    fn render_header(&self, frame: &mut Frame, area: Rect) {
        let header = Line::from(vec![
            Span::styled(" codex-tui ", Style::new().black().on_cyan().bold()),
            Span::styled("  local session", Style::new().dark_gray()),
        ]);
        frame.render_widget(header, area);
    }

    fn render_messages(&mut self, frame: &mut Frame, area: Rect) {
        let mut lines = Vec::new();

        if self.messages.is_empty() {
            lines.push(Line::from("开始输入一条消息吧。").dark_gray());
        }

        for message in &self.messages {
            let (label, style) = match message.role {
                Role::User => ("You", Style::new().cyan().add_modifier(Modifier::BOLD)),
                Role::Assistant => ("Codex", Style::new().green().add_modifier(Modifier::BOLD)),
            };

            lines.push(Line::from(Span::styled(label, style)));
            lines.extend(message.content.lines().map(Line::from));
            lines.push(Line::default());
        }

        let content_width = area.width.saturating_sub(2).max(1) as usize;
        let line_count: usize = lines
            .iter()
            .map(|line| line.width().max(1).div_ceil(content_width))
            .sum();

        let block = Block::new()
            .borders(Borders::TOP)
            .border_style(Color::DarkGray);
        let paragraph = Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false });

        let content_height = area.height.saturating_sub(1) as usize;
        self.max_scroll = line_count.saturating_sub(content_height) as u16;
        self.scroll = self.scroll.min(self.max_scroll);

        frame.render_widget(paragraph.scroll((self.scroll, 0)), area);

        if self.max_scroll > 0 {
            let mut scrollbar_state =
                ScrollbarState::new(self.max_scroll as usize).position(self.scroll as usize);
            frame.render_stateful_widget(
                Scrollbar::new(ScrollbarOrientation::VerticalRight),
                area,
                &mut scrollbar_state,
            );
        }
    }

    fn render_input(&self, frame: &mut Frame, area: Rect) {
        let block = Block::bordered()
            .title(" Message ")
            .border_style(Color::Cyan);
        let inner = block.inner(area);
        frame.render_widget(block, area);

        let prefix = "> ";
        let prefix_width = Line::from(prefix).width() as u16;
        let [prefix_area, content_area] = inner.layout(&Layout::horizontal([
            Constraint::Length(prefix_width),
            Constraint::Fill(1),
        ]));

        frame.render_widget(Paragraph::new(prefix).dark_gray(), prefix_area);
        frame.render_widget(
            Paragraph::new(self.input.as_str()).wrap(Wrap { trim: false }),
            content_area,
        );

        let content_width = content_area.width.max(1) as usize;
        let cursor_offset = Line::from(self.input.as_str()).width();
        let cursor_row = cursor_offset / content_width;
        let cursor_column = cursor_offset % content_width;
        let cursor_x = content_area
            .x
            .saturating_add(u16::try_from(cursor_column).unwrap_or(u16::MAX));
        let cursor_y = content_area
            .y
            .saturating_add(u16::try_from(cursor_row).unwrap_or(u16::MAX));
        frame.set_cursor_position(Position::new(cursor_x, cursor_y));
    }

    fn render_footer(&self, frame: &mut Frame, area: Rect) {
        let footer = Line::from(" Enter send  •  ↑/↓ PgUp/PgDn scroll  •  Esc/Ctrl-C quit ")
            .centered()
            .dark_gray();
        frame.render_widget(footer, area);
    }
}
