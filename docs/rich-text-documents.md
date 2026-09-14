# Rich text documents

Schema 18 text items store `document: {runs: [{text, bold?, italic?, color?}]}` and retain `text` as the exact concatenation for compatible readers. Protocol 1 reports `rich_text_documents`. Existing simple text requests remain supported.

`timeline_add_text` accepts either `text` or `document`; `timeline_update_item` accepts at most one. Both fields together and explicit null are invalid. Simple text creates/replaces the document with one unstyled run. A style-only update retains all runs. Existing component-create/update requests with plain text-item records may omit `document`; core constructs the same one-run document while schema-18 persisted items still require it. Document edits also work in `timeline_batch_edit`, with normal aliases, revision checks, rollback, undo/redo and durable drafts.

```json
{"document":{"runs":[{"text":"Hello ","bold":true},{"text":"world","color":"#ff8800"}]}}
```

Documents contain 1-256 runs and 1-4096 total UTF-8 bytes. Empty individual runs are legal; empty aggregate text is not. Run order and Unicode, spaces and newlines are preserved exactly; run boundaries do not create line breaks. No string offsets are exposed. Unknown fields, null styles, invalid Unicode and invalid colors fail with INVALID_ARGUMENT. Text is literal: markup is not executed, and runs cannot reference external paths, fonts or URLs.

Item font, size, color and layout remain defaults. Optional run styles retain six-digit hex color semantics. Common transforms, anchors, stacking and integer-millisecond half-open timing are unchanged. Through schema 18, plain documents preserve the legacy render path. Schema 19 pins all four style faces and uses versioned shaping for both plain and styled text; see [content-addressed text layout](text-layout.md). Missing styled dependencies fail during font resolution with DEPENDENCY_UNAVAILABLE, while damaged retained bytes fail with ASSET_INTEGRITY_FAILED.

Template text/rich-text slots replace the effective document under the existing override/default precedence. Each instance resolves independently; shared definitions remain unchanged. Frame, range, draft preview and export consume the same evaluated text.

Opening supported schemas 1-17 migrates current state and every retained undo/redo snapshot under the project lock in one recoverable transaction. Each old text item, including hidden/unused component content, gains one unstyled run with its original string. Other content, revisions, references and assets remain unchanged. Invalid or future state fails before publication; schema-18 documents must match their compatibility text. Reopening does not repeat migration. Older binaries reject schema 18; rollback requires a complete pre-migration backup, not a downgrade.

Schema 18 is now an intermediate migration step. The current schema is 19;
font activation can change kerning, ligatures and wrapping once, then retains
the exact font bytes and layout profile across reopen, history and drafts.
