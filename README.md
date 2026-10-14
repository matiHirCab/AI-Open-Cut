Unverified issue191 recovery artifacts. Extract at a workspace root; paths are relative to /workspace. Start with issue191-parent-checkpoint.md and tracked docs/verification/illustrated-workflow/RECOVERY-CHECKPOINT.md on branch chore/issue-191-illustrated-workflow. Generated GUI executable, private Xauthority, caches and runtime sockets deliberately excluded. No feature work or tests are running in the old environment.

Reassemble: cat issue191-recovery-evidence.tar.gz.part1 issue191-recovery-evidence.tar.gz.part2 > issue191-recovery-evidence.tar.gz
Verify archive SHA256: 6cabbef6c475e8da5b18fb881db425f32da416c5de431d589deb7ad47ea8cd8f
