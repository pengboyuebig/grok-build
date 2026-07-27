/**
 * @author glkj_pj <glkj@glkj.com>
 * @date 2026-07-27
 */

import { expect, test } from '@playwright/test';

test.describe('Grok local browser console', () => {
  test('uses the local token for chat approvals and terminal launch', async ({ page }) => {
    await page.addInitScript(() => {
      const host = window as typeof window & { __webTestRequests__: Array<{ path: string; token: string | null }> };
      host.__webTestRequests__ = [];
      window.fetch = async (input, init) => {
        const path = String(input);
        const headers = new Headers(init?.headers);
        host.__webTestRequests__.push({ path, token: headers.get('X-Grok-Local-Token') });
        if (path === '/api/commands') return new Response(JSON.stringify({ commands: [] }));
        if (path === '/api/sessions') return new Response(JSON.stringify({ session_id: 'web-session-1' }));
        return new Response(null, { status: 204 });
      };

      class LocalWebSocket extends EventTarget {
        constructor(_url: string, _protocols: string[]) {
          super();
          queueMicrotask(() => this.dispatchEvent(new MessageEvent('message', {
            data: JSON.stringify({ kind: 'approval_requested', approval_id: 'web-approval-1', text: 'Read the test file' }),
          })));
        }

        close() {}
      }
      window.WebSocket = LocalWebSocket as unknown as typeof WebSocket;
    });

    await page.goto('/#token=web-test-token');
    await page.locator('#chat-message').fill('Fix the failing tests');
    await page.locator('form button[type="submit"]').click();

    const approval = page.locator('section[aria-label] button').last();
    await approval.waitFor({ timeout: 30_000 });
    await approval.click();
    await page.locator('main > aside > button').click();

    await expect.poll(() => page.evaluate(() => window.__webTestRequests__)).toEqual(expect.arrayContaining([
      expect.objectContaining({ path: '/api/sessions', token: 'web-test-token' }),
      expect.objectContaining({ path: '/api/sessions/web-session-1/messages', token: 'web-test-token' }),
      expect.objectContaining({ path: '/api/approvals/web-approval-1', token: 'web-test-token' }),
      expect.objectContaining({ path: '/api/terminal-sessions', token: 'web-test-token' }),
    ]));
    await expect(page).toHaveURL('http://127.0.0.1:4173/');
  });
});
