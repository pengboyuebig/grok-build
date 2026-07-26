/**
 * @author glkj_pj <glkj@glkj.com>
 * @date 2026-07-24
 */

type ApprovalRequest = {
  id: string;
  description: string;
};

type ApprovalDialogProps = {
  request: ApprovalRequest;
  onRespond: (id: string, approved: boolean) => void;
};

export function ApprovalDialog({ request, onRespond }: ApprovalDialogProps) {
  return (
    <section aria-label="工具权限确认" className="rounded-xl border border-amber-500/50 bg-amber-950/40 p-4">
      <h2 className="text-sm font-semibold text-amber-100">需要你的确认</h2>
      <p className="mt-2 text-sm text-amber-50">{request.description}</p>
      <div className="mt-4 flex gap-2">
        <button className="rounded-lg border border-slate-600 px-3 py-2 text-sm text-slate-100" onClick={() => onRespond(request.id, false)} type="button">
          拒绝
        </button>
        <button className="rounded-lg bg-amber-400 px-3 py-2 text-sm font-semibold text-slate-950" onClick={() => onRespond(request.id, true)} type="button">
          允许
        </button>
      </div>
    </section>
  );
}
