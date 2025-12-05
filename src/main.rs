use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use random_word::Lang;
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Constraint, Direction, Flex, Layout, Rect},
    text::Text,
    widgets::{Block, Clear, Paragraph, Widget, Wrap},
};
use serde::{Deserialize, Serialize};
use serde_json;
use std::fs;
use std::io;
use std::path::PathBuf;
extern crate dirs;

const DEFAULT_STAT: &'static str = r#"{
    "won": 0,
    "lost": 0,
    "average": 0
}"#;

#[derive(Serialize, Deserialize, Debug)]
struct Stats {
    won: u32,
    lost: u32,
    average: f64,
}

fn main() -> io::Result<()> {
    let mut terminal = ratatui::init();

    let word1 = "hangman".to_string();
    let blanks = vec!['_'; word1.chars().count()];

    let confloc: PathBuf = dirs::config_dir().expect("Can't find config location");
    check_config(&confloc);

    let mut app = App {
        exit: false,
        word: word1,
        guess: blanks,
        lives: 10,
        guessed: Vec::new(),
        win: false,
        popup: false,
        stats: false,
        statstruct: get_stats(&confloc),
        //config_path: confloc,
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
    stats: bool,
    statstruct: Stats,
    //config_path: PathBuf,
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

            if let Ok(true) = event::poll(std::time::Duration::from_millis(50)) {
                if let Ok(Event::Key(key_event)) = event::read() {
                    if key_event.kind == KeyEventKind::Press {
                        self.handle_key_event(key_event);
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('Q') => self.exit(),
            KeyCode::Char('N') => self.new_game(),
            KeyCode::Char('S') => {
                if self.stats {
                    self.stats = false
                } else {
                    self.stats = true
                }
            }
            KeyCode::Char(c) => {
                if c.is_ascii_alphabetic() {
                    let letter = c.to_ascii_lowercase();
                    if self.check(letter) {
                        self.statstruct.won += 1;
                        if self.statstruct.lost == 0 {
                            self.statstruct.average = 1.0;
                        } else {
                            self.statstruct.average = self.statstruct.won as f64
                                / (self.statstruct.lost + self.statstruct.won) as f64
                                * 100.0;
                        };
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

        let stats_text = format!(
            "\nGames won: {}\nGames lost: {}\n Win rate: {}%",
            self.statstruct.won,
            self.statstruct.lost,
            self.statstruct.average.ceil()
        );

        if self.stats {
            let popup = Paragraph::new(stats_text)
                .block(Block::bordered())
                .centered();
            let area = center(
                frame.area(),
                Constraint::Percentage(60),
                Constraint::Percentage(40),
            );
            frame.render_widget(Clear, area);
            frame.render_widget(popup, area);
        }
    }

    fn exit(&mut self) {
        save_stats(
            &self.statstruct,
            &dirs::config_dir().expect("Can't find config location"),
        );
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
            if self.lives == 1 {
                self.lives -= 1;
                self.statstruct.lost += 1;
                if self.statstruct.lost == 0 {
                    self.statstruct.average = 1.0;
                } else {
                    self.statstruct.average = self.statstruct.won as f64
                        / (self.statstruct.lost + self.statstruct.won) as f64
                        * 100.0;
                };
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
            .constraints(vec![
                Constraint::Percentage(30),
                Constraint::Percentage(65),
                Constraint::Percentage(5),
            ])
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
        Paragraph::new(Text::from(
            "<shift-n>: new game | <shift-s>: view stats | <shift-q>: quit",
        ))
        .wrap(Wrap { trim: true })
        .render(layout[2], buf);
    }
}

fn center(area: Rect, horizontal: Constraint, vertical: Constraint) -> Rect {
    let [area] = Layout::horizontal([horizontal])
        .flex(Flex::Center)
        .areas(area);
    let [area] = Layout::vertical([vertical]).flex(Flex::Center).areas(area);
    area
}

fn check_config(conf_locs: &PathBuf) {
    let mut config = conf_locs.clone();
    config.push("hangman");
    config.push("stats.json");
    match fs::exists(config) {
        Ok(true) => return,
        Ok(false) => generate_config(conf_locs),
        Err(_) => println!("Can't check for exisiting stats"),
    };
}

fn get_stats(conf_locs: &PathBuf) -> Stats {
    let mut path = conf_locs.clone();
    path.push("hangman");
    path.push("stats.json");
    let data = fs::read_to_string(&path).unwrap();
    serde_json::from_str::<Stats>(&data).unwrap()
}

fn save_stats(stats: &Stats, conf_locs: &PathBuf) {
    let mut path = conf_locs.clone();
    path.push("hangman");
    path.push("stats.json");

    let json = serde_json::to_string_pretty(&stats).unwrap();
    let _ = fs::write(path, json);
}

fn generate_config(conf_locs: &PathBuf) {
    let mut config: PathBuf = conf_locs.clone();
    config.push("hangman");
    match fs::create_dir(&config) {
        Ok(()) => println!("Directory created"),
        Err(e) => println!("Failed to create directory: {:?}", e.kind()),
    };
    config.push("stats.json");
    match fs::write(config, DEFAULT_STAT) {
        Ok(()) => println!("Stats loaded"),
        Err(_) => println!("Can't write file"),
    };
}
