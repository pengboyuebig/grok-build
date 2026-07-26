/**
 * @author glkj_pj <glkj@glkj.com>
 * @date 2026-07-24
 */

import { FormEvent, useState } from 'react';

import type { CommandCatalog } from '../../lib/bridge';

type CommandMenuProps = {
  commands: CommandCatalog['commands'];
  onDispatch: (slash: string, value?: string) => void;
};

const argumentLabels: Record<string, string> = {
  session_name: '会话名称',
  model: '模型名称',
  effort: '推理强度',
};

export function CommandMenu({ commands, onDispatch }: CommandMenuProps) {
  const [selectedSlash, setSelectedSlash] = useState<string>();
  const [argument, setArgument] = useState('');
  const selected = commands.find((command) => command.slash === selectedSlash);

  function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!selected) {
      return;
    }
    onDispatch(selected.slash, argument.trim() || undefined);
    setArgument('');
    setSelectedSlash(undefined);
  }

  return (
    <nav aria-label="命令菜单" className="space-y-2">
      {commands.map((command) => (
        <button
          className="w-full rounded-lg px-3 py-2 text-left text-sm text-slate-200 hover:bg-slate-800"
          key={command.slash}
          onClick={() => {
            if (command.kind === 'form') {
              setSelectedSlash(command.slash);
              return;
            }
            onDispatch(command.slash);
          }}
          type="button"
        >
          {command.slash}
        </button>
      ))}
      {selected ? (
        <form className="space-y-2 rounded-xl border border-slate-700 p-3" onSubmit={submit}>
          <label className="block text-sm text-slate-200" htmlFor="command-argument">
            {argumentLabels[selected.arguments[0]?.name] ?? '命令参数'}
          </label>
          <input
            className="w-full rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm text-slate-100"
            id="command-argument"
            onChange={(event) => setArgument(event.target.value)}
            required={selected.arguments[0]?.required}
            value={argument}
          />
          <button className="rounded-lg bg-cyan-500 px-3 py-2 text-sm font-semibold text-slate-950" type="submit">
            执行
          </button>
        </form>
      ) : null}
    </nav>
  );
}
