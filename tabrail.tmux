# tabrail: a sidebar on the left of every tmux tab, all tabs grouped by project. Load it from
# tmux.conf with  source-file /path/to/tabrail/tabrail.tmux  (mouse on); point @tabrail_groups at your
# groups file before that, else groups.conf next to this file is used (groups.conf.example).
# Hooks use fixed slots ([50]) so a reload replaces them. Switches, renames and closes reach the
# sidebars over their tmux control connection; option changes by touching the sidebar-refresh buffer.

set -gF @tabrail_dir "#{d:current_file}"
%hidden MENUS="$XDG_RUNTIME_DIR/tabrail"

# A click into a sidebar goes to the sidebar but does not move the keyboard there; letting go of the
# button goes to the program under it too (the sidebar drags tabs and groups with it)
bind -n MouseDown1Pane if -F -t = "#{@sidebar}" "send-keys -M -t =" "select-pane -t = ; send-keys -M -t ="
bind -n MouseUp1Pane send-keys -M -t =
bind -n MouseDragEnd1Pane send-keys -M -t =
# Right button up over a sidebar: tmux opens the menu the sidebar keeps for that row (a menu takes
# clicks only when opened from a mouse binding like this one)
bind -n MouseUp3Pane if -F -t = "#{@sidebar}" "source-file -q -F '$MENUS/#{pid}-#{pane_id}-#{mouse_y}.conf'" "send-keys -M -t ="
# A layout command resizes the sidebar too: mark the tab so it goes back to the shared width rather
# than taking the new one (src/width.rs); it rearranges the reader bar or the tab's slot as well, which
# goes back across the bottom (tabrail-reader fit); prefix Space as select-layout -n, as next-layout
# has no hook
bind Space select-layout -n
set-hook -g after-select-layout[50] {
  set -w @sidebar_relayout 1
  if -F "#{&&:#{@tabrail_reader},#{P:#{@reader}#{@reader_slot}}}" {
    run-shell -b "#{@tabrail_dir}/tabrail-reader fit '#{window_id}'"
  }
}
# prefix M: move this tab to another group
bind M run-shell -b "#{@tabrail_dir}/tabrail-move '#{window_id}' '#{client_name}'"

# Every new tab gets a sidebar; a tab is closed when only its sidebar (and the reader pane) is left.
# A command of 10 s or more that finished out of sight leaves ✓ until you look (needs OSC 133 marks
# from the shell).
set-hook -g after-new-window[50] "run-shell -b '#{@tabrail_dir}/tabrail-tmux ensure #{window_id}'"
set-hook -g after-new-session[50] "run-shell -b '#{@tabrail_dir}/tabrail-tmux ensure'"
set-hook -g client-attached[50] "run-shell -b '#{@tabrail_dir}/tabrail-tmux ensure'"
set-hook -g after-select-window[50] "set -wu @finished ; set-buffer -b sidebar-refresh x"
set-hook -g pane-exited[50] "run-shell -b '#{@tabrail_dir}/tabrail-tmux reap'"
set-hook -g after-kill-pane[50] "run-shell -b '#{@tabrail_dir}/tabrail-tmux reap'"
set-hook -g pane-command-finished[50] 'if -F "#{&&:#{e|>=:#{pane_command_duration},10},#{!:#{&&:#{window_active},#{session_attached}}}}" "set -w @finished 1 ; set-buffer -b sidebar-refresh x"'

# The reader pane, only if @tabrail_reader names its command (tabrail-reader): a bar across the bottom
# of the tab shown that follows it, also into another session, swapping places with each tab's slot;
# a double-click into it grows it or shrinks it again, the tab's own pane taking the keyboard shrinks
# it (the sidebar does not); it and the slots get their size back after a resize
set-hook -g session-window-changed[50] {
  if -F "#{&&:#{@tabrail_reader},#{!:#{P:#{@reader}}}}" {
    run-shell -b "#{@tabrail_dir}/tabrail-reader follow '#{session_id}'"
  }
}
set-hook -g client-session-changed[50] {
  if -F "#{&&:#{@tabrail_reader},#{!:#{P:#{@reader}}}}" {
    run-shell -b "#{@tabrail_dir}/tabrail-reader follow '#{session_id}'"
  }
}
set-hook -g window-pane-changed[50] {
  if -F "#{&&:#{@tabrail_reader},#{P:#{@reader}}}" {
    run-shell -b "#{@tabrail_dir}/tabrail-reader fit '#{window_id}'"
  }
}
set-hook -g window-resized[50] {
  if -F "#{&&:#{@tabrail_reader},#{P:#{@reader}#{@reader_slot}}}" {
    run-shell -b "#{@tabrail_dir}/tabrail-reader fit '#{window_id}'"
  }
}
# A double-click into the bar: tabrail-reader grow; elsewhere tmux's own (select and copy a word)
bind -n DoubleClick1Pane if -F -t = "#{@reader}" {
  select-pane -t = ; run-shell -b -t = "#{@tabrail_dir}/tabrail-reader grow '#{window_id}'"
} {
  select-pane -t = ; if -F "#{||:#{pane_in_mode},#{mouse_any_flag}}" { send-keys -M } { copy-mode -H ; send-keys -X select-word ; run-shell -d 0.3 ; send-keys -X copy-pipe-and-cancel }
}
# prefix &: close the tab like the menus' Close, the reader pane moving on first
bind & confirm-before -p "kill-window #W? (y/n)" {
  run-shell -b "#{@tabrail_dir}/tabrail-tmux close '#{window_id}'"
}

run-shell -b "#{@tabrail_dir}/tabrail-tmux ensure"
