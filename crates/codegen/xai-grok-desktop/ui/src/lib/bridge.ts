/**
 * @author glkj_pj <glkj@glkj.com>
 * @date 2026-07-24
 */

import { invoke } from '@tauri-apps/api/core';
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

export function validateCatalog(input: unknown): CommandCatalog {
  return catalogSchema.parse(input);
}

export async function getCommandCatalog(): Promise<CommandCatalog> {
  return validateCatalog(await invoke<unknown>('get_command_catalog'));
}
