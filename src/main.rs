use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use rand::prelude::*;
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::{Line, Text},
    widgets::{Block, Paragraph, Widget, Wrap},
};
use std::{
    fmt::Debug,
    fs,
    time::{Duration, SystemTime},
};

mod ascii_art;

const C_BG: Color = Color::Rgb(15, 11, 25);
const C_SUBTLE: Color = Color::Rgb(75, 60, 95);
const C_LIME: Color = Color::Rgb(163, 230, 53);
const C_PURP: Color = Color::Rgb(170, 130, 240);
const C_ERR: Color = Color::Rgb(244, 63, 94);

const TYPING_TIME: u32 = 30;
#[derive(Debug, Default, PartialEq, Eq)]
pub enum GameState {
    #[default]
    BeforeTyping,
    Typing,
    Finished,
}

#[derive(Debug)]
pub struct App {
    words: String,
    user_input_words: String,
    exit: bool,
    frame_count: u64,
    start_time: Option<SystemTime>,
    finished_time: Option<SystemTime>,
    time_left: u32,
    game_state: GameState,
}

impl Default for App {
    fn default() -> Self {
        Self {
            words: get_words(),
            user_input_words: String::new(),
            exit: false,
            frame_count: 0,
            start_time: None,
            finished_time: None,
            time_left: TYPING_TIME,
            game_state: GameState::BeforeTyping,
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    #[cfg(test)]
    pub fn with_words(words: String) -> Self {
        Self {
            words,
            user_input_words: String::new(),
            exit: false,
            frame_count: 0,
            start_time: None,
            finished_time: None,
            time_left: TYPING_TIME,
            game_state: GameState::BeforeTyping,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
            self.check_timer();
            self.frame_count += 1;
        }
        Ok(())
    }

    pub fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> Result<()> {
        if !event::poll(Duration::from_millis(30))? {
            return Ok(());
        }

        if let Event::Key(key) = event::read()?
            && key.kind == KeyEventKind::Press
        {
            self.handle_key_event(key);
        }

        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        if self.game_state == GameState::Finished {
            match key_event.code {
                KeyCode::Esc => self.exit(),
                KeyCode::Char('r') => {
                    if let Some(ft) = self.finished_time
                        && ft.elapsed().unwrap_or_default() >= Duration::from_millis(1000)
                    {
                        self.restart_game();
                    }
                }
                _ => {}
            }
            return;
        }
        match key_event.code {
            KeyCode::Esc => self.exit(),
            KeyCode::Backspace => {
                self.user_input_words.pop();
            }
            KeyCode::Char(c) => {
                self.user_input_words.push(c);
                if self.start_time.is_none() {
                    self.start_time = Some(SystemTime::now());
                    self.game_state = GameState::Typing;
                }
            }
            _ => {}
        }
    }

    fn check_timer(&mut self) {
        self.time_left = self.time_left();

        if self.time_left == 0 && self.game_state != GameState::Finished {
            self.game_state = GameState::Finished;
            self.finished_time = Some(SystemTime::now());
        }
    }

    fn restart_game(&mut self) {
        self.words = get_words();
        self.user_input_words = String::new();
        self.time_left = TYPING_TIME;
        self.start_time = None;
        self.finished_time = None;
        self.game_state = GameState::BeforeTyping;
    }

    fn exit(&mut self) {
        self.exit = true;
    }

    fn time_left(&self) -> u32 {
        let passed: u32 = match self.start_time {
            Some(st) => SystemTime::now()
                .duration_since(st)
                .expect("Time went backwards")
                .as_secs()
                .try_into()
                .unwrap(),
            None => 0,
        };

        TYPING_TIME.saturating_sub(passed)
    }

    fn generate_typing_spans(&self) -> Text<'static> {
        let target_chars: Vec<char> = self.words.chars().collect();
        let user_chars: Vec<char> = self.user_input_words.chars().collect();

        let mut spans = Vec::new();

        for i in 0..target_chars.len() {
            if i < user_chars.len() {
                if user_chars[i] == target_chars[i] {
                    spans.push(target_chars[i].to_string().fg(C_LIME));
                } else {
                    spans.push(target_chars[i].to_string().fg(C_ERR).underlined());
                }
            } else if i == user_chars.len() {
                spans.push(target_chars[i].to_string().fg(C_BG).bg(C_PURP));
            } else {
                spans.push(target_chars[i].to_string().fg(C_SUBTLE));
            }
        }

        Text::from(Line::from(spans))
    }

    fn count_words(&self) -> u32 {
        let mut count: u32 = 0;
        let target_words = self.words.split_whitespace();

        for (word, target_word) in self.user_input_words.split_whitespace().zip(target_words) {
            if word == target_word {
                count += 1;
            }
        }

        count
    }

    fn get_wpm(&self) -> u32 {
        let divy: u32 = 60 / TYPING_TIME;
        self.count_words() * divy
    }

    fn render_typing_screen(&self, area: Rect, buf: &mut Buffer) {
        let vertical_chunks = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(10),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .split(area);

        let horizontal_chunks = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(75),
            Constraint::Fill(1),
        ])
        .split(vertical_chunks[1]);

