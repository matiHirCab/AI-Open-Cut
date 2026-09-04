## ADDED Requirements

### Requirement: Attributed Bun bootstrap regression evidence
Repository validation MUST exercise the hardened bootstrap with a malicious checkout-controlled `bunfig.toml` while every other required workflow, Moon, and proto source is valid. The regression MUST distinguish the canonical Bun-configuration preflight rejection from unrelated missing-source or process failures and MUST prove that the preload, Moon child, and policy attestation remain unreachable. Its independent real-Moon reproduction MUST isolate inherited parent Moon/proto metadata and stores and MUST have a bounded execution budget sufficient for nested startup on the protected Ubuntu runner.

#### Scenario: Reject the malicious Bun configuration for the intended reason
- **WHEN** the real hardened Bun invocation receives a valid reviewed workflow and otherwise-canonical Moon and proto boundary with only `bunfig.toml` altered to preload forgery code
- **THEN** it exits nonzero with the canonical Bun-configuration rejection, creates no preload sentinel, launches no Moon child, and writes no policy attestation
