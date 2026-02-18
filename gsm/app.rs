use crate::{events, git, ui};
use anyhow::Result;
use ratatui::{backend::Backend, Terminal};

#[derive(Debug, PartialEq, Clone)]
pub enum Mode {
    Normal,
    Diff,
    Files,
    Confirm(ConfirmAction),
    NewStash,
    Message(String), // show result message
}

#[derive(Debug, PartialEq, Clone)]
pub enum ConfirmAction {
    Drop,
    Pop,
    Apply,
}

pub struct App {
    pub stashes: Vec<git::Stash>,
    pub selected: usize,
    pub mode: Mode,
    pub diff_content: Vec<String>,
    pub diff_scroll: usize,
    pub search_query: String,
    pub searching: bool,
    pub new_stash_input: String,
    pub new_stash_untracked: bool,
    pub status_msg: Option<String>,
    pub current_branch: String,
}

impl App {
    pub fn new() -> Result<Self> {
        let stashes = git::list_stashes()?;
        let current_branch = git::current_branch().unwrap_or_default();
        Ok(Self {
            stashes,
            selected: 0,
            mode: Mode::Normal,
            diff_content: Vec::new(),
            diff_scroll: 0,
            search_query: String::new(),
            searching: false,
            new_stash_input: String::new(),
            new_stash_untracked: false,
            status_msg: None,
            current_branch,
        })
    }

    pub fn reload(&mut self) -> Result<()> {
        self.stashes = git::list_stashes()?;
        self.current_branch = git::current_branch().unwrap_or_default();
        if self.selected >= self.stashes.len() && !self.stashes.is_empty() {
            self.selected = self.stashes.len() - 1;
        }
        Ok(())
    }

    pub fn filtered_stashes(&self) -> Vec<&git::Stash> {
        if self.search_query.is_empty() {
            self.stashes.iter().collect()
        } else {
            let q = self.search_query.to_lowercase();
            self.stashes
                .iter()
                .filter(|s| {
                    s.short_msg.to_lowercase().contains(&q)
                        || s.branch.to_lowercase().contains(&q)
                })
                .collect()
        }
    }

    pub fn selected_stash(&self) -> Option<&git::Stash> {
        let filtered = self.filtered_stashes();
        filtered.get(self.selected).copied()
    }

    pub fn load_diff(&mut self) -> Result<()> {
        if let Some(stash) = self.selected_stash() {
            let raw = git::stash_diff(&stash.name)?;
            self.diff_content = raw.lines().map(|l| l.to_string()).collect();
            self.diff_scroll = 0;
        }
        Ok(())
    }

    pub fn load_files(&mut self) -> Result<()> {
        if let Some(stash) = self.selected_stash() {
            let raw = git::stash_files(&stash.name)?;
            self.diff_content = raw.lines().map(|l| l.to_string()).collect();
            self.diff_scroll = 0;
        }
        Ok(())
    }

    pub fn move_up(&mut self) {
        if self.selected > 0 {
            self.selected -= 1;
        }
    }

    pub fn move_down(&mut self) {
        let len = self.filtered_stashes().len();
        if len > 0 && self.selected < len - 1 {
            self.selected += 1;
        }
    }

    pub fn scroll_diff_up(&mut self) {
        if self.diff_scroll > 0 {
            self.diff_scroll -= 1;
        }
    }

    pub fn scroll_diff_down(&mut self) {
        if self.diff_scroll + 1 < self.diff_content.len() {
            self.diff_scroll += 1;
        }
    }
}

pub fn run<B: Backend>(terminal: &mut Terminal<B>) -> Result<()> {
    let mut app = App::new()?;

    loop {
        terminal.draw(|f| ui::render(f, &app))?;

        if events::handle_events(&mut app)? {
            break;
        }
    }

    Ok(())
}