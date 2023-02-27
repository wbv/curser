use std::io;

use tokio::time::{Duration, Instant};
use tokio_stream::StreamExt;
use tui::{
    backend::CrosstermBackend,
    style::Style,
    widgets::{Block, Borders, Paragraph},
    Frame,
};
use tui_textarea::{Input, Key, TextArea};

type Backend = tui::backend::CrosstermBackend<io::Stdout>;

fn reset_inputbox<'a>() -> TextArea<'a> {
    let mut inputbox = TextArea::default();
    inputbox.set_tab_length(0);
    inputbox.set_max_histories(0);
    inputbox.remove_line_number();
    inputbox.set_cursor_line_style(Style::default());
    inputbox
}

struct ChatState<'a> {
    status: String,
    messages: Vec<String>,
    inputbox: TextArea<'a>,
}

impl<'a> ChatState<'a> {
    pub fn new() -> io::Result<Self> {
        let status = String::new();
        let messages = Vec::new();
        let inputbox = reset_inputbox();

        Ok(Self {
            status,
            messages,
            inputbox,
        })
    }

    fn render(&self, f: &mut Frame<Backend>) {
        const UI_CONSTRAINTS: [tui::layout::Constraint; 3] = [
            tui::layout::Constraint::Length(3), // status area: 1 line (+ border top/bottom)
            tui::layout::Constraint::Min(12), // chat area: at least 10 lines (+ border top/bottom)
            tui::layout::Constraint::Length(1), // input area: just one line
        ];

        let sections = tui::layout::Layout::default()
            .direction(tui::layout::Direction::Vertical)
            .constraints(UI_CONSTRAINTS)
            .split(f.size());

        if sections.len() != 3 {
            let msg = format!(
                "Couldn't create all 3 UI sections (only got {})",
                sections.len()
            );
            render_err(f, msg.as_str());
            return;
        }

        let status = Paragraph::new(self.status.clone())
            .block(Block::default().title("status").borders(Borders::ALL));
        let messages = Paragraph::new(self.messages.join("\n"))
            .block(Block::default().title("chat").borders(Borders::ALL));
        let (status_chunk, messages_chunk, input_chunk) = (sections[0], sections[1], sections[2]);
        f.render_widget(status, status_chunk);
        f.render_widget(messages, messages_chunk);
        f.render_widget(self.inputbox.widget(), input_chunk);
    }
}

fn render_err(f: &mut Frame<Backend>, msg: &str) {
    let par = Paragraph::new(msg);
    let block = Block::default().title("UI Error");
    let par = par.block(block);
    f.render_widget(par, f.size());
}

fn random_duration() -> Duration {
    Duration::from_secs(4).mul_f64(fastrand::f64())
}

#[tokio::main]
async fn main() -> Result<(), io::Error> {
    // first: move to alternate terminal mode
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(io::stdout(), crossterm::terminal::EnterAlternateScreen,)?;

    let mut terminal = tui::Terminal::new(CrosstermBackend::new(io::stdout()))?;
    let mut state = ChatState::new()?;
    let mut input_reader = crossterm::event::EventStream::new();

    // background timer randomly adds messages
    let timer = tokio::time::sleep(random_duration());
    tokio::pin!(timer);

    // status gets updated with these counters
    let (mut rx, mut tx) = (0, 0);

    loop {
        state.status = format!("Messages sent: {tx:4}  Messages received: {rx:4}");
        terminal.draw(|f| state.render(f))?;

        tokio::select! {
            try_event = input_reader.next() => {
                // feed key events into the inputbox
                if let Some(Ok(crossterm::event::Event::Key(e))) = try_event {
                    match e.into() {
                        // Esc key --> quit
                        Input { key: Key::Esc, .. } => break,
                        // Enter key --> send message
                        Input { key: Key::Enter, .. } => {
                            let newmsg = state.inputbox.lines().join("\n");
                            state.messages.push(newmsg);
                            state.inputbox = reset_inputbox();
                            tx += 1;
                        }
                        // all other keys --> update input
                        i => {
                            state.inputbox.input(i);
                            continue
                        }
                    }
                }
            }
            // timer
            _ = &mut timer => {
                state.messages.push(String::from("hello!"));
                rx += 1;
                timer.as_mut().reset(Instant::now() + random_duration());
            }
        }
    }

    // move back to standard terminal
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(io::stdout(), crossterm::terminal::LeaveAlternateScreen)?;

    Ok(())
}
