/**
 * @author glkj_pj <glkj@glkj.com>
 * @date 2026-07-24
 */

import { act, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { App } from './App';

const bridge = vi.hoisted(() => ({
  getCommandCatalog: vi.fn().mockResolvedValue({ commands: [] }),
  launchTerminalSession: vi.fn(),
  listenToChatEvents: vi.fn().mockResolvedValue(() => undefined),
  respondToApproval: vi.fn(),
  sendMessage: vi.fn(),
}));

vi.mock('./lib/bridge', async (importOriginal) => ({
  ...(await importOriginal<typeof import('./lib/bridge')>()),
  getCommandCatalog: bridge.getCommandCatalog,
  launchTerminalSession: bridge.launchTerminalSession,
  listenToChatEvents: bridge.listenToChatEvents,
  respondToApproval: bridge.respondToApproval,
  sendMessage: bridge.sendMessage,
}));

describe('App', () => {
  beforeEach(() => {
    bridge.getCommandCatalog.mockClear();
    bridge.launchTerminalSession.mockClear();
    bridge.listenToChatEvents.mockClear();
    bridge.respondToApproval.mockClear();
    bridge.sendMessage.mockClear();
  });

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

  it('shows an approval request and sends the explicit response', async () => {
    render(<App />);

    await waitFor(() => expect(bridge.listenToChatEvents.mock.calls.length).toBeGreaterThanOrEqual(2));
    await act(async () => {
      bridge.listenToChatEvents.mock.calls.at(-1)?.[0]({
        approval_id: 'a1',
        kind: 'approval_requested',
        text: '读取项目文件',
      });
    });
    await act(async () => {
      fireEvent.click(screen.getByRole('button', { name: '允许' }));
      await Promise.resolve();
    });

    expect(bridge.respondToApproval).toHaveBeenCalledWith('a1', true);
  });
});
