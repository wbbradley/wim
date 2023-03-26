use crate::classify::{classify, CharType};
use crate::error::Result;
use crate::prelude::*;
use crate::row::Row;
use crate::types::{Coord, Pos};
use crate::utils::read_lines;

#[derive(Debug)]
#[allow(dead_code)]
pub struct Doc {
    filename: Option<String>,
    rows: Vec<Row>,
    dirty: bool,
}

#[allow(dead_code)]
impl Doc {
    pub fn empty() -> Self {
        Self {
            filename: None,
            rows: Vec::default(),
            dirty: false,
        }
    }
    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
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
            row_iter: self.rows[y..].iter(),
        }
    }
    pub fn iter_from(&self, pos: Pos) -> IterChars {
        IterChars {
            rows: &self.rows,
            row: self.rows.get(pos.y),
            pos,
            y_offset: 0,
        }
    }
    pub fn iter_line(&self, y: Coord) -> IterChars {
        if self.rows.len() > y {
            IterChars {
                rows: &self.rows[y..=y],
                row: self.rows.get(y),
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
    pub fn render_line(&self, pos: Pos) -> std::slice::Iter<'_, char> {
        if pos.y < self.rows.len() {
            self.rows.get(pos.y).unwrap().render_chars_from(pos.x)
        } else {
            EMPTY.iter()
        }
    }

    pub fn line_count(&self) -> usize {
        self.rows.len()
    }
    pub fn get_line_buf(&self, y: Coord) -> Option<&Row> {
        self.rows.get(y)
    }
    pub fn open(filename: String) -> Result<Self> {
        let mut doc = Doc::empty();
        doc.rows.truncate(0);
        let lines = read_lines(&filename)?;
        for line in lines {
            doc.rows.push(Row::from_line(&line?));
        }
        doc.filename = Some(filename);
        Ok(doc)
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
            if row.len() == 0 || cursor.x >= row.len() - 1 {
                return (None, None);
            }
            let end_index = match noun {
                Noun::Line => row.len(),
                Noun::Char => std::cmp::min(cursor.x + 1, row.len() - 1),
                Noun::Word => row.next_word_break(cursor.x),
            };
            row.splice(cursor.x..end_index, "");
            self.dirty = true;
        }
        (None, None)
    }
    pub fn delete_backwards(&mut self, cursor: Pos, noun: Noun) -> (Option<Coord>, Option<Coord>) {
        if let Some(row) = self.rows.get_mut(cursor.y) {
            if row.len() == 0 || cursor.x == 0 {
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
    pub fn join_lines(&mut self, range: std::ops::Range<Coord>) {
        if self.rows.len() <= 1 {
            return;
        }

        let mut new_row = Row::default();
        for _ in range.start..=std::cmp::min(range.end, self.rows.len() - 1) {
            let row = self.rows.remove(range.start);
            new_row.append_row(&row);
        }
        self.rows.insert(range.start, new_row);
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
