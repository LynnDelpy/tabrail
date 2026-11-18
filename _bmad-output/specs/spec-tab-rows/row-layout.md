# Row layout

Before (current `draw.rs`):

```
 ▾ ◆ Website
  ❯ 1 Fix login redirect ⚙ 2m
    2 website·release-2 ◇

 ▾ ▲ Infra
    3 TERM=xterm-256color ⏱ 4m
    4 infra ✗ 2

 ▸ ● Notes  2 tabs
```

After:

```
▾ ◆ Website
 ❯ Fix login redirect ⚙ 2m
   website·release-2 ◇

▾ ▲ Infra
   lde@app01 infra ⏱ 4m
   infra ✗ 2

▸ ● Notes  2 tabs
```

Columns (1-based):

| Row | 1 | 2 | 3 | 4 | 5 on |
|---|---|---|---|---|---|
| Group | `▾` or `▸` | space | icon | space | name, then `  N tabs` when folded |
| Tab | space | `❯` (current) or space | space | name… | name, space, status |

Unchanged: the tint across the full group row, the blank line after each group, status right after the name with a long name cut to make room, the clock on the last line.
