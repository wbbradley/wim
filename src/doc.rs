use crate::error::Result;
use crate::noun::Noun;
use crate::row::Row;
use crate::types::{Coord, Pos, SafeCoordCast};
use crate::utils::read_lines;

pub struct Doc {
    filename: Option<String>,
    rows: Vec<Row>,
    dirty: bool,
}

impl Doc {
    pub fn get_filename(&self) -> Option<String> {
        self.filename.clone()
    }
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
    pub fn iter_lines(&self) -> IterLines {
        IterLines {
            row_iter: self.rows.iter(),
        }
    }
    pub fn line_count(&self) -> usize {
        self.rows.len()
    }
    pub fn get_line_buf(&self, y: Coord) -> Option<&Row> {
        self.rows.get(y as usize)
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
        let y = std::cmp::min(y as usize, self.rows.len());
        self.rows.splice(y..y, [Row::from_line("")]);
        self.dirty = true;
    }
    pub fn insert_char(&mut self, cursor: Pos, ch: char) {
        if let Some(row) = self.rows.get_mut(cursor.y as usize) {
            row.insert_char(cursor.x, ch);
        } else {
            self.rows.push(Row::from_line(&ch.to_string()));
        }
        self.dirty = true;
    }
    pub fn delete_forwards(&mut self, cursor: Pos, noun: Noun) -> (Option<Coord>, Option<Coord>) {
        match noun {
            Noun::Line => {
                if let Some(row) = self.rows.get_mut(cursor.y as usize) {
                    row.splice(cursor.x..row.len().as_coord(), "");
                    self.dirty = true;
                }
                (None, None)
            }
            Noun::Char => {
                if let Some(row) = self.rows.get_mut(cursor.y as usize) {
                    if cursor.x < row.len().as_coord() {
                        row.splice(cursor.x..cursor.x + 1, "");
                        self.dirty = true;
                    }
                }
                (None, None)
            }
        }
    }
    pub fn delete_backwards(&mut self, cursor: Pos, noun: Noun) -> (Option<Coord>, Option<Coord>) {
        match noun {
            Noun::Line => {
                if let Some(row) = self.rows.get_mut(cursor.y as usize) {
                    row.splice(0..cursor.x.as_coord(), "");
                    self.dirty = true;
                    (Some(0), None)
                } else {
                    (None, None)
                }
            }
            Noun::Char => {
                if let Some(row) = self.rows.get_mut(cursor.y as usize) {
                    if cursor.x > 0 {
                        row.splice(cursor.x - 1..cursor.x, "");
                        self.dirty = true;
                        return (Some(cursor.x - 1), None);
                    }
                }
                (None, None)
            }
        }
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
