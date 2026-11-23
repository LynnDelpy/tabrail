//! The reader bar's height, while tabrail-chat runs in it (tmux.rs finds out): after the input or
//! the conversation changed, the rows of the input and of the latest question and its answer (at
//! least one), rounded up to a multiple of 4, at most every 500 ms; while a new question is typed (no
//! turn running) the input's rows and one only. tabrail-reader takes it only while the bar has the
//! keyboard. The buttons ▲ ▼ make it a sixth of the window bigger or smaller,
//! a size that holds until the next question is sent or the bar is back at 4 rows.
use crate::input::Input;
use crate::tmux::{self, Ask};
use crate::view::View;
use std::sync::mpsc::{Receiver, Sender};
use std::time::{Duration, Instant};

pub struct Bar {
    asks: Sender<Ask>,
    found: Receiver<()>,              // a message once this pane is found to be the reader
    heights: Receiver<Option<usize>>, // the bar's height after each button's ask
    reader: bool,
    manual: bool,                     // a button set the height: none of its own until the next question
    pressed: usize,                   // buttons asked whose height is not back yet
    shown: usize,                     // the screen's height at the last frame
    last: Option<Instant>,            // when its own height was last asked for
    at: Option<(usize, usize)>,       // where ▲ is on the screen: row, column
}

impl Bar {
    pub fn new() -> Bar {
        let (asks, found, heights) = tmux::start();
        Bar { asks, found, heights, reader: false, manual: false, pressed: 0, shown: 0, last: None, at: None }
    }

    /// The input's rows and cursor (Input::rows) on a screen `height` rows high, in the reader
    /// with the buttons at the end of its last row, which is the screen's.
    pub fn input(&mut self, input: &Input, (width, height): (usize, usize), turn: Option<Instant>)
                 -> (Vec<String>, (usize, usize)) {
        self.reader = self.reader || self.found.try_recv().is_ok();
        let (rows, cursor, at) = input.rows(width, height.saturating_sub(1), turn, self.reader);
        self.at = at.map(|col| (height.saturating_sub(1), col));
        (rows, cursor)
    }

    /// A left press: on ▲ (or the space before it) a step bigger, on ▼ (or the space before it) a
    /// step smaller.
    pub fn click(&mut self, row: usize, col: usize) {
        let Some((_, c)) = self.at.filter(|&(r, _)| r == row) else { return };
        let ask = match col {
            col if (c.saturating_sub(1)..=c).contains(&col) => Ask::Up,
            col if (c + 1..=c + 2).contains(&col) => Ask::Down,
            _ => return,
        };
        (self.manual, self.pressed) = (true, self.pressed + 1);
        let _ = self.asks.send(ask);
    }

    /// A question was sent: the bar's own height again.
    pub fn sent(&mut self) {
        self.manual = false;
    }

    /// After a frame on a screen of `(width, height)`, a turn running or not: its own height when
    /// the input or the conversation changed, unless it is the height already (tmux.rs leaves out one
    /// tabrail-reader would not change either). A change within 500 ms of the last ask waits.
    pub fn fit(&mut self, (width, height): (usize, usize), turn: bool, input: &mut Input, view: &mut View) {
        // back at 4 rows (left there by a button, or shrunk: the own pane clicked, a tab switch): the
        // size a button set is gone, and with it the hold
        let after: Vec<Option<usize>> = self.heights.try_iter().collect();
        self.pressed = self.pressed.saturating_sub(after.len());
        let small = after.last().is_some_and(|h| h.is_some_and(|h| h <= 4)) || (self.shown > 4 && height <= 4);
        self.manual &= !(small && self.pressed == 0);
        self.shown = height;
        let soon = self.last.is_some_and(|t| t.elapsed() < Duration::from_millis(500));
        if !(input.changed || view.changed) || soon {
            return;
        }
        (input.changed, view.changed) = (false, false);
        if !self.reader || self.manual {
            return;
        }
        // a new question being typed: the input grows the bar, not the answer before (Lynn, 2026-11-23)
        let talk = if turn || input.is_empty() { view.latest(width).max(1) } else { 1 };
        let rows = (input.height(width) + talk).div_ceil(4) * 4;
        if rows != height {
            self.last = Some(Instant::now());
            let _ = self.asks.send(Ask::Rows(rows));
        }
    }
}
