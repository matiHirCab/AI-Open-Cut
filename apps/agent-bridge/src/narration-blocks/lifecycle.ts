import type { HeadlessClient } from "../headless";
import type { JobRuntimeGuard } from "../job-runtime";
import type { SpeechApplicationService } from "../speech";
import type { NarrationBlocksWorkflow } from "./workflow";

interface SessionResources {
  guard?: Pick<JobRuntimeGuard, "release"> | undefined;
  headless?: Pick<HeadlessClient, "close"> | undefined;
  speech?: Pick<SpeechApplicationService, "close"> | undefined;
  workflow?: Pick<NarrationBlocksWorkflow, "close"> | undefined;
}
// Close in ownership order, attempting the transport even when the provider
// cannot confirm retirement. An uncertain cleanup keeps crash admission held.
export async function closeNarrationSession(resources: SessionResources) {
  const errors: unknown[] = [];
  let publicationUnknown = false;
  try {
    if (resources.workflow) {
      const closed = await resources.workflow.close();
      ({ publicationUnknown } = closed);
      errors.push(...(closed.cleanupErrors ?? []));
    } else {
      await resources.speech?.close();
    }
  } catch (error) {
    errors.push(error);
  }
  try {
    await resources.headless?.close();
  } catch (error) {
    errors.push(error);
  }
  try {
    resources.guard?.release(errors.length === 0);
  } catch (error) {
    errors.push(error);
  }
  return { cleanupConfirmed: errors.length === 0, errors, publicationUnknown };
}
