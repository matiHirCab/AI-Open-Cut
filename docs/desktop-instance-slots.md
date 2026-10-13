# Desktop instance slots

Select a root component instance in the hierarchy and use its **Instance slots** inspector. Slots come from the current component definition in declared order; the inspector shows ID, name, kind, binding, required state, shared default and stored override. Source labels distinguish an override, a shared definition default and an absent optional value that retains the bound source property. The inspector does not calculate a second effective scene.

Text, solid color, `visual.hidden` Boolean and `media.asset` slots expose controls. Text and newlines remain literal (`Shift+Enter` inserts a newline); color preserves the typed hex6 casing and the bound paint's existing alpha semantics. Hidden has explicit true/false choices. Asset choices use existing managed project assets; the field also accepts a literal managed ID, with missing or incompatible assets rejected by core. The list shows at most128 assets, and the inspector shows at most128 definition slots. Component-local occurrences and rich text, number, enum, duration or unsupported bindings remain visibly read-only.

**Apply override** submits the displayed expected revision to the existing core component-instance update. It changes one literal map key and retains other overrides, component reference, timing, stagger and static visual properties. **Clear override · use shared default/source** removes that key through a revisioned edit; required slots without a default can reject this action. **Draft Reset · discard input** discards uncommitted input and leaves project/history unchanged. Empty text is an explicit value, rather than a clear action.

The input buffer holds16384 UTF-8 bytes without truncating4096 supplementary Unicode scalars. Core still owns admission: slot scalar/declared constraints and the resolved text1–4096 UTF-8-byte limit both apply. Buffer exhaustion blocks Apply until correction or reset. Boolean parsing requires exactly `true` or `false`; semantic failures retain core code, message and retryability. No UI constraint validator, importer or renderer is added.

Slot drafts retain complete occurrence selection, revision, component ID and slot ID/kind/binding. Choosing another field/slot, selection/tab changes, Reset, refresh, dispatch/history and successful completion discard drafts. Busy work serializes mutations; a revision conflict requires explicit **Refresh**, then selecting and replanning the edit. There is no optimistic publication or automatic retry. Undo, redo and reopening use existing core/session operations.

## Disposable source fixture

Create a new directory using the desktop example, then start the source app with its printed arguments:

```sh
cargo run -p opencut-desktop --example instance_slot_fixture -- /tmp/new-instance-slot-fixture
cargo run -p opencut-desktop -- --project-store /tmp/new-instance-slot-fixture/projects --project-id <printed-id>
```

The generator rejects an existing directory. It reuses original synthetic rule-card artwork and core APIs, retaining the older fixture unchanged. Two independently customized root cards share a definition; two wrapper occurrences expose repeated `nested` local instance IDs for read-only inspection. The definition includes supported/default/required/absent slots, special map IDs (`__proto__`, `constructor`, `toString`) and unsupported number, enum, duration and rich text. Managed bitmap and audio fixtures require no downloaded or private media.

Focused tests compare complete authoritative snapshots and failure inventories, preserve managed resources and independent occurrences, exercise exact canonical errors, legal/over-limit Unicode representation, history/reopen, pending context, and the authoritative128-slot boundary. Run `cargo test -p opencut-desktop instance_slots_tests` along with the complete repository gates.

Actual GUI evidence must identify source/build, input events, settled screenshots and authoritative project revisions/inventories for all four families, both resets, canonical failure/conflict/lock feedback, read-only slots and lifecycle. A black/empty baseline or screenshot alone does not establish acceptance. Software Vulkan/private-display measurements apply only to that verification environment; they make no installed-production, portable-performance or render-quality claim.
