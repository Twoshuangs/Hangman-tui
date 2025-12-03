use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use random_word::Lang;
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    text::Text,
    widgets::{Block, Borders, Paragraph, Widget},
};
use std::io;

#[tokio::main]
async fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();

    let word1 = "hangman".to_string();
    let blanks = vec!['_'; word1.chars().count()];

    let mut app = App {
        exit: false,
        word: word1,
        guess: blanks,
        lives: 10,
    };

    app.run(&mut terminal)?;

    ratatui::restore();
    Ok(())
}

struct App {
    exit: bool,
    word: String,
    guess: Vec<char>,
    lives: i32,
}

impl App {
    pub fn new_game(&mut self) {
        self.word = random_word::get(Lang::En).to_string();
        self.guess = vec!['_'; self.word.chars().count()];
        self.lives = 10;
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;

            // Non-blocking input
            if let Ok(true) = event::poll(std::time::Duration::from_millis(50)) {
                if let Ok(Event::Key(key_event)) = event::read() {
                    if key_event.kind == KeyEventKind::Press {
                        self.handle_key_event(key_event);
                    }
                }
            }

            //if self.lives > 0 && !self.guess.contains(&'_') {
            //    self.win();
            //}
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('Q') => self.exit(),
            KeyCode::Char('N') => self.new_game(),
            KeyCode::Char(c) => {
                let letter = c.to_ascii_lowercase();
                self.check(letter);
            }
            _ => {}
        }
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn check(&mut self, letter: char) -> bool {
        if self.word.contains(letter) {
            for (index, ch) in self.word.chars().enumerate() {
                if ch == letter {
                    self.guess[index] = letter;
                }
            }
            !self.guess.contains(&'_')
        } else {
            self.lives -= 1;
            false
        }
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![
                Constraint::Percentage(30),
                Constraint::Percentage(10),
                Constraint::Percentage(30),
                Constraint::Percentage(30),
            ])
            .split(area);

        Paragraph::new(Text::from(self.lives.to_string()))
            .block(Block::bordered().title_top("Guesses remaining"))
            .centered()
            .render(layout[1], buf);

        Paragraph::new(Text::from(self.guess.iter().collect::<String>()))
            .block(Block::bordered().title_top("Hangman"))
            .centered()
            .render(layout[2], buf);
    }
}