        let center_area = horizontal_chunks[1];

        let top_bar_chunks =
            Layout::vertical([Constraint::Length(2), Constraint::Min(1)]).split(center_area);

        let timer_text = Line::from(vec![
            self.time_left.to_string().fg(C_LIME).bold(),
            "s   |   ".fg(C_SUBTLE),
            format!("{}/{}", self.user_input_words.len(), self.words.len()).fg(C_PURP),
        ]);

        Paragraph::new(timer_text)
            .alignment(Alignment::Left)
            .render(top_bar_chunks[0], buf);

        let inner_text = self.generate_typing_spans();

        Paragraph::new(inner_text)
            .alignment(Alignment::Left)
            .wrap(Wrap { trim: false })
            .render(top_bar_chunks[1], buf);

        let footer_text = Line::from("esc to quit".fg(C_SUBTLE));
        Paragraph::new(footer_text)
            .alignment(Alignment::Center)
            .render(vertical_chunks[3], buf);
    }

    fn render_finished_screen(&self, area: Rect, buf: &mut Buffer) {
        let wpm = self.get_wpm();
        let words_typed = self.count_words();

        let mut content = vec![
            Line::from(""),
            Line::from("     ___________     ".fg(C_PURP)),
            Line::from("    '._==_==_=_.'    ".fg(C_PURP)),
            Line::from("    .-\\:      /-.    ".fg(C_PURP)),
            Line::from("   | (|:.     |) |   ".fg(C_PURP)),
            Line::from("    '-|:.     |-'    ".fg(C_PURP)),
            Line::from("      \\::.    /      ".fg(C_PURP)),
            Line::from("       '::. .'       ".fg(C_PURP)),
            Line::from("         ) (         ".fg(C_PURP)),
            Line::from("       _.' '._       ".fg(C_PURP)),
            Line::from("      `\"\"\"\"\"\"\"`      ".fg(C_PURP)),
            Line::from(""),
        ];

        let ascii_wpm = ascii_art::get_wpm_ascii(wpm);
        for line in ascii_wpm {
            content.push(Line::from(line.fg(C_LIME).bold()));
        }

        content.push(Line::from(""));

        content.push(Line::from(vec![
            " Words: ".fg(C_SUBTLE),
            format!("{:>7}", words_typed).fg(C_PURP).bold(),
        ]));
        content.push(Line::from(vec![
            " Time:  ".fg(C_SUBTLE),
            format!("{:>6}s", TYPING_TIME).fg(C_PURP).bold(),
        ]));

        // Ratatui will aggressively delete trailing whitespace from standard Wrap configurations.
        // We set trim: false here so it respects the invisible padding spaces that shape our ASCII art.
        let paragraph = Paragraph::new(content)
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false });

        let vertical_chunks = Layout::vertical([
            Constraint::Fill(1),
            Constraint::Length(32),
            Constraint::Fill(1),
            Constraint::Length(1),
        ])
        .split(area);

        let horizontal_chunks = Layout::horizontal([
            Constraint::Fill(1),
            Constraint::Length(70),
            Constraint::Fill(1),
        ])
        .split(vertical_chunks[1]);

        paragraph.render(horizontal_chunks[1], buf);

        let footer_text = Line::from(vec![
            "r to restart   ".fg(C_SUBTLE),
            "esc to quit".fg(C_SUBTLE),
        ]);
        Paragraph::new(footer_text)
            .alignment(Alignment::Center)
            .render(vertical_chunks[3], buf);
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Block::default().bg(C_BG).render(area, buf);

        match self.game_state {
            GameState::BeforeTyping | GameState::Typing => {
                self.render_typing_screen(area, buf);
            }
            GameState::Finished => {
                self.render_finished_screen(area, buf);
            }
        }
    }
}

