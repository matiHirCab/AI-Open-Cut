# Authorized single-pair serial diagnostic — 2026-10-29

Parent reviewer explicitly authorized one serial baseline/current comparison with identical dependencies, no simultaneous checks, unchanged assertions/timeouts and no CI or repeated rounds. Baseline is detached worktree `/workspace/AI-Open-Cut-pr233-baseline` at2768cd00; changed tree remains `/workspace/AI-Open-Cut-pr233-inspection` at the same base plus approved local diff. Baseline node_modules links to changed tree node_modules; neither lockfile nor toolchain changed. Executed baseline first, then changed once. No code changes in this experiment.

Both files passed all44 cases under the original5-second guard. Baseline catalog3241.685ms, changed2280.096ms; baseline readiness ready=true148.536ms/false13.491ms, changed true31.448ms/false8.593ms. Earlier concurrent execution remains a real failure: catalog6410ms and readiness6490ms. These new runs do not erase it or establish a Windows repair.

One sequential pair is compatible with resource contention contributing to earlier local failures, but does not isolate causality: baseline always ran first, caches/JIT/load can differ, and no concurrent-run resource baseline was captured. Cgroup measurements include all container activity and host load averages are not test-only CPU utilization. Current-head Windows evidence remains mandatory. No additional offline failure profiling is needed under the conditional instruction because both serial runs passed. No new optimization or specification change was made. Broader checks, conformance and publication remain pending; this diagnostic alone is not whole-PR completion or authorization to bypass failed protected gates. Native diagnostic stays active.

Resource observations (4-CPU quota,16GiB memory maximum):

```json
[
  {
    "run": "baseline",
    "exit": 0,
    "wall_s": 15.042,
    "cgroupCPUSeconds": 18.401,
    "throttledSeconds": 0.0,
    "throttledPeriods": 0,
    "cgroupMemoryMiBMin": 14511.3,
    "cgroupMemoryMiBMax": 15443.7,
    "loadavgBefore": [
      0.08447265625,
      0.20751953125,
      0.09521484375
    ],
    "loadavgAfter": [
      0.2880859375,
      0.2470703125,
      0.10986328125
    ],
    "cases": [
      {
        "fullName": "canonical public contracts expands the approved additive MCP capability catalog deterministically",
        "duration": 3241.6845030000004,
        "status": "passed"
      },
      {
        "fullName": "canonical public contracts preserves readiness-gated linear composition in live MCP status (ready=true)",
        "duration": 148.53552000000127,
        "status": "passed"
      },
      {
        "fullName": "canonical public contracts preserves readiness-gated linear composition in live MCP status (ready=false)",
        "duration": 13.491269000000102,
        "status": "passed"
      }
    ]
  },
  {
    "run": "changed",
    "exit": 0,
    "wall_s": 9.045,
    "cgroupCPUSeconds": 11.348,
    "throttledSeconds": 0.0,
    "throttledPeriods": 0,
    "cgroupMemoryMiBMin": 14512.1,
    "cgroupMemoryMiBMax": 15398.2,
    "loadavgBefore": [
      0.2880859375,
      0.2470703125,
      0.10986328125
    ],
    "loadavgAfter": [
      0.39794921875,
      0.2724609375,
      0.11962890625
    ],
    "cases": [
      {
        "fullName": "canonical public contracts expands the approved additive MCP capability catalog deterministically",
        "duration": 2280.0958060000003,
        "status": "passed"
      },
      {
        "fullName": "canonical public contracts preserves readiness-gated linear composition in live MCP status (ready=true)",
        "duration": 31.447558999999274,
        "status": "passed"
      },
      {
        "fullName": "canonical public contracts preserves readiness-gated linear composition in live MCP status (ready=false)",
        "duration": 8.593134999999165,
        "status": "passed"
      }
    ]
  }
]
```

Full local artifacts (SHA256):
- `/tmp/opencut-pr233-serial-experiment.py`: `80f85ec6f342d20fd45dc2480a3ae148aeaaadf0c0699da1648d4ff6014f05af`
- `/tmp/opencut-pr233-serial-experiment.json`: `f38be4455113417db1d89cecfdb8c28da69a7f9be9f92d38ed09a212800a6aa7`
- `/tmp/opencut-pr233-serial-baseline.log`: `6798e8fc90e0dff801bfcb13ccc0d3df4157745d17a814d1679dd7736a35e89f`
- `/tmp/opencut-pr233-serial-baseline.json`: `e0fdf95eee677850b6f195ece109b7772ff2029cc01c89185b83b80667628261`
- `/tmp/opencut-pr233-serial-changed.log`: `eb699b3ebba38fc128d79f2c8e03afe8c25840f6b1a1945d27594d5bd01afadd`
- `/tmp/opencut-pr233-serial-changed.json`: `9b82f5217bdcc301542eab2ba99219f86d33ae1e8295a983728fd5722cdd035a`

Memory event deltas were zero in both runs (including max/oom/oom_kill). Memory.current includes charged cache as well as processes, and must not be reported as test RSS. Pressure files were unavailable to this snapshot collector.
