/**
 * @author glkj_pj <glkj@glkj.com>
 * @date 2026-07-24
 */

import { FormEvent, useState } from 'react';

import { useChatSession } from './useChatSession';

type ChatWorkspaceProps = {
  sessionId: string;
};

export function ChatWorkspace({ sessionId }: ChatWorkspaceProps) {
  const [draft, setDraft] = useState('');
  const { error, messages, submit } = useChatSession(sessionId);

  function handleSubmit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const message = draft;
    setDraft('');
    void submit(message);
  }

  return (
    <section className="flex min-h-0 flex-1 flex-col bg-slate-950 text-slate-100">
      <div aria-live="polite" className="flex-1 space-y-4 overflow-y-auto p-6">
        {messages.map((message) => (
          <article
            className={
              message.role === 'user'
                ? 'ml-auto max-w-2xl rounded-2xl bg-cyan-600 px-4 py-3 text-sm text-white'
                : 'max-w-2xl rounded-2xl border border-slate-700 bg-slate-900 px-4 py-3 text-sm text-slate-100'
            }
            key={message.id}
          >
            {message.text}
          </article>
        ))}
        {error ? <p className="rounded-lg bg-red-950 px-3 py-2 text-sm text-red-200">{error}</p> : null}
      </div>
      <form className="border-t border-slate-800 p-4" onSubmit={handleSubmit}>
        <label className="sr-only" htmlFor="chat-message">
          消息输入
        </label>
        <div className="flex gap-3">
          <textarea
            className="min-h-12 flex-1 resize-none rounded-xl border border-slate-700 bg-slate-900 px-3 py-2 text-sm text-slate-100 outline-none focus:border-cyan-400"
            id="chat-message"
            onChange={(event) => setDraft(event.target.value)}
            placeholder="描述要完成的代码任务"
            value={draft}
          />
          <button className="rounded-xl bg-cyan-500 px-4 py-2 text-sm font-semibold text-slate-950" type="submit">
            发送
          </button>
        </div>
      </form>
    </section>
  );
}
