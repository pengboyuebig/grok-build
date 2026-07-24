/**
 * @author glkj_pj <glkj@glkj.com>
 * @date 2026-07-24
 */

import { act, fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { App } from './App';

const bridge = vi.hoisted(() => ({
  getCommandCatalog: vi.fn().mockResolvedValue({ commands: [] }),
  launchTerminalSession: vi.fn(),
  listenToChatEvents: vi.fn().mockResolvedValue(() => undefined),
  sendMessage: vi.fn(),
}));

vi.mock('./lib/bridge', async (importOriginal) => ({
  ...(await importOriginal<typeof import('./lib/bridge')>()),
  getCommandCatalog: bridge.getCommandCatalog,
  launchTerminalSession: bridge.launchTerminalSession,
  listenToChatEvents: bridge.listenToChatEvents,
  sendMessage: bridge.sendMessage,
}));

describe('App', () => {
  it('hands selected context to the terminal launcher', async () => {
    render(<App />);

    await act(async () => {
      await Promise.resolve();
    });
    await act(async () => {
      fireEvent.click(screen.getByRole('button', { name: '打开终端会话' }));
      await Promise.resolve();
    });

    expect(bridge.launchTerminalSession).toHaveBeenCalledWith({
      cwd: expect.any(String),
      effort: 'medium',
      model: 'grok-build',
      permissionMode: 'ask',
    });
  });
});
