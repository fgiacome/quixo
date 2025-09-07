use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use quixo_core::{
    game::{Board, Move, Player, Shift, winner},
    mcts::{GameState, mcts},
};
use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint::Length, Flex, Layout, Rect},
    style::Stylize,
    text::{Line, Text},
    widgets::{Cell, Paragraph, Row, Table},
};
use std::time::Duration;

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
        let layout = Layout::vertical([Length(5), Length(1), Length(3)]);
        let [table_area, status_area, help_area] = layout.areas(frame.area());
        let [table_area] = Layout::horizontal([Length(19)])
            .flex(Flex::Center)
            .areas(table_area);
        let status_line = Line::from(format!(
            "Turn: {}, Winner: {}",
            self.turn,
            self.winner.map_or(String::from("-"), |p| p.to_string()),
         ))
        .centered();
        let help = Paragraph::new(vec![
            Line::from("left, right, top, bottom: move selection").centered(),
            Line::from("h, j, k, l: move selected piece").centered(),
            Line::from("c: call mcts, r: reset, q: quit").centered(),
        ]);
        self.render_table(frame, table_area);
        frame.render_widget(status_line, status_area);
        frame.render_widget(help, help_area);
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

        Ok(())
    }

    /// Handles the key events and updates the state of [`App`].
    fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            // Add other key handlers here.
            (_, KeyCode::Up) => {
                self.selected_position.0 = match self.selected_position.0 {
                    0 => 0,
                    _ => self.selected_position.0 - 1,
                }
            }
            (_, KeyCode::Down) => {
                self.selected_position.0 = match self.selected_position.0 {
                    4 => 4,
                    _ => self.selected_position.0 + 1,
                }
            }
            (_, KeyCode::Left) => {
                self.selected_position.1 = match self.selected_position.1 {
                    0 => 0,
                    _ => self.selected_position.1 - 1,
                }
            }
            (_, KeyCode::Right) => {
                self.selected_position.1 = match self.selected_position.1 {
                    4 => 4,
                    _ => self.selected_position.1 + 1,
                }
            }
            (_, KeyCode::Char('H') | KeyCode::Char('h')) => {
                let m = Move {
                    x: self.selected_position.1 as u8,
                    y: self.selected_position.0 as u8,
                    shift: Shift::LEFT,
                };
                if let Ok(b) = m.apply(self.turn, &self.board) {
                    self.board = b;
                    self.turn = self.turn.next();
                    self.winner = winner(&b);
                }
            }
            (_, KeyCode::Char('J') | KeyCode::Char('j')) => {
                let m = Move {
                    x: self.selected_position.1 as u8,
                    y: self.selected_position.0 as u8,
                    shift: Shift::BOTTOM,
                };
                if let Ok(b) = m.apply(self.turn, &self.board) {
                    self.board = b;
                    self.turn = self.turn.next();
                    self.winner = winner(&b);
                }
            }
            (_, KeyCode::Char('K') | KeyCode::Char('k')) => {
                let m = Move {
                    x: self.selected_position.1 as u8,
                    y: self.selected_position.0 as u8,
                    shift: Shift::TOP,
                };
                if let Ok(b) = m.apply(self.turn, &self.board) {
                    self.board = b;
                    self.turn = self.turn.next();
                    self.winner = winner(&b);
                }
            }
            (_, KeyCode::Char('L') | KeyCode::Char('l')) => {
                let m = Move {
                    x: self.selected_position.1 as u8,
                    y: self.selected_position.0 as u8,
                    shift: Shift::RIGHT,
                };
                if let Ok(b) = m.apply(self.turn, &self.board) {
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
                let m = mcts(gm, 1000, 1000);
                if let Some(m) = m {
                    self.board = m.apply(self.turn, &self.board).unwrap();
                    self.turn = self.turn.next();
                    self.winner = winner(&self.board);
                }
            }
            _ => {}
        }
    }

    /// Set running to false to quit the application.
    fn quit(&mut self) {
        self.running = false;
    }
}
