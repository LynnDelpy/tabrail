//! One sidebar width for every tab: the global option @sidebar_width. tmux spreads a terminal resize
//! over all panes of a tab, so when the tab's width or the shared width changes, the sidebar sets its
//! own pane back to the shared width. When its width changes while the tab's does not, its border was
//! dragged: once the width holds still, it becomes the shared width, and every other sidebar follows.
//! A layout command (tabrail.tmux marks the tab with @sidebar_relayout) puts it back instead.
use crate::control::Tmux;

/// The default when @sidebar_width is not set; `tabrail-tmux ensure` uses the same.
pub const DEFAULT: usize = 26;
/// What `keep` reads, appended to the sidebar's display query: tab width, shared width, the
/// zoomed/left/top/bottom/right flags ("01110": not zoomed, the full-height left column, not alone)
/// and whether a layout command ran.
pub const FORMAT: &str = "#{window_width}\t#{@sidebar_width}\t#{window_zoomed_flag}#{pane_at_left}\
                          #{pane_at_top}#{pane_at_bottom}#{pane_at_right}\t#{@sidebar_relayout}";

#[derive(Default)]
pub struct Width {
    seen: Option<(usize, usize)>, // tab width and shared width the sidebar last settled at
    pane: Option<usize>,          // its own width at the last frame; None right after resizing it
    moved: bool,                  // changed in place since: a border drag
}

impl Width {
    /// Keep `pane` (now `width` columns) at the shared width; `fields` are FORMAT's. True if it was
    /// resized, so this frame's width is stale.
    pub fn keep(&mut self, t: &mut Tmux, pane: &str, width: usize, fields: &[&str]) -> bool {
        let num = |i: usize| fields.get(i).and_then(|v| v.parse().ok());
        let (Some(window), shared) = (num(0), num(1).unwrap_or(DEFAULT)) else { return false };
        // zoomed: resize-pane would unzoom the tab; off the left column: a layout command moved it
        if fields.get(2) != Some(&"01110") {
            return false;
        }
        if fields.get(3) == Some(&"1") {
            t.ask(&format!("set -wu -t {pane} @sidebar_relayout"));
            self.seen = None;
        }
        if self.seen != Some((window, shared)) {
            (self.seen, self.moved) = (Some((window, shared)), false);
            if width != shared {
                self.pane = None; // the next frame takes whatever width tmux allowed
                t.ask(&format!("resize-pane -t {pane} -x {shared}"));
                return true;
            }
        } else if self.pane.is_some_and(|p| p != width) {
            self.moved = true;
        } else if self.moved && self.pane.is_some() {
            self.moved = false;
            if width != shared {
                self.seen = Some((window, width));
                t.ask(&format!("set -g @sidebar_width {width}"));
                t.ask("set-buffer -b sidebar-refresh x");
            }
        }
        self.pane = Some(width);
        false
    }
}