fn get_words() -> String {
    let file_content = fs::read_to_string("./english_short.txt");

    let mut dictionary: Vec<String> = match file_content {
        Ok(content) => content.lines().map(String::from).collect(),
        Err(_) => vec![
            "test".into(),
            "words".into(),
            "typing".into(),
            "game".into(),
            "fallback".into(),
        ],
    };

    let mut words: String = String::new();

    // Saturating bounds ensure we don't crash if the dictionary file is tiny or missing
    for _ in 1..=50 {
        if dictionary.is_empty() {
            break;
        }
        let number = rand::rng().random_range(0..dictionary.len());
        words.push_str((dictionary.remove(number) + " ").as_str());
    }

    words
}

fn main() -> Result<()> {
    color_eyre::install()?;
    ratatui::run(|terminal| App::new().run(terminal))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_word_counting() {
        let mut app = App::with_words("hello world test code".to_string());

        app.user_input_words = "hello wor test".to_string();
        // "hello" matches, "wor" != "world", "test" matches. Total = 2.
        assert_eq!(app.count_words(), 2);
    }

    #[test]
    fn test_wpm_calculation() {
        let mut app = App::with_words("hello world test code".to_string());

        app.user_input_words = "hello world test".to_string();
        // 3 words matched. Formula: count * (60 / 5) => 3 * 12 = 36 WPM
        assert_eq!(app.get_wpm(), 36);
    }

    #[test]
    fn test_input_and_state_transition() {
        let mut app = App::with_words("hello".to_string());

        assert_eq!(app.game_state, GameState::BeforeTyping);
        assert!(app.start_time.is_none());

        // Hitting a character transitions state and starts timer
        app.handle_key_event(KeyEvent::from(KeyCode::Char('h')));
        assert_eq!(app.user_input_words, "h");
        assert_eq!(app.game_state, GameState::Typing);
        assert!(app.start_time.is_some());

        // Backspace removes characters
        app.handle_key_event(KeyEvent::from(KeyCode::Backspace));
        assert_eq!(app.user_input_words, "");
    }

    #[test]
    fn test_exit_flag() {
        let mut app = App::default();
        assert!(!app.exit);

        app.handle_key_event(KeyEvent::from(KeyCode::Esc));
        assert!(app.exit);
    }

    #[test]
    fn test_restart_resets_state() {
        let mut app = App::default();
        app.game_state = GameState::Finished;
        app.time_left = 0;
        app.user_input_words = "mistakes were made".to_string();
        app.start_time = Some(SystemTime::now());
        app.finished_time = Some(SystemTime::now());

        app.restart_game();

        assert_eq!(app.game_state, GameState::BeforeTyping);
        assert_eq!(app.time_left, TYPING_TIME);
        assert_eq!(app.user_input_words, "");
        assert!(app.start_time.is_none());
        assert!(app.finished_time.is_none());
    }
}
