use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use random_word::Lang;
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Direction, Flex, Layout, Rect},
    text::Text,
    widgets::{Block, Clear, Paragraph, Widget},
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
        guessed: Vec::new(),
        win: false,
        popup: false,
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
    guessed: Vec<char>,
    win: bool,
    popup: bool,
}

impl App {
    pub fn new_game(&mut self) {
        self.word = random_word::get(Lang::En).to_string();
        self.guess = vec!['_'; self.word.chars().count()];
        self.lives = 10;
        self.guessed.clear();
        self.win = false;
        self.popup = false;
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
                if c.is_ascii_alphabetic() {
                    let letter = c.to_ascii_lowercase();
                    if self.check(letter) {
                        self.win = true;
                        self.popup = true;
                    }
                }
            }
            _ => {}
        }
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());

        let notif = if self.win {
            "Congratulations. \nYou have found the word.".to_owned()
        } else {
            "You died x_x. \nThe word was ".to_string() + self.word.clone().as_str()
        };
        let notiftext = Text::from(notif.clone());

        if self.popup {
            let popup = Paragraph::new(notif).block(Block::bordered()).centered();
            let area = center(
                frame.area(),
                Constraint::Length(notiftext.width() as u16 + 4),
                Constraint::Length(4),
            );
            frame.render_widget(Clear, area);
            frame.render_widget(popup, area);
        };
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
            self.guessed.push(letter);
            if self.lives == 0 {
                self.popup = true;
            } else {
                self.lives -= 1;
            }
            false
        }
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let gamearea = center(area, Constraint::Percentage(60), Constraint::Percentage(40));

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vec![Constraint::Percentage(30), Constraint::Percentage(70)])
            .split(gamearea);

        let inner_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(layout[0]);

        Paragraph::new(Text::from(self.lives.to_string()))
            .block(Block::bordered().title_top("Guesses \nremaining"))
            .centered()
            .render(inner_layout[0], buf);
        Paragraph::new(Text::from(self.guessed.iter().collect::<String>()))
            .block(Block::bordered().title_top("Alr guessed"))
            .centered()
            .render(inner_layout[1], buf);

        Paragraph::new(Text::from(self.guess.iter().collect::<String>()))
            .block(Block::bordered().title_top("Hangman"))
            .centered()
            .render(layout[1], buf);
    }
}

fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}
