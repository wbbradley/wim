use crate::error::Result;
use crate::noun::Noun;
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
    pub fn iter_lines(&self) -> IterLines {
        IterLines {
            row_iter: self.rows.iter(),
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
