## Decisions

Replace both four-byte chunks_exact iterators with as_chunks::<4>().0.iter(). Existing prior equal byte lengths, decoded-frame/finite assertions and f32 little-endian/RMS semantics stay exact. Four-byte remainder handling remains identical because both methods exclude the remainder; existing decoder byte alignment assertions remain authoritative. Run actual default native proof, strict Clippy and formatting, then conformance/archive/protected/strict gates. Preserve failed cf54fece CI evidence and issue-scoped PR lineage. No warning suppression or weakened acceptance.
