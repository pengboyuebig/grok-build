/**
 * @author glkj_pj <glkj@glkj.com>
 * @date 2026-07-24
 */

import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { createWebBridge } from './webBridge';
import { chatEventSchema, validateCatalog, type ChatEvent, type CommandCatalog, type TerminalLaunchRequest } from './contracts';

export { validateCatalog, type ChatEvent, type CommandCatalog, type TerminalLaunchRequest } from './contracts';

const isTauri = '__TAURI_INTERNALS__' in window;
let webBridge: ReturnType<typeof createWebBridge> | undefined;

function browserBridge() {
  if (isTauri) return undefined;
  webBridge ??= createWebBridge();
  return webBridge;
}

export async function getCommandCatalog(): Promise<CommandCatalog> {
  const web = browserBridge();
  if (web) return web.getCommandCatalog();
  return validateCatalog(await invoke<unknown>('get_command_catalog'));
}

export async function startSession(cwd: string): Promise<string> {
  const web = browserBridge();
  if (web) return web.startSession(cwd);
  return invoke<string>('start_session', { cwd });
}

export async function sendMessage(sessionId: string, message: string): Promise<void> {
  const web = browserBridge();
  if (web) return web.sendMessage(sessionId, message);
  await invoke('send_message', { sessionId, message });
}

export async function respondToApproval(approvalId: string, approved: boolean): Promise<void> {
  const web = browserBridge();
  if (web) return web.respondToApproval(approvalId, approved);
  await invoke('respond_to_approval', { approvalId, approved });
}

export async function launchTerminalSession(request: TerminalLaunchRequest): Promise<void> {
  const web = browserBridge();
  if (web) return web.launchTerminalSession(request);
  await invoke('launch_terminal_session', { request });
}

export async function listenToChatEvents(handler: (event: ChatEvent) => void): Promise<() => void> {
  const web = browserBridge();
  if (web) return web.listenToChatEvents(handler);
  return listen<unknown>('chat:event', (event) => handler(chatEventSchema.parse(event.payload)));
}
