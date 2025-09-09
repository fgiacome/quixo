use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use quixo_core::{
    game::{Board, Move, Player, Shift, winner},
    mcts::{GameState, mcts},
};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{
        Constraint::{self, Length},
        Flex, Layout, Rect,
    },
    style::Stylize,
    text::{Line, Text},
    widgets::{Cell, Gauge, Paragraph, Row, Table},
};
use std::{
    sync::mpsc,
    thread::{self, JoinHandle},
    time::Duration,
};

fn main() -> color_eyre::Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = App::new().run(terminal);
    ratatui::restore();
    result
}

/// The main application which holds the state and logic of the application.
#[derive(Debug)]
pub struct App {
    turn: Player,
    board: Board,
    running: bool,
    selected_position: (usize, usize),
    winner: Option<Player>,
    thread_handle: Option<JoinHandle<Option<Move>>>,
    progress_channel: Option<mpsc::Receiver<(u32, Option<Move>)>>,
    progress_value: Option<u32>,
}

impl App {
    /// Construct a new instance of [`App`].
    pub fn new() -> Self {
        App {
            turn: Player::X,
            board: [[None; 5]; 5],
            running: false,
            selected_position: (0, 0),
            winner: None,
            thread_handle: None,
            progress_channel: None,
            progress_value: None,
        }
    }

    pub fn reset(&mut self) {
        self.board = [[None; 5]; 5];
        self.winner = None;
        self.turn = Player::X;
    }

