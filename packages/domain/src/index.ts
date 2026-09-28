export type ClientPlatform = 'linux' | 'windows' | 'macos' | 'android' | 'ios';
export const CLIENT_PLATFORMS: readonly ClientPlatform[] = ['linux', 'windows', 'macos', 'android', 'ios'];
export const SCAFFOLD_NOTICE = 'Scaffold only — agent execution, remote access and approvals are not implemented.';
export type AttemptState = 'queued' | 'provisioning' | 'running' | 'waiting_for_input' | 'waiting_for_approval' | 'validating' | 'review_ready' | 'completed' | 'failed' | 'cancelled' | 'interrupted' | 'outcome_unknown';
