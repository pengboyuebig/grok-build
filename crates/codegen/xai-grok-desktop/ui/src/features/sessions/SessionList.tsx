/**
 * @author glkj_pj <glkj@glkj.com>
 * @date 2026-07-24
 */

type SessionListProps = {
  activeId: string;
  sessions: Array<{ id: string; title: string }>;
  onSelect: (id: string) => void;
};

export function SessionList({ activeId, sessions, onSelect }: SessionListProps) {
  return (
    <section aria-label="会话列表" className="space-y-2">
      <h2 className="text-xs font-semibold uppercase tracking-wide text-slate-500">会话</h2>
      {sessions.map((session) => (
        <button
          className={
            session.id === activeId
              ? 'w-full rounded-lg bg-slate-800 px-3 py-2 text-left text-sm text-white'
              : 'w-full rounded-lg px-3 py-2 text-left text-sm text-slate-300 hover:bg-slate-900'
          }
          key={session.id}
          onClick={() => onSelect(session.id)}
          type="button"
        >
          {session.title}
        </button>
      ))}
    </section>
  );
}
