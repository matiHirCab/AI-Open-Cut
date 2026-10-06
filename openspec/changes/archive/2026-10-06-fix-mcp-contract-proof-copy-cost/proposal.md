# Fix Windows MCP proof copying cost

## Why
Exact PR147 commit e995d18e fails Windows canonical deterministic MCP proof at its unchanged5000ms deadline (run37504015291, job112407950229, contracts.test.ts422). All9other leaf jobs pass. The proof must retain every assertion in one test. Repeated cached-reference fresh copying currently allocates Object.entries tuples, mapped tuples and Object.fromEntries objects at every nested node.

## What Changes
Optimize only the test-fixture fresh-copy helper with direct own-key copying. Preserve independent fresh trees, own property semantics (including __proto__), scalar/array/key order and all existing strict expansion/drift checks. Add focused nested copy/key/scalar isolation regression, leave full six-digest single test and5000ms deadline unchanged. No production/schema/catalog/pin/CI/task-timeout changes.

## Impact
Only apps/agent-bridge/tests/fixtures/mcp-surface-catalog.ts and the regression in contracts.test.ts; existing contract-governance bounded proof requirement. Required local full gates, independent review, accepted-delta lifecycle/protected gate and exact new-head all11CI remain required before completion.
