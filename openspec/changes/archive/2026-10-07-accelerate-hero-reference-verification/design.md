## Context
CI annotation112661339340 reports5000ms timeout for the complete scalar oracle test. Gaussian convolution repeats for identical masked-source rasters across baseline/reverse and feature witnesses.

## Decision
Define a pure-raster memoization decorator with functools.cache keyed by tuple(input). Evaluate the original unchanged function body on a fresh list; store tuple(output). Return fresh list(output) every invocation so subsequent tint mutation cannot alias a cached raster. Decorate gaussian and glow only; retain existing plate cache. Lifetime is one oracle process with the bounded canonical fixture. No fixture or production output is cache authority.

## Failure and Compatibility
All original scalar arithmetic order, byte hashes, exact expectations, mutation controls, WAV generation, refusal and CLI remain unchanged. Fresh-copy cache isolation is checked automatically inside the verifier and independently with mutated returned rasters: for gaussian/glow, snapshot a pure empty-raster result, mutate a returned copy, then require a fresh call to preserve the snapshot and the original input. All complete frozen-reference comparisons still execute. No timeout increases, suppressions, extra dependencies, production code or version changes.

## Verification
Independently approve before implementation; compare all9 original scalar bodies by AST; verify whole16plates/8counterfactual/WAV, cache isolation, required relevant checks and unchanged5s unit/default/native deadlines. Retain failed CI receipt. Independently review source and conformance; archive own modified requirement, rerun protected/strict, commit separately/push samePR and require exactnewhead all11SUCCESS.
