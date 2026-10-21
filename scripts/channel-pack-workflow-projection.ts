/** Test-only inverse for CP1's single approved additive native coverage line. */
export const CHANNEL_WORKFLOW_LINE = '          OPENCUT_HEADLESS_PATH="$GITHUB_WORKSPACE/target/debug/opencut-headless" bun run --cwd apps/agent-bridge tests/channel-packs-workflow.ts\n';
const BUILD = "          cargo build -p opencut-headless\n";
const MATTE = "          bun run --cwd apps/agent-bridge test:unit --no-file-parallelism tests/track-matte-native.test.ts\n";

export const restoreChannelWorkflowPredecessor = (source: string): string => {
  if (source.split(CHANNEL_WORKFLOW_LINE).length !== 2 || !source.includes(`${BUILD}${CHANNEL_WORKFLOW_LINE}${MATTE}`)) {
    throw new Error("Channel native coverage must occur exactly once at its approved build anchor");
  }
  return source.replace(`${BUILD}${CHANNEL_WORKFLOW_LINE}${MATTE}`, `${BUILD}${MATTE}`);
};
