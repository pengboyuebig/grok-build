/**
 * @author glkj_pj <glkj@glkj.com>
 * @date 2026-07-24
 */

import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { z } from 'zod';

const commandSchema = z.object({
  slash: z.string().regex(/^\/[a-z0-9-]+$/),
  kind: z.enum(['action', 'form', 'prompt_dispatch']),
  requires_confirmation: z.boolean(),
  can_spawn_process: z.literal(false),
  arguments: z.array(
    z.object({
      name: z.string().min(1),
      required: z.boolean(),
    }),
  ),
});

const catalogSchema = z.object({
  commands: z.array(commandSchema),
});

export type CommandCatalog = z.infer<typeof catalogSchema>;

const chatEventSchema = z.object({
  kind: z.enum(['assistant_delta', 'assistant_final', 'error']),
  text: z.string().optional(),
});

export type ChatEvent = z.infer<typeof chatEventSchema>;

export function validateCatalog(input: unknown): CommandCatalog {
  return catalogSchema.parse(input);
}

export async function getCommandCatalog(): Promise<CommandCatalog> {
  return validateCatalog(await invoke<unknown>('get_command_catalog'));
}

export async function sendMessage(sessionId: string, message: string): Promise<void> {
  await invoke('send_message', { sessionId, message });
}

export type TerminalLaunchRequest = {
  cwd: string;
  model: string;
  effort: 'low' | 'medium' | 'high' | 'xhigh';
  permissionMode: 'ask' | 'auto' | 'always_approve';
};

export async function launchTerminalSession(request: TerminalLaunchRequest): Promise<void> {
  await invoke('launch_terminal_session', { request });
}

export async function listenToChatEvents(handler: (event: ChatEvent) => void): Promise<() => void> {
  return listen<unknown>('chat:event', (event) => handler(chatEventSchema.parse(event.payload)));
}
