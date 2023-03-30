use crate::classify::{classify, CharType};
use crate::error::Result;
use crate::prelude::*;
use crate::row::Row;
use crate::types::{Coord, Pos};
use crate::undo::ChangeTracker;
use crate::undo::{Change, ChangeStack};
use crate::utils::read_lines;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Doc {
    filename: Option<String>,
    tracked_rows: Vec<Row>,
    dirty: bool,
    change_stack: ChangeStack,
}

impl Doc {
    pub fn empty() -> Self {
        Self {
            filename: None,
            tracked_rows: vec![Row::default()],
            dirty: false,
            change_stack: Default::default(),
        }
    }
    pub fn new_change_tracker(&mut self) -> ChangeTracker {
        ChangeTracker::begin_changes(self)
    }
    pub fn is_empty(&self) -> bool {
        self.tracked_rows.is_empty()
    }
    pub fn is_dirty(&self) -> bool {
        self.dirty
    }
    pub fn get_filename(&self) -> Option<&str> {
        match self.filename {
            Some(ref filename) => Some(filename.as_str()),
            None => None,
        }
    }
    pub fn iter_lines(&self, y: Coord) -> IterLines {
        IterLines {
            row_iter: self.tracked_rows[y..].iter(),
        }
    }
    pub fn iter_from(&self, pos: Pos) -> IterChars {
        IterChars {
            rows: &self.tracked_rows,
            row: self.tracked_rows.get(pos.y),
            pos,
            y_offset: 0,
        }
    }
    pub fn iter_line(&self, y: Coord) -> IterChars {
        if self.tracked_rows.len() > y {
            IterChars {
                rows: &self.tracked_rows[y..=y],
                row: self.tracked_rows.get(y),
                pos: Pos { x: 0, y },
                y_offset: 0,
            }
        } else {
            IterChars {
                rows: &[],
                row: None,
                pos: Pos::zero(),
                y_offset: 0,
            }
        }
    }
    pub fn push_change(&mut self, change: Change) {
        let mut temp = ChangeStack::default();
        std::mem::swap(&mut self.change_stack, &mut temp);
        temp.push(self, change);
        std::mem::swap(&mut self.change_stack, &mut temp);
    }
    pub fn swap_rows(&mut self, range: &mut Range<Coord>, rows: &mut Vec<Row>) {
        assert!((0..=self.tracked_rows.len()).contains(&range.start));
        assert!((range.start..=self.tracked_rows.len()).contains(&range.end));
        let mut result_rows = self.tracked_rows[*range].iter().cloned().collect();
        let mut result_range = range.start..range.start + rows.len();
        self.tracked_rows.splice(*range, rows.iter().cloned());
        std::mem::swap(&mut result_rows, rows);
        std::mem::swap(&mut result_range, range);
    }
    pub fn render_line(&self, pos: Pos) -> std::slice::Iter<'_, char> {
        if pos.y < self.tracked_rows.len() {
            self.tracked_rows
                .get(pos.y)
                .unwrap()
                .render_chars_from(pos.x)
        } else {
            EMPTY.iter()
        }
    }
    pub fn line_count(&self) -> usize {
        self.tracked_rows.len()
    }
    pub fn get_row(&self, y: Coord) -> Option<&Row> {
        self.tracked_rows.get(y)
    }
    pub fn open(filename: String) -> Result<Self> {
        let mut doc = Doc::empty();
        doc.change_stack.clear();
        doc.tracked_rows.truncate(0);
        let lines = read_lines(&filename)?;
        for line in lines {
            doc.tracked_rows.push(Row::from_line(&line?));
        }
        doc.filename = Some(filename);
        doc.dirty = false;
        Ok(doc)
    }
    pub fn split_newline(&self, cursor: Pos) -> (crate::undo::Op, Pos) {
        let new_row = if let Some(row) = self.rows.get_mut(cursor.y) {
            let x = cursor.x.clamp(0, row.len());
            let ret = Row::from_chars(row.get_slice(x..row.len()));
            row.truncate(x);
            ret
                Op::RowsSwap(cursor.y..cursor.y+1,
                    vec![*self.get_row(cursor.y)
        } else {
            Row::from_line("")
        };
        self.rows.insert(cursor.y + 1, new_row);
        self.dirty = true;
    }
    pub fn insert_newline(&mut self, y: Coord) {
        let y = std::cmp::min(y, self.rows.len());
        self.rows.splice(y..y, [Row::from_line("")]);
        self.dirty = true;
    }
    pub fn insert_char(&mut self, cursor: Pos, ch: char) {
        if let Some(row) = self.rows.get_mut(cursor.y) {
            row.insert_char(cursor.x, ch);
        } else {
            self.rows.push(Row::from_line(&ch.to_string()));
        }
        self.dirty = true;
    }
    pub fn delete_forwards(&mut self, cursor: Pos, noun: Noun) -> (Option<Coord>, Option<Coord>) {
        if let Some(row) = self.rows.get_mut(cursor.y) {
            if row.is_empty() || cursor.x >= row.len() - 1 {
                return (None, None);
            }
            let end_index = match noun {
                Noun::Line => row.len(),
                Noun::Char => std::cmp::min(cursor.x + 1, row.len()),
                Noun::Word => row.next_word_break(cursor.x),
            };
            row.splice(cursor.x..end_index, "");
            self.dirty = true;
        }
        (None, None)
    }
    pub fn delete_backwards(&mut self, cursor: Pos, noun: Noun) -> (Option<Coord>, Option<Coord>) {
        if let Some(row) = self.rows.get_mut(cursor.y) {
            if row.is_empty() || cursor.x == 0 {
                if cursor.y > 0 {
                    let prior_row_len = self.rows.get_mut(cursor.y - 1).unwrap().len();
                    self.join_lines(cursor.y - 1..cursor.y);
                    return (Some(prior_row_len), Some(cursor.y - 1));
                }
                return (None, None);
            }
            let start_index = match noun {
                Noun::Line => 0,
                Noun::Char => cursor.x - 1,
                Noun::Word => row.prev_word_break(cursor.x),
            };
            row.splice(start_index..cursor.x, "");
            self.dirty = true;
            (Some(start_index), None)
        } else {
            (None, None)
        }
    }
    pub fn join_lines(&mut self, range: std::ops::Range<Coord>) -> (Option<Coord>, Option<Coord>) {
        if self.rows.len() <= 1 {
            return (None, None);
        }

        let mut new_cursor_pos = Pos {
            x: 0,
            y: range.start,
        };
        let mut new_row = Row::default();
        for _ in range.start..=std::cmp::min(range.end, self.rows.len() - 1) {
            let mut row = self.rows.remove(range.start);
            new_cursor_pos.x = new_row.len();
            if !row.is_empty() && classify(row.char_at(row.len() - 1).unwrap()) != CharType::Space {
                row.append_str(" ");
            }
            new_row.append_row(row);
        }
        self.rows.insert(range.start, new_row);
        (Some(new_cursor_pos.x), Some(new_cursor_pos.y))
    }

    pub fn get_next_word_pos(&self, from: Pos) -> Option<Pos> {
        let mut iter = self.iter_from(from);
        let first_cp = iter.next()?;
        let mut last_class = classify(first_cp.ch);
        let mut prior_pos = first_cp.pos;
        for cp in iter {
            trace!("fwditer({:?})", cp);
            let new_class = classify(cp.ch);
            if new_class != last_class && new_class != CharType::Space {
                return Some(cp.pos);
            } else {
                last_class = new_class;
                prior_pos = cp.pos;
            }
        }
        Some(prior_pos)
    }
    pub fn get_prior_word_pos(&self, from: Pos) -> Option<Pos> {
        let mut iter = self.iter_from(from).rev();
        trace!("AAAA");
        iter.next()?;
        trace!("BBBB");
        let first_cp = iter.next()?;
        let mut last_class = classify(first_cp.ch);
        let mut prior_pos = first_cp.pos;
        trace!("BBBDBD");
        for cp in iter {
            trace!("reviter({:?})", cp);
            let new_class = classify(cp.ch);
            if new_class != last_class && new_class == CharType::Space {
                return Some(prior_pos);
            } else {
                last_class = new_class;
                prior_pos = cp.pos;
            }
        }
        Some(prior_pos)
    }
}

pub struct IterLines<'a> {
    row_iter: std::slice::Iter<'a, Row>,
}

