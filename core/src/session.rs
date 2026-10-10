//! One typing session: the Latin buffer, ranked candidates, selection and paging.
//!
//! The popup shows a page of up to `PAGE_SIZE` Sinhala candidates followed by the
//! raw Latin text as the last numbered row, as in the Google Input Tools UI.

use crate::{suggest, Dictionary};

pub const PAGE_SIZE: usize = 5;
const MAX_CANDIDATES: usize = 30;

/// What the shell needs to draw the popup.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PopupView {
    /// Candidates on this page, then the raw Latin text as the final row.
    /// Empty when not composing.
    pub rows: Vec<String>,
    /// Highlighted row index within `rows`.
    pub selected: usize,
    pub page: usize,
    pub page_count: usize,
}

pub struct Session<'a> {
    dict: &'a Dictionary<'a>,
    buffer: String,
    candidates: Vec<String>,
    /// Absolute index into `candidates`.
    selected: usize,
}

impl<'a> Session<'a> {
    pub fn new(dict: &'a Dictionary<'a>) -> Self {
        Session {
            dict,
            buffer: String::new(),
            candidates: Vec::new(),
            selected: 0,
        }
    }

    pub fn is_composing(&self) -> bool {
        !self.buffer.is_empty()
    }

    pub fn buffer(&self) -> &str {
        &self.buffer
    }

    /// Appends a typed character; resets the selection to the first candidate.
    pub fn push_char(&mut self, c: char) {
        self.buffer.push(c);
        self.refresh();
    }

    /// Removes the last character. Returns false if there was nothing to
    /// remove, meaning the shell should pass Backspace through to the app.
    pub fn backspace(&mut self) -> bool {
        if self.buffer.pop().is_none() {
            return false;
        }
        self.refresh();
        true
    }

    /// Moves the highlight by `delta` candidates, clamped; may cross pages.
    pub fn move_selection(&mut self, delta: i32) {
        if self.candidates.is_empty() {
            return;
        }
        let max = self.candidates.len() as i64 - 1;
        self.selected = (self.selected as i64 + delta as i64).clamp(0, max) as usize;
    }

    /// Turns pages by `delta`, clamped; selects the first row of the new page.
    pub fn turn_page(&mut self, delta: i32) {
        if self.candidates.is_empty() {
            return;
        }
        let page = (self.selected / PAGE_SIZE) as i64 + delta as i64;
        let page = page.clamp(0, self.page_count() as i64 - 1) as usize;
        self.selected = page * PAGE_SIZE;
    }

    pub fn view(&self) -> PopupView {
        if !self.is_composing() {
            return PopupView {
                rows: Vec::new(),
                selected: 0,
                page: 0,
                page_count: 1,
            };
        }
        let page = self.selected / PAGE_SIZE;
        let start = page * PAGE_SIZE;
        let end = (start + PAGE_SIZE).min(self.candidates.len());
        let mut rows: Vec<String> = self.candidates[start..end].to_vec();
        let selected = if self.candidates.is_empty() {
            0
        } else {
            self.selected - start
        };
        rows.push(self.buffer.clone());
        PopupView {
            rows,
            selected,
            page,
            page_count: self.page_count(),
        }
    }

    /// Commits the highlighted candidate and ends composition.
    pub fn commit(&mut self) -> Option<String> {
        if !self.is_composing() {
            return None;
        }
        let text = self
            .candidates
            .get(self.selected)
            .cloned()
            .unwrap_or_else(|| self.buffer.clone());
        self.reset();
        Some(text)
    }

    /// Commits row `n` (1-based) of the current page; the last row is the raw
    /// Latin text. Out-of-range numbers do nothing and keep composing.
    pub fn select_number(&mut self, n: usize) -> Option<String> {
        let rows = self.view().rows;
        if n == 0 || n > rows.len() {
            return None;
        }
        let text = rows[n - 1].clone();
        self.reset();
        Some(text)
    }

    /// Abandons composition without committing.
    pub fn cancel(&mut self) {
        self.reset();
    }

    fn page_count(&self) -> usize {
        self.candidates.len().div_ceil(PAGE_SIZE).max(1)
    }

    fn refresh(&mut self) {
        self.candidates = if self.buffer.is_empty() {
            Vec::new()
        } else {
            suggest(&self.buffer, self.dict, MAX_CANDIDATES)
        };
        self.selected = 0;
    }

    fn reset(&mut self) {
        self.buffer.clear();
        self.candidates.clear();
        self.selected = 0;
    }
}
