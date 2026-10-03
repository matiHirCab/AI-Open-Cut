## MODIFIED Requirements

### Requirement: Provenance follows primitive lifecycle
Successful preset replacement MUST replace that identity's source record without accumulating an application log. `set_animation_channels` MUST clear all source records on its item, including when its replacement is byte-equivalent or empty. Other accepted operations changing/removing/retiming an authored channel MUST clear that identity's source record; operations preserving its source primitives and effective source clock under animation-edit-semantics MUST preserve its known source record. Raw full component track/document inputs MUST NOT introduce trusted source labels: new definitions MUST start without them, and replacements MUST retain only previously known records for the same scope/item/identity with unchanged complete channels, including any retained clock. Exact core copies MUST retain source records with equivalent source primitives and effective source clocks; materializing an implicit zero clock on a duplicate is equivalent and MUST NOT change original compilation attribution. Deleting an item MUST delete its records. Undo/redo MUST restore primitives and records together. Ordinary unclocked authored-channel updates MUST retain existing item-duration key bounds and compiler half-open timing constraints; only approved split, trim and duplicate operations may implicitly materialize validated retained clocks under animation-edit-semantics. Split and trim MUST follow animation-edit-semantics by preserving exact source keys, curves, loops and descriptive original compilation attribution while composing validated retained source clocks; retained channel keys MUST satisfy their effective source-duration bounds even when outside the shorter edited item window. This MUST NOT automatically retime source keys, validate malformed source records, relax edit boundaries or bypass candidate safety constraints. Newly compiled or replaced preset channels MUST use fresh item-local timing without retained clocks; unrelated typed channels and legacy animation MUST retain their existing independent clocks.

#### Scenario: Clear labels through the raw setter and restore by undo
- **WHEN** an item with preset source records receives `set_animation_channels`, including an identical or empty replacement, then undo and redo
- **THEN** the setter clears all its labels, undo restores exact prior primitives/labels, and redo returns to the unlabeled replacement

#### Scenario: Preserve copies and unrelated edits
- **WHEN** core makes an exact duplicate or a move, static/base-volume, visibility or parenting edit leaves source channels and effective source clocks unchanged
- **THEN** known source records remain associated with equivalent source primitives and effective source clocks without recompilation

#### Scenario: Reconcile raw component replacement
- **WHEN** a raw component document changes one previously labeled channel, preserves another, or submits labels on new items
- **THEN** only previously known labels for unchanged scoped identities survive and submitted labels cannot label newly authored output

#### Scenario: Preserve existing duration and split outcomes
- **WHEN** duration or split violates edit boundaries, ordinary unclocked authored-channel update bounds, retained source-clock/source-key bounds or candidate safety rules, or an accepted operation changes an authored primitive
- **THEN** the invalid edit fails atomically or the changed primitive's label is cleared, without automatic retiming

#### Scenario: Preserve preset source attribution through retained edits
- **WHEN** a supported split or trim shortens a preset-generated item window while its original source keys remain valid against the retained source duration, or core duplicates that animated item
- **THEN** the edit preserves exact source keys, curves, loops and original descriptive compilation parameters, composes or copies equivalent effective source clocks, and samples preserved source values without recompilation or clearing attribution

#### Scenario: Compile one replacement on its fresh local clock
- **WHEN** a preset replaces one property on an item with retained typed and legacy animation clocks
- **THEN** only the replaced property receives newly compiled item-local primitives and source attribution with no retained clock, while unrelated typed channels and legacy keys retain their exact clocks and records
