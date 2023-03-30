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
    pub fn push(&mut self, doc: &mut Doc, mut change: Change) {
        self.changes.truncate(self.index);
        change.execute(doc);
        self.changes.push(change);
    }

    pub fn pop(&mut self, doc: &mut Doc) {
        if self.index > 0 {
            self.index -= 1;
        }
        self.changes[self.index].execute(doc);
    }
}

#[derive(Debug)]
pub struct ChangeTracker<'a> {
    doc: &'a mut Doc,
    change: Change,
}

impl<'a> ChangeTracker<'a> {
    pub fn begin_changes(doc: &'a mut Doc) -> Self {
        Self {
            doc,
            change: Default::default(),
        }
    }
    pub fn commit(self) {
        self.doc.push_change(self.change);
    }
    pub fn add_rows_swap(
        &mut self,
        range: Range<usize>,
        rows: Vec<Row>,
        before_cursor: Pos,
        after_cursor: Pos,
    ) {
        self.change
            .push_op(Op::RowsSwap { range, rows }, before_cursor, after_cursor);
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
    pub fn push_op(&mut self, op: Op, before_cursor: Pos, after_cursor: Pos) {
        if self.ops.is_empty() {
            self.before_cursor = before_cursor;
        }
        self.ops.push(op);
        self.after_cursor = after_cursor;
    }
    #[must_use]
    pub fn execute(&mut self, doc: &mut Doc) -> Pos {
        for op in &mut self.ops {
            match op {
                Op::RowsSwap(ref mut row_op) => {
                    // NB: implicitly swapping redo/undo info inside this call.
                    doc.swap_rows(&mut row_op.range, &mut row_op.rows);
                }
            }
        }
        // Flip this change so that it executes in reverse next time.
        std::mem::swap(&mut self.before_cursor, &mut self.after_cursor);
        self.ops.reverse();
        self.before_cursor
    }
}
