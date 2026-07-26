/**
 * @author glkj_pj <glkj@glkj.com>
 * @date 2026-07-24
 */

type ActivityItem = {
  id: string;
  label: string;
  detail: string;
};

type ActivityPanelProps = {
  items: ActivityItem[];
};

export function ActivityPanel({ items }: ActivityPanelProps) {
  return (
    <aside aria-label="运行活动" className="space-y-3 border-l border-slate-800 bg-slate-950 p-4">
      <h2 className="text-sm font-semibold text-slate-100">运行活动</h2>
      {items.map((item) => (
        <article className="rounded-lg border border-slate-800 bg-slate-900 p-3" key={item.id}>
          <p className="text-sm font-medium text-slate-100">{item.label}</p>
          <p className="mt-1 break-words text-xs text-slate-400">{item.detail}</p>
        </article>
      ))}
    </aside>
  );
}
