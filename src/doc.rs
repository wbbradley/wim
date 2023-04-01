use crate::classify::{classify, CharType};
use crate::error::Result;
use crate::prelude::*;
use crate::rel::Rel;
use crate::row::Row;
use crate::types::{Coord, Pos};
use crate::undo::{Change, ChangeStack};
use crate::undo::{ChangeTracker, Op};
use crate::utils::read_lines;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Doc {
    filename: Option<String>,
    tracked_rows: Vec<Row>,
    dirty: bool,
    change_stack: ChangeStack,
}

#[allow(dead_code)]
impl Doc {
    pub fn empty() -> Self {
        Self {
            filename: None,
            tracked_rows: vec![Row::default()],
            dirty: false,
            change_stack: Default::default(),
        }
    }
    pub fn new_change_tracker(&mut self, pos: Pos) -> ChangeTracker {
        ChangeTracker::begin_changes(self, pos)
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
    #[must_use]
    pub fn pop_change(&mut self) -> Option<Pos> {
        let mut temp = ChangeStack::default();
        std::mem::swap(&mut self.change_stack, &mut temp);
        let pos = temp.pop(self);
        std::mem::swap(&mut self.change_stack, &mut temp);
        if pos.is_some() {
            // TODO: save last saved undo index and pass that around.
            self.dirty = true;
        }
        pos
    }
    #[must_use]
    pub fn push_change(&mut self, change: Change) -> Pos {
        let mut temp = ChangeStack::default();
        std::mem::swap(&mut self.change_stack, &mut temp);
        let pos = temp.push(self, change);
        std::mem::swap(&mut self.change_stack, &mut temp);
        self.dirty = true;
        pos
    }
    pub fn swap_rows(&mut self, range: &mut Range<Coord>, rows: &mut Vec<Row>) {
        assert!((0..=self.tracked_rows.len()).contains(&range.start));
        assert!((range.start..=self.tracked_rows.len()).contains(&range.end));
        let mut result_rows: Vec<Row> = self.tracked_rows[range.clone()].to_vec();
        let mut result_range: Range<Coord> = range.start..range.start + rows.len();
        self.tracked_rows
            .splice(range.clone(), rows.iter().cloned());
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
    pub fn render_line_slice(&self, pos: Pos, len: usize) -> &[char] {
        if pos.y < self.tracked_rows.len() {
            self.tracked_rows
                .get(pos.y)
                .unwrap()
                .get_slice(pos.x..pos.x + len)
        } else {
            &EMPTY[..]
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
    pub fn split_newline(&self, cursor: Pos) -> (Op, Pos) {
        if let Some(row) = self.tracked_rows.get(cursor.y) {
            let x = cursor.x.clamp(0, row.len());
            (
                Op::RowsSwap {
                    range: cursor.y..cursor.y + 1,
                    rows: row.split_at(x).to_vec(),
                },
                Pos {
                    x: 0,
                    y: cursor.y + 1,
                },
            )
        } else {
            panic!("what is this situation?");
        }
    }
    pub fn insert_newline(&self, y: Coord) -> Op {
        Op::RowsSwap {
            range: y..y,
            rows: vec![Row::from_line("")],
        }
    }
    pub fn insert_char(&self, cursor: Pos, ch: char) -> Op {
        if let Some(row) = self.tracked_rows.get(cursor.y) {
            Op::RowsSwap {
                range: cursor.y..cursor.y + 1,
                rows: vec![row.insert_char(cursor.x, ch)],
            }
        } else {
            Op::RowsSwap {
                range: cursor.y..cursor.y,
                rows: vec![Row::from_line(&ch.to_string())],
            }
        }
    }
    pub fn delete_forwards(&self, cursor: Pos, noun: Noun) -> Option<Op> {
        if let Some(row) = self.tracked_rows.get(cursor.y) {
            if row.is_empty() || cursor.x >= row.len() - 1 {
                return None;
            }
            let end_index = match noun {
                Noun::Line => row.len(),
                Noun::Char => std::cmp::min(cursor.x + 1, row.len()),
                Noun::Word => row.next_word_break(cursor.x),
            };
            Some(Op::RowsSwap {
                range: cursor.y..cursor.y + 1,
                rows: vec![row.splice(cursor.x..end_index, "")],
            })
        } else {
            panic!("what to do here?")
        }
    }
    pub fn delete_range(&self, mut start: Pos, mut end: Pos) -> Option<(Op, Pos)> {
        if start == end {
            return None;
        }
        if start > end {
            std::mem::swap(&mut start, &mut end);
        }
        let new_row = Row::joined_rows(
            &self.tracked_rows[start.y],
            &self.tracked_rows[end.y],
            start.x,
            end.x,
        );
        Some((
            Op::RowsSwap {
                range: start.y..end.y + 1,
                rows: vec![new_row],
            },
            start,
        ))
    }
    pub fn find_range(&self, cursor: Pos, noun: Noun, rel: Rel) -> (Pos, Pos) {
        if let Some(row) = self.tracked_rows.get(cursor.y) {
            match rel {
                Rel::Next => {
                    if row.is_empty() || cursor.x >= row.len() - 1 {
                        return (cursor, cursor);
                    }
                    let end_index = match noun {
                        Noun::Line => row.len(),
                        Noun::Char => std::cmp::min(cursor.x + 1, row.len()),
                        Noun::Word => row.next_word_break(cursor.x),
                    };
                    (
                        cursor,
                        Pos {
                            x: end_index,
                            y: cursor.y,
                        },
                    )
                }
                _ => {
                    panic!("unhandled {:?} {:?}", noun, rel);
                }
            }
        } else {
            panic!("wakka wakka");
        }
    }
    /*
    pub fn delete_backwards(&self, cursor: Pos, noun: Noun) -> Option<(Op, Pos)> {
        if let Some(row) = self.tracked_rows.get(cursor.y) {
            if row.is_empty() || cursor.x == 0 {
                if cursor.y > 0 {
                    let prior_row_len = self.rows.get(cursor.y - 1).unwrap().len();
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
    */
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
