# Narration sentence blocks

The source workflow plans text into independently identified sentence blocks using a saved narration profile. Every block retains an immutable copy of its resolved profile, exact display text, scalar source offsets and prepared spoken intent. These strings describe the requested pronunciation; they are not an acoustic transcript.

Run from the repository with Bun:

```sh
bun run apps/agent-bridge/src/narration-blocks-cli.ts
```

Keep the process open and send one JSON object per line. It uses the existing bridge environment, local provider and headless configuration. The commands are:

- `{"command":"plan","input":{"sourceText":"A. B.","profileId":"<saved profile UUID>","expectedVersion":1,"maxTextCharacters":4096,"boundaries":[3,5]}}`
- `{"command":"preview","plan":<result from plan>}` creates a retained group. Inspect every completed block before publishing. A partial result never publishes. To resume explicitly, resend the same plan with its `groupId`; completed tokens are reused.
- `{"command":"commit","groupId":"<group>","projectId":"<project>","expectedRevision":0,"trackId":"<audio track>","startMs":100}` publishes all completed blocks in one native transaction. Start positions use actual generated durations. Optional `markerPolicy` and `caption:{"trackId":"<caption track>","granularity":"sentence"}` require actual compatible alignment; captions additionally require a named alignment model.
- `{"command":"discard","groupId":"<group>"}` releases an unpublished group.
- `{"command":"reconcile","groupId":"<group>"}` checks an uncertain publication against the current native manifest, assets, provenance and requested selections. An absent or changed manifest stays uncertain. It never republishes.
- `{"command":"abandon","groupId":"<group>"}` disposes retained files while reporting any unresolved publication.

Offsets and manual boundary ends count Unicode scalar values, including each combining mark separately. An emoji outside the BMP counts once. Boundaries must increase, cover the entire source and produce nonempty trimmed requests. Display text preserves original whitespace; effective request text uses the ECMAScript whitespace trim set, including U+FEFF and excluding U+0085 (NEL). Automatic boundaries use pinned Unicode sentence rules and can split abbreviations such as `Dr.`; use explicit boundaries when reviewing such text. Paragraph order is preserved. No text is silently truncated or split to fit a provider.

The limits are 32 KiB source, 32 blocks, 4096 UTF-8 bytes and at most 4096 provider-limited scalars per display/spoken block, 32 spoken segments per block, 128 KiB effective request and 512 KiB manifest. A project retains at most 32 manifests, 256 live blocks and 4 MiB total manifest JSON. One publication creates at most 100 combined caption/cue references. Input lines are bounded to 16 MiB and responses to 1 MiB. A session holds at most four groups and 128 tokens including cleanup debt. All block preparation is checked before inference.

Preview expiry is preserved during a lease. Revision conflicts and confirmed prepublication failures retain usable previews until their original expiry; a lost acknowledgement quarantines the entire group. Inspect and reconcile before beginning replacement work. Closing or interrupting waits for active publication to settle, then disposes owned previews; uncertainty is reported explicitly. The CLI exits nonzero after a command error.

Published audio and optional caption/cue bindings are native project data in schema47, retained across undo/redo and reopening without the shared profile library. Existing edits that invalidate owned bindings fail atomically. The headless `detach_narration_block` operation removes one ownership entry without changing audio, captions, cues or unrelated blocks; detached content can then be edited normally. Duplicating native audio creates unowned content.

Schema1–46 projects and retained history migrate to schema47 with empty manifests. EditDraftv2 remains unchanged. Style pack v1 components remain schema46; capture accepts compatible schema46/47 sources and portable export/import preserve their existing contract. Future project versions and forged introduced metadata are rejected.

This is a source JSON-lines workflow using the configured local TTS worker. It does not add installed GUI controls or MCP tools. Read provider readiness first; unavailable local models remain unavailable. Native acceptance uses a simulated synthesizer producing real WAVs plus actual headless and FFprobe in both source and packaged sidecar platform checks. That evidence does not establish real model quality or interoperability with unmerged PRs.
