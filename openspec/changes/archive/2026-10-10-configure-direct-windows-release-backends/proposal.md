## Why

Exact d568f3d7 Windows native release failed strict warm final-encoder reuse; configured FFmpeg/FFprobe paths point to Chocolatey/bin forwarding executables. The existing core identity intentionally refuses arbitrary forwarding wrappers. Configure direct installed FFmpeg-family executables rather than changing that production guard or allowing a Windows fallback.

## What Changes

Add private Windows release setup selecting one direct native sibling FFmpeg/FFprobe pair below the installed Chocolatey ffmpeg/tools package tree. Reject absent/ambiguous/escaped/non-native/forwarding candidates and publish its directory to GITHUB_PATH before the unchanged existing dependency setup. Check real -version commands through that setup and actual native release proof. Add hermetic negative controls and exact step/order policy controls. Keep package/version pin7.1.1, all eligible warm reuse assertions and original cache trace exact.

## Capabilities

### Modified Capabilities
- `motion-release-gates`: Require direct installed Windows backend setup before complete native verification.

## Impact

Private Python helper/tests, only new79 workflow/policy and its reproduction documentation. Existing78 shared setup/workflow, original required workflows, renderer/domain/public/provider/schema behavior, frozen fixtures and #15 remain unchanged. Standing issue-scoped delegation approves this bounded correction before implementation.
