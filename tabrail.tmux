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
# than taking the new one (src/width.rs); prefix Space as select-layout -n, as next-layout has no hook
bind Space select-layout -n
set-hook -g after-select-layout[50] "set -w @sidebar_relayout 1"
# prefix M: move this tab to another group
bind M run-shell -b "#{@tabrail_dir}/tabrail-move '#{window_id}' '#{client_name}'"

# Every new tab gets a sidebar; a tab is closed when only its sidebar is left. A command of 10 s or
# more that finished out of sight leaves ✓ until you look (needs OSC 133 marks from the shell).
set-hook -g after-new-window[50] "run-shell -b '#{@tabrail_dir}/tabrail-tmux ensure #{window_id}'"
set-hook -g after-new-session[50] "run-shell -b '#{@tabrail_dir}/tabrail-tmux ensure'"
set-hook -g client-attached[50] "run-shell -b '#{@tabrail_dir}/tabrail-tmux ensure'"
set-hook -g after-select-window[50] "set -wu @finished ; set-buffer -b sidebar-refresh x"
set-hook -g pane-exited[50] "run-shell -b '#{@tabrail_dir}/tabrail-tmux reap'"
set-hook -g after-kill-pane[50] "run-shell -b '#{@tabrail_dir}/tabrail-tmux reap'"
set-hook -g pane-command-finished[50] 'if -F "#{&&:#{e|>=:#{pane_command_duration},10},#{!:#{&&:#{window_active},#{session_attached}}}}" "set -w @finished 1 ; set-buffer -b sidebar-refresh x"'

run-shell -b "#{@tabrail_dir}/tabrail-tmux ensure"
