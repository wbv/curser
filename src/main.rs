use std::{io, time::Duration};
use tui::{
    backend::CrosstermBackend,
    widgets::{
        Block,
        Borders,
    },
};
use tui_textarea::{Input, Key};


type Backend = tui::backend::CrosstermBackend<io::Stdout>;

struct ChatUI<'a> {
    inputbox: tui_textarea::TextArea<'a>,
}

impl<'a> ChatUI<'a> {
    pub fn new() -> io::Result<Self> {
        let backend = CrosstermBackend::new(io::stdout());
        let inputbox = tui_textarea::TextArea::default();

        Ok(Self { inputbox })
    }

    //fn draw(&mut self) -> io::Result<()> {
    //    self.terminal.draw(|f| self.render(f))?;
    //    Ok(())
    //}

    fn set_inputbox(&mut self, input: Input) {
        self.inputbox.input(input);
    }

    fn render(inputbox: &tui_textarea::TextArea, f: &mut tui::Frame<Backend>) {
        const UI_CONSTRAINTS: [tui::layout::Constraint; 3] = [
            tui::layout::Constraint::Length(3), // status area: 1 line (+ border top/bottom)
            tui::layout::Constraint::Min(12),   // chat area: at least 10 lines (+ border top/bottom)
            tui::layout::Constraint::Length(1), // input area: just one line
        ];

        let sections = tui::layout::Layout::default()
            .direction(tui::layout::Direction::Vertical)
            .constraints(UI_CONSTRAINTS)
            .split(f.size());

        if sections.len() != 3 {
            let msg = format!("Couldn't create all 3 UI sections (found {})", sections.len());
            Self::render_err(f, msg);
            return;
        }

        let (status, chat, input) = (sections[0], sections[1], sections[2]);

        // render each block with dummy text
        let text = tui::widgets::Paragraph::new("Status: stats | Other stats: here!")
            .block(Block::default()
                   .title("status")
                   .borders(Borders::ALL));
        f.render_widget(text, status);
        let text = tui::widgets::Paragraph::new("chat will go here\n and here!")
            .block(Block::default()
                   .title("chat")
                   .borders(Borders::ALL));
        f.render_widget(text, chat);
        f.render_widget(inputbox.widget(), input);
    }

    fn render_err(f: &mut tui::Frame<Backend>, msg: String) {
        let par = tui::widgets::Paragraph::new(msg);
        let block = Block::default()
            .title("UI Error");
        let par = par.block(block);
        f.render_widget(par, f.size());
    }
}


fn main() -> Result<(), io::Error> {
    tokio::runtime::Runtime::new().unwrap().block_on(run())
}

/// Primary run-loop
async fn run() -> Result<(), io::Error> {
    // first: move to alternate terminal mode
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(
        io::stdout(),
        crossterm::terminal::EnterAlternateScreen,
    )?;

    let mut terminal = tui::Terminal::new(CrosstermBackend::new(io::stdout()))?;
    let mut ui = ChatUI::new()?;

    loop {
        terminal.draw(|f| ChatUI::render(&ui.inputbox, f))?;

        match crossterm::event::read()?.into() {
            Input { key: Key::Esc, .. } => break,
            input => {
                ui.set_inputbox(input);
            }
        }
    }

    //tokio::time::sleep(Duration::from_secs(5)).await;

    // move back to standard terminal
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(
        io::stdout(),
        crossterm::terminal::LeaveAlternateScreen
    )?;

    Ok(())
}