impl<'a> Iterator for IterLines<'a> {
    type Item = &'a Row;
    fn next(&mut self) -> Option<Self::Item> {
        self.row_iter.next()
    }
}

pub struct IterChars<'a> {
    rows: &'a [Row],
    row: Option<&'a Row>,
    pos: Pos,
    y_offset: Coord,
}

pub struct IterCharsRev<'a> {
    rows: &'a [Row],
    row: Option<&'a Row>,
    pos: Pos,
    y_offset: Coord,
}

#[derive(Debug)]
pub struct CharPos {
    pub ch: char,
    pub pos: Pos,
}

impl<'a> IterChars<'a> {
    fn rev(self) -> IterCharsRev<'a> {
        IterCharsRev {
            rows: self.rows,
            row: self.row,
            pos: self.pos,
            y_offset: self.y_offset,
        }
    }
}

impl<'a> IterChars<'a> {
    fn advance(&mut self) {
        if let Some(row) = self.row {
            if self.pos.x == row.len() {
                self.pos.x = 0;
                self.pos.y += 1;
                self.row = self.rows.get(self.pos.y);
            } else {
                self.pos.x += 1;
            }
        }
    }
}

impl<'a> Iterator for IterChars<'a> {
    type Item = CharPos;
    fn next(&mut self) -> Option<Self::Item> {
        if let Some(row) = self.row {
            let ret = Some(CharPos {
                ch: row.char_at(self.pos.x)?,
                pos: Pos {
                    x: self.pos.x,
                    y: self.pos.y + self.y_offset,
                },
            });
            self.advance();
            return ret;
        }
        None
    }
}

impl<'a> IterCharsRev<'a> {
    fn advance(&mut self) {
        if self.pos.x == 0 {
            if self.pos.y == 0 {
                self.row = None;
                return;
            }
            self.pos.y -= 1;
            self.row = self.rows.get(self.pos.y);
            self.pos.x = self.row.unwrap().len();
        } else {
            self.pos.x -= 1;
        }
    }
}

impl<'a> Iterator for IterCharsRev<'a> {
    type Item = CharPos;
    fn next(&mut self) -> Option<Self::Item> {
        match self.row {
            None if self.pos.y == self.rows.len() => {
                if self.pos.y > 0 {
                    self.pos.y -= 1;
                    self.row = self.rows.get(self.pos.y);
                    self.pos.x = self.row.map(|row| row.len()).unwrap_or(0);
                }
                Some(CharPos {
                    ch: '\n',
                    pos: Pos {
                        x: self.pos.x,
                        y: self.pos.y + self.y_offset,
                    },
                })
            }
            Some(row) => {
                let ret = Some(CharPos {
                    ch: row.char_at(self.pos.x)?,
                    pos: Pos {
                        x: self.pos.x,
                        y: self.pos.y + self.y_offset,
                    },
                });
                self.advance();
                ret
            }
            None => None,
        }
    }
}
