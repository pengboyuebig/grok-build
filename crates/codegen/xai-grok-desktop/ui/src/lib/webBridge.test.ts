/**
 * @author glkj_pj <glkj@glkj.com>
 * @date 2026-07-26
 */

import { afterEach, describe, expect, it, vi } from 'vitest';

import { createWebBridge } from './webBridge';

describe('createWebBridge', () => {
  afterEach(() => vi.restoreAllMocks());

  it('moves the URL token into memory and sends it with HTTP requests', async () => {
    window.history.replaceState(null, '', '/#token=test-token');
    const fetchMock = vi.spyOn(window, 'fetch').mockResolvedValue(new Response(JSON.stringify({ commands: [] })));
    const bridge = createWebBridge();

    await bridge.getCommandCatalog();

    const headers = fetchMock.mock.calls[0]?.[1]?.headers as Headers;
    expect(headers.get('X-Grok-Local-Token')).toBe('test-token');
    expect(window.location.hash).toBe('');
  });
});
