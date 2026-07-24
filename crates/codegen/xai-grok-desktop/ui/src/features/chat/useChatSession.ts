/**
 * @author glkj_pj <glkj@glkj.com>
 * @date 2026-07-24
 */

import { useEffect, useState } from 'react';

import { listenToChatEvents, sendMessage } from '../../lib/bridge';
import type { ChatMessage } from './types';

export function useChatSession(sessionId: string) {
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [error, setError] = useState<string>();

  useEffect(() => {
    let active = true;
    let unlisten: (() => void) | undefined;

    void listenToChatEvents((event) => {
      if (!active) {
        return;
      }
      if (event.kind === 'error') {
        setError(event.text ?? '对话连接发生错误');
        return;
      }
      const eventText = event.text;
      if (eventText) {
        setMessages((current) => {
          const last = current.at(-1);
          if (event.kind === 'assistant_delta' && last?.role === 'assistant') {
            return [...current.slice(0, -1), { ...last, text: `${last.text}${eventText}` }];
          }
          return [...current, { id: crypto.randomUUID(), role: 'assistant', text: eventText }];
        });
      }
    }).then((cleanup) => {
      unlisten = cleanup;
    });

    return () => {
      active = false;
      unlisten?.();
    };
  }, []);

  async function submit(message: string) {
    const text = message.trim();
    if (!text) {
      return;
    }
    setError(undefined);
    setMessages((current) => [...current, { id: crypto.randomUUID(), role: 'user', text }]);
    try {
      await sendMessage(sessionId, text);
    } catch {
      setError('发送消息失败，请检查会话状态后重试');
    }
  }

  return { error, messages, submit };
}
