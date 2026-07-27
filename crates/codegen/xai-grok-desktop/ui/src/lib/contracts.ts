/**
 * @author glkj_pj <glkj@glkj.com>
 * @date 2026-07-27
 */

import { z } from 'zod';

export const commandSchema = z.object({
  slash: z.string().regex(/^\/[a-z0-9-]+$/),
  kind: z.enum(['action', 'form', 'prompt_dispatch']),
  requires_confirmation: z.boolean(),
  can_spawn_process: z.literal(false),
  arguments: z.array(z.object({ name: z.string().min(1), required: z.boolean() })),
});

const catalogSchema = z.object({ commands: z.array(commandSchema) });
export type CommandCatalog = z.infer<typeof catalogSchema>;

export const chatEventSchema = z.object({
  kind: z.enum(['assistant_delta', 'assistant_final', 'approval_requested', 'error']),
  approval_id: z.string().min(1).optional(),
  text: z.string().optional(),
});
export type ChatEvent = z.infer<typeof chatEventSchema>;

export type TerminalLaunchRequest = {
  cwd: string;
  model: string;
  effort: 'low' | 'medium' | 'high' | 'xhigh';
  permissionMode: 'ask' | 'auto' | 'always_approve';
};

export function validateCatalog(input: unknown): CommandCatalog {
  return catalogSchema.parse(input);
}