    /// Run the application's main loop.
    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        self.running = true;
        while self.running {
            terminal.draw(|frame| self.render(frame))?;
            self.handle_crossterm_events()?;
        }
        Ok(())
    }

    /// Renders the user interface.
    ///
    /// This is where you add new widgets. See the following resources for more information:
    ///
    /// - <https://docs.rs/ratatui/latest/ratatui/widgets/index.html>
    /// - <https://github.com/ratatui/ratatui/tree/main/ratatui-widgets/examples>
    fn render(&mut self, frame: &mut Frame) {
        let layout = Layout::vertical([Length(5), Length(1), Length(1), Length(3)]);
        let [table_area, status_area, progress_area, help_area] = layout.areas(frame.area());
        let [table_area] = Layout::horizontal([Length(19)])
            .flex(Flex::Center)
            .areas(table_area);
        let [progress_area] = Layout::horizontal([Constraint::Max(20)])
            .flex(Flex::Center)
            .areas(progress_area);
        let status_line = Line::from(format!(
            "Turn: {}, Winner: {}",
            self.turn,
            self.winner.map_or(String::from("-"), |p| p.to_string()),
        ))
        .centered();
        let gauge = if let Some(p) = self.progress_value {
            Some(Gauge::default().percent((p / 10) as u16))
        } else {
            None
        };
        let help = Paragraph::new(vec![
            Line::from("left, right, top, bottom: move selection").centered(),
            Line::from("shift + left, right, top, bottom: move selected piece").centered(),
            Line::from("c: call mcts, r: reset, q: quit").centered(),
        ]);
        self.render_table(frame, table_area);
        frame.render_widget(status_line, status_area);
        frame.render_widget(help, help_area);
        if let Some(g) = gauge {
            frame.render_widget(g, progress_area);
        }
    }

    pub fn render_table(&mut self, frame: &mut Frame, area: Rect) {
        let mut rows = Vec::new();
        for (i, r) in self.board.iter().enumerate() {
            let mut row = Vec::new();
            for (j, p) in r.iter().enumerate() {
                let text = match p {
                    Some(Player::X) => Text::from("X").bold().blue(),
                    Some(Player::O) => Text::from("O").bold().red(),
                    _ => Text::from("-"),
                }
                .centered();
                if self.selected_position == (i, j) {
                    row.push(Cell::from(text).on_white());
                } else {
                    row.push(Cell::from(text));
                }
            }
            let row = Row::new(row);
            rows.push(row);
        }
        let rows: Vec<_> = rows;
        frame.render_widget(Table::new(rows, [3; 5]), area);
    }

    /// Reads the crossterm events and updates the state of [`App`].
    ///
    /// If your application needs to perform work in between handling events, you can use the
    /// [`event::poll`] function to check if there are any events available with a timeout.
    fn handle_crossterm_events(&mut self) -> Result<()> {
        if event::poll(Duration::from_millis(5))? {
            match event::read()? {
                // it's important to check KeyEventKind::Press to avoid handling key release events
                Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
                Event::Mouse(_) => {}
                Event::Resize(_, _) => {}
                _ => {}
            }
        }

        if let Some(_) = self.thread_handle
            && let Some(rx) = &self.progress_channel
            && let Ok(p) = rx.try_recv()
        {
            self.progress_value = Some(p.0);
        }

        if let Some(h) = &self.thread_handle
            && h.is_finished()
            && let Some(h) = self.thread_handle.take()
            && let Some(m) = h.join().unwrap()
        {
            self.progress_channel = None;
            self.progress_value = None;
            self.board = m.apply(self.turn, &self.board).unwrap();
            self.turn = self.turn.next();
            self.winner = winner(&self.board);
        }

        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            // Add other key handlers here.
            (KeyModifiers::NONE, KeyCode::Up) => {
                self.selected_position.0 = match self.selected_position.0 {
                    0 => 0,
                    _ => self.selected_position.0 - 1,
                }
            }
            (KeyModifiers::NONE, KeyCode::Down) => {
                self.selected_position.0 = match self.selected_position.0 {
                    4 => 4,
                    _ => self.selected_position.0 + 1,
                }
            }
            (KeyModifiers::NONE, KeyCode::Left) => {
                self.selected_position.1 = match self.selected_position.1 {
                    0 => 0,
                    _ => self.selected_position.1 - 1,
                }
            }
            (KeyModifiers::NONE, KeyCode::Right) => {
                self.selected_position.1 = match self.selected_position.1 {
                    4 => 4,
                    _ => self.selected_position.1 + 1,
                }
            }
            (KeyModifiers::SHIFT, KeyCode::Left) => {
                let m = Move {
                    x: self.selected_position.1 as u8,
                    y: self.selected_position.0 as u8,
                    shift: Shift::LEFT,
                };
                if let None = self.thread_handle
                    && let Ok(b) = m.apply(self.turn, &self.board)
                {
                    self.board = b;
                    self.turn = self.turn.next();
                    self.winner = winner(&b);
                }
            }
            (KeyModifiers::SHIFT, KeyCode::Down) => {
                let m = Move {
                    x: self.selected_position.1 as u8,
                    y: self.selected_position.0 as u8,
                    shift: Shift::BOTTOM,
                };
                if let None = self.thread_handle
                    && let Ok(b) = m.apply(self.turn, &self.board)
                {
                    self.board = b;
                    self.turn = self.turn.next();
                    self.winner = winner(&b);
                }
            }
            (KeyModifiers::SHIFT, KeyCode::Up) => {
                let m = Move {
                    x: self.selected_position.1 as u8,
                    y: self.selected_position.0 as u8,
                    shift: Shift::TOP,
                };
                if let None = self.thread_handle
                    && let Ok(b) = m.apply(self.turn, &self.board)
                {
                    self.board = b;
                    self.turn = self.turn.next();
                    self.winner = winner(&b);
                }
            }
            (KeyModifiers::SHIFT, KeyCode::Right) => {
                let m = Move {
                    x: self.selected_position.1 as u8,
                    y: self.selected_position.0 as u8,
                    shift: Shift::RIGHT,
                };
                if let None = self.thread_handle
                    && let Ok(b) = m.apply(self.turn, &self.board)
                {
                    self.board = b;
                    self.turn = self.turn.next();
                    self.winner = winner(&b);
                }
            }
            (_, KeyCode::Char('R') | KeyCode::Char('r')) => {
                self.reset();
            }
            (_, KeyCode::Char('C') | KeyCode::Char('c')) => {
                let gm = GameState {
                    board: self.board,
                    player: self.turn,
                };
                let (tx, rx) = mpsc::channel();
                self.progress_channel = Some(rx);
                self.thread_handle = Some(thread::spawn(move || mcts(gm, 1000, 1000, Some(tx))));
            }
            _ => {}
        }
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }
}
