//! Ring 2 · Elaboration · **Layer 3 (Refinements) — assumption stack**
//!
//! A tiny stack of *frames*, each holding a list of predicates currently
//! assumed to be true. Frames match the lexical structure of the program:
//! entering a function or block pushes a frame; leaving pops it. Adding an
//! assumption always lands in the current frame.
//!
//! Kept as its own file so future path-sensitivity refinements (e.g.
//! per-basic-block frames after CFG construction) are a one-file change.

use ast::Expression;

#[derive(Debug, Default, Clone)]
pub struct AssumptionStack {
    frames: Vec<Vec<Expression>>,
}

impl AssumptionStack {
    pub fn New() -> Self {
        Self { frames: vec![Vec::new()] }
    }

    pub fn EnterFrame(&mut self) {
        self.frames.push(Vec::new());
    }

    pub fn LeaveFrame(&mut self) {
        if self.frames.len() > 1 {
            self.frames.pop();
        }
    }

    pub fn PushAssumption(&mut self, e: Expression) {
        if let Some(top) = self.frames.last_mut() {
            top.push(e);
        }
    }

    /// Flatten every frame into one `Vec` — this is what Layer 4 sees.
    pub fn Snapshot(&self) -> Vec<Expression> {
        self.frames.iter().flatten().cloned().collect()
    }
}
