# Desktop hierarchy inspection

The desktop can open an existing local project through the same core used by headless and MCP:

```sh
cargo run -p opencut-desktop -- --project-store /path/to/projects --project-id <project-id>
```

Both arguments are required together. With neither, the app shows an empty state. The store is the directory containing project-ID directories, not an individual project directory. The session uses core path policy, validation and schema migrations; it never reads a project through a separate desktop decoder. No new wire operation, capability flag or persisted schema is introduced.

Expand groups and instances in **Hierarchy**. Repeated component contents have separate selections identified by their complete instance path and composition scope. Hidden layers remain inspectable. The inspector shows the owning track, local time, duration, parent, z-index and stack order. Instance rows also show stored slot overrides; component-local children show stored definition properties and are read-only. The tree stops at 4096 displayed rows; collapse branches to inspect other content. This is a presentation limit, not a project validity rule.

Tree indentation shows parentage, including parents on another track. It does not show compositing order. Paint order is track index, then signed z-index, then stable item order. Higher entries composite above lower entries. A component occurrence is a contiguous stacking block. Root **Timeline** rows share selection with Hierarchy and Inspector.

Select a root visual item to edit its z-index or parent. Click the z-index field, use Ctrl+A to clear, type a signed integer, and press Enter or click Apply. Backspace removes the last character; Escape restores the stored value. The accepted range is -2147483648 through 2147483647. Select a root group or Detach to change parent. Reparenting preserves local coordinates and can therefore move the content in world space. Component-local and nonvisual items have no parent/z-index controls.

Edits, Undo and Redo use the displayed revision, run outside the render callback and reload the authoritative snapshot on success. Only one request runs at a time. Missing references, cycles, locked tracks and revision conflicts retain core error codes and retryability. Conflicting edits are not retried automatically: use Refresh to read external changes, then decide whether to reapply. Selection survives refresh/history if its occurrence still exists. Reopening reconstructs the hierarchy from persisted state.

## Reproducible rule-card project

```sh
cargo run -p opencut-editor-core --example rule_card_fixture -- local-data/rule-card-demo
```

Supply a new directory. The example creates three synthetic managed icons, a one-second tone and a project containing three slotted rule cards; it prints the desktop startup arguments. It performs a parent move and undo, leaving an original-layout project with usable history. It never overwrites an existing fixture directory.

Each card has a background, accent, rule number, title, body and icon. Numbers are text slots; opacity independently exercises numeric slots. The shared definition is immutable during per-instance override resolution. Native frame/range/export and lifecycle evidence is described in [render regression fixtures](render-regression-fixtures.md).

## Native smoke checklist

Open the generated project, expand its parent and all three instances, and select equal local child IDs to inspect distinct instance paths. Select a root instance in the timeline and verify its text/icon/opacity overrides. Change z-index, detach and reparent, then Undo and Redo and reopen. Make an external core edit while the desktop snapshot is stale: verify a conflict, Refresh and the new revision. Also start without arguments and with a nonexistent project to inspect empty/error states. The preview panel remains a placeholder; this issue's render evidence runs through production core preview/export APIs.
