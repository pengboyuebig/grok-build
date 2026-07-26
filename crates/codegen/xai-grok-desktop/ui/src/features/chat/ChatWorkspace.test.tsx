/**
 * @author glkj_pj <glkj@glkj.com>
 * @date 2026-07-24
 */

import { act, fireEvent, render, screen } from '@testing-library/react';
import { describe, expect, it, vi } from 'vitest';

import { ChatWorkspace } from './ChatWorkspace';

const bridge = vi.hoisted(() => ({
  emit: undefined as undefined | ((event: unknown) => void),
  sendMessage: vi.fn(),
}));

vi.mock('../../lib/bridge', () => ({
  listenToChatEvents: (handler: (event: unknown) => void) => {
    bridge.emit = handler;
    return Promise.resolve(() => undefined);
  },
  sendMessage: bridge.sendMessage,
}));

describe('ChatWorkspace', () => {
  it('sends a user message and renders streamed assistant text', async () => {
    render(<ChatWorkspace sessionId="s1" />);

    fireEvent.change(screen.getByLabelText('消息输入'), {
      target: { value: '修复测试' },
    });
    fireEvent.click(screen.getByRole('button', { name: '发送' }));
    await act(async () => {
      bridge.emit?.({ kind: 'assistant_delta', text: '我来处理。' });
    });

    expect(bridge.sendMessage).toHaveBeenCalledWith('s1', '修复测试');
    expect(await screen.findByText('我来处理。')).toBeTruthy();
  });
});
