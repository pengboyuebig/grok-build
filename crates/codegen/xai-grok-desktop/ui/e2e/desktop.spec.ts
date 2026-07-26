/**
 * @author glkj_pj <glkj@glkj.com>
 * @date 2026-07-25
 */

import { expect, test } from '@playwright/test';

test.describe('Grok Desktop', () => {
  test('sends a prompt, approves a tool, and launches the terminal session', async ({ page }) => {
    await page.addInitScript(() => {
      let nextCallbackId = 0;
      const callbacks = new Map<number, (event: { payload: unknown }) => void>();
      const listeners = new Map<number, number>();
      const host = window as typeof window & {
        __desktopTestCalls__: string[];
        __TAURI_EVENT_PLUGIN_INTERNALS__: {
          registerListener: () => void;
          unregisterListener: () => void;
        };
        __TAURI_INTERNALS__: {
          invoke: (command: string, args: Record<string, unknown>) => Promise<unknown>;
          transformCallback: (callback: (event: { payload: unknown }) => void) => number;
          unregisterCallback: (callbackId: number) => void;
        };
      };

      host.__TAURI_INTERNALS__ = {
        invoke: async (command, args) => {
          host.__desktopTestCalls__.push(command);
          if (command === 'get_command_catalog') {
            return { commands: [] };
          }
          if (command === 'start_session') {
            return 'acp-session-1';
          }
          if (command === 'plugin:event|listen') {
            const listenerId = nextCallbackId++;
            listeners.set(listenerId, args.handler as number);
            return listenerId;
          }
          if (command === 'plugin:event|unlisten') {
            listeners.delete(args.eventId as number);
            return undefined;
          }
          if (command === 'send_message') {
            queueMicrotask(() => {
              for (const callbackId of listeners.values()) {
                callbacks.get(callbackId)?.({
                  payload: {
                    approval_id: 'approval-1',
                    kind: 'approval_requested',
                    text: 'Read the failing test file',
                  },
                });
              }
            });
          }
          return undefined;
        },
        transformCallback: (callback) => {
          const callbackId = nextCallbackId++;
          callbacks.set(callbackId, callback);
          return callbackId;
        },
        unregisterCallback: (callbackId) => callbacks.delete(callbackId),
      };
      host.__TAURI_EVENT_PLUGIN_INTERNALS__ = {
        registerListener: () => undefined,
        unregisterListener: () => undefined,
      };
      host.__desktopTestCalls__ = [];
    });

    await page.goto('/');
    await page.locator('#chat-message').fill('Fix the failing tests');
    await page.locator('form button[type="submit"]').click();

    const approval = page.locator('section[aria-label] button').last();
    await approval.waitFor({ timeout: 30_000 });
    await approval.click();

    await page.locator('main > aside > button').click();
    await expect.poll(() => page.evaluate(() => window.__desktopTestCalls__)).toContain('launch_terminal_session');
  });
});
