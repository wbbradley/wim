use crate::doc::Doc;
use crate::prelude::*;
use crate::row::Row;

#[derive(Default, Debug)]
pub struct ChangeStack {
    changes: Vec<Change>,
    index: usize,
}

impl ChangeStack {
    pub fn clear(&mut self) {
        self.index = 0;
        self.changes.truncate(0);
    }
    #[must_use]
    pub fn push(&mut self, doc: &mut Doc, mut change: Change) -> Pos {
        self.changes.truncate(self.index);
        let pos = change.execute(doc);
        self.changes.push(change);
        self.index += 1;
        pos
    }

    #[allow(dead_code)]
    #[must_use]
    pub fn pop(&mut self, doc: &mut Doc) -> Option<Pos> {
        if self.index > 0 {
            self.index -= 1;
            Some(self.changes[self.index].execute(doc))
        } else {
            None
        }
    }
}

#[derive(Debug)]
pub struct ChangeTracker<'a> {
    doc: &'a mut Doc,
    before_cursor: Pos,
    after_cursor: Option<Pos>,
    ops: Vec<Op>,
}

impl<'a> ChangeTracker<'a> {
    pub fn begin_changes(doc: &'a mut Doc, cursor: Pos) -> Self {
        Self {
            doc,
            before_cursor: cursor,
            after_cursor: None,
            ops: Default::default(),
        }
    }
    #[must_use]
    pub fn commit(self) -> Pos {
        self.doc.push_change(Change {
            before_cursor: self.before_cursor,
            after_cursor: self.after_cursor.unwrap_or(self.before_cursor),
            ops: self.ops,
        })
    }
    pub fn add_op(&mut self, op: Op, cursor: Pos) {
        self.ops.push(op);
        self.after_cursor = Some(cursor);
    }
}

#[derive(Default, Debug)]
pub struct Change {
    ops: Vec<Op>,
    before_cursor: Pos,
    after_cursor: Pos,
}

#[derive(Debug)]
pub enum Op {
    RowsSwap { range: Range<usize>, rows: Vec<Row> },
}

impl Change {
    #[must_use]
    pub fn execute(&mut self, doc: &mut Doc) -> Pos {
        for op in &mut self.ops {
            match op {
                Op::RowsSwap { range, rows } => {
                    doc.swap_rows(range, rows);
                }
            }
        }
        // Flip this change so that it executes in reverse next time.
        std::mem::swap(&mut self.before_cursor, &mut self.after_cursor);
        self.ops.reverse();
        self.before_cursor
    }
}
