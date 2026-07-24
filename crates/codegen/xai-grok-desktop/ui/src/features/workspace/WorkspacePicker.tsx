/**
 * @author glkj_pj <glkj@glkj.com>
 * @date 2026-07-24
 */

type WorkspacePickerProps = {
  cwd: string;
  onChange: (cwd: string) => void;
};

export function WorkspacePicker({ cwd, onChange }: WorkspacePickerProps) {
  return (
    <label className="block text-xs text-slate-400" htmlFor="workspace-path">
      工作目录
      <input
        className="mt-1 w-full rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm text-slate-100"
        id="workspace-path"
        onChange={(event) => onChange(event.target.value)}
        value={cwd}
      />
    </label>
  );
}
