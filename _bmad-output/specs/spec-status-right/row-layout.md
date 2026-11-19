# Row layout: status at the right edge

Tab rows at width 26, `|` marking the sidebar's edges. Name and program cut exactly as today; only the
padding moves from after the status to before it.

Before (current `draw.rs`):

```
| ❯ Fix login redire ✳ ⚙ 2m|
|   deploy · vim ⏱ 4m      |
|   lde@app01 · ssh ✓ 3m12s|
|   infra ✗ 2              |
|   Ready specs ✳ ● waits  |
|   notes                  |
```

After:

```
| ❯ Fix login redire ✳ ⚙ 2m|
|   deploy · vim       ⏱ 4m|
|   lde@app01 · ssh ✓ 3m12s|
|   infra               ✗ 2|
|   Ready specs ✳   ● waits|
|   notes                  |
```

Unchanged: group rows, blank lines, the clock line, a row that fills the width (rows 1 and 3).
