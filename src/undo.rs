use crate::doc::Doc;
use crate::row::Row;

#[derive(Default, Debug)]
pub struct ChangeStack {
    changes: Vec<Change>,
    index: usize,
}

impl ChangeStack {
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
}

#[derive(Default, Debug)]
pub struct Change {
    ops: Vec<Op>,
}

#[derive(Debug)]
enum Op {
    RowSwap(RowSwap),
}

#[derive(Debug)]
pub struct RowSwap {
    pub index: usize,
    pub row: Row,
}

impl Change {
    pub fn execute(&mut self, doc: &mut Doc) {
        for op in &mut self.ops {
            match op {
                Op::RowSwap(ref mut row_op) => {
                    doc.swap_row(row_op.index, &mut row_op.row);
                }
            }
        }
        self.ops.reverse();
    }
}
