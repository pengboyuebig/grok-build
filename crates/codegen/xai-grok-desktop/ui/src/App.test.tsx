/**
 * @author glkj_pj <glkj@glkj.com>
 * @date 2026-07-24
 */

import { act, fireEvent, render, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';

import { App } from './App';

const bridge = vi.hoisted(() => ({
  getCommandCatalog: vi.fn(),
  launchTerminalSession: vi.fn(),
  listenToChatEvents: vi.fn(),
  respondToApproval: vi.fn(),
  sendMessage: vi.fn(),
  startSession: vi.fn(),
}));

vi.mock('./lib/bridge', async (importOriginal) => ({
  ...(await importOriginal<typeof import('./lib/bridge')>()),
  getCommandCatalog: bridge.getCommandCatalog,
  launchTerminalSession: bridge.launchTerminalSession,
  listenToChatEvents: bridge.listenToChatEvents,
  respondToApproval: bridge.respondToApproval,
  sendMessage: bridge.sendMessage,
  startSession: bridge.startSession,
}));

describe('App', () => {
  beforeEach(() => {
    vi.clearAllMocks();
    bridge.getCommandCatalog.mockResolvedValue({ commands: [] });
    bridge.listenToChatEvents.mockResolvedValue(() => undefined);
    bridge.startSession.mockResolvedValue('acp-session-1');
  });

  it('hands selected context to the terminal launcher', async () => {
    render(<App />);

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

  it('creates the agent session with the selected workspace', async () => {
    render(<App />);

    await waitFor(() => expect(bridge.startSession).toHaveBeenCalledWith('.'));
  });

  it('shows an approval request and sends the explicit response', async () => {
    render(<App />);

    await waitFor(() => expect(bridge.listenToChatEvents).toHaveBeenCalledTimes(2));
    await act(async () => {
      const event = {
        approval_id: 'a1',
        kind: 'approval_requested',
        text: 'Read the project files',
      };
      bridge.listenToChatEvents.mock.calls.forEach(([handler]) => handler(event));
    });
    await act(async () => {
      fireEvent.click(screen.getByRole('button', { name: '允许' }));
      await Promise.resolve();
    });

    expect(bridge.respondToApproval).toHaveBeenCalledWith('a1', true);
  });

  it('dispatches a menu command to the created chat session', async () => {
    bridge.getCommandCatalog.mockResolvedValueOnce({
      commands: [{ arguments: [], can_spawn_process: false, kind: 'action', requires_confirmation: false, slash: '/new' }],
    });
    render(<App />);

    await waitFor(() => expect(bridge.startSession).toHaveBeenCalledOnce());
    fireEvent.click(await screen.findByRole('button', { name: '/new' }));

    await waitFor(() => expect(bridge.sendMessage).toHaveBeenCalledWith('acp-session-1', '/new'));
  });
});
