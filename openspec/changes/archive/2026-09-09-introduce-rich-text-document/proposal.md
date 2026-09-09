## Why

Issue [#32](https://github.com/matiHirCab/AI-Open-Cut/issues/32) requires ordinary text items to own a RichTextDocument while preserving simple text requests and existing output. The core already implements run-based rich text for template slots, but TextItem persists only a string; prerequisite #17 is closed.

## What Changes

- Persist a required `document` on text items using the existing closed `{runs:[{text,bold?,italic?,color?}]}` RichTextDocument contract. Retain `text` as its concatenated compatibility projection and existing item-level typography/layout defaults.
- Accept optional `document` in existing text creation and item-update operations, including batches; simple `text` creates or replaces a document with one unstyled run. Reject simultaneous text/document request fields.
- Migrate supported schemas 1-17, current state and every retained undo/redo snapshot, atomically to schema 18. Preserve legacy content, rendering, revisions, references and provenance.
- Route stored documents and template overrides through the shared evaluator; retain the legacy render path for semantically plain documents.
- Add bounded validation, canonical fixtures, protocol/MCP parity, capability reporting and regression coverage.

## Capabilities

### New Capabilities

- `rich-text-documents`: Canonical text-item documents, validation, compatibility inputs, and reversible edits.

### Modified Capabilities

- `project-persistence`: Atomic schema-18 text-document migration and source-version validation.
- `agent-bridge`: Additive document input/output and support reporting across headless/MCP contracts.
- `rendering-export`: Stored document evaluation, template precedence, and unchanged legacy output.

## Non-goals

Content-addressed fonts and a new shaping backend (#33), grapheme-indexed range editing, new font/style fields, auto-fit, additional effects, caption conversion, and desktop inspector features are excluded. The larger roadmap describes future rich-text features; this proposal activates the existing run vocabulary only.

## Impact

Core model, validation, timeline mutations, migration, component bindings, evaluated scene, renderer preparation, and their tests; typed headless union, bridge schemas/registration, project-state schemas, capabilities, canonical fixtures and governed consumers; persistence/text documentation and render fixtures. No new dependency edge or provider protocol is intended.

Public protocol 1 remains additive: existing simple requests and compatibility response fields remain valid. The persisted schema advances to 18; older binaries reject it, with no automatic downgrade. Existing slot documents retain their wire shape and limits. Canonical ownership follows contracts/contract-ownership-v1.json; @matiHirCab review is required for changed public contracts. The user approved this proposal in the task on 2026-09-09. Final contract review evidence is recorded in verification.md.
