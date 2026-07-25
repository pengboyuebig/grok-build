/**
 * @author glkj_pj <glkj@glkj.com>
 * @date 2026-07-24
 */

import { useEffect, useState } from 'react';

import { ActivityPanel } from './features/activity/ActivityPanel';
import { ApprovalDialog } from './features/approvals/ApprovalDialog';
import { ChatWorkspace } from './features/chat/ChatWorkspace';
import { CommandMenu } from './features/commands/CommandMenu';
import { SessionList } from './features/sessions/SessionList';
import { WorkspacePicker } from './features/workspace/WorkspacePicker';
import {
  getCommandCatalog,
  launchTerminalSession,
  listenToChatEvents,
  respondToApproval,
  type CommandCatalog,
  type TerminalLaunchRequest,
} from './lib/bridge';

const DEFAULT_CWD = 'C:/work/demo';

export function App() {
  const [activeSessionId, setActiveSessionId] = useState('s1');
  const [catalog, setCatalog] = useState<CommandCatalog>({ commands: [] });
  const [cwd, setCwd] = useState(DEFAULT_CWD);
  const [model, setModel] = useState('grok-build');
  const [effort, setEffort] = useState<TerminalLaunchRequest['effort']>('medium');
  const [permissionMode, setPermissionMode] = useState<TerminalLaunchRequest['permissionMode']>('ask');
  const [activities, setActivities] = useState<Array<{ id: string; label: string; detail: string }>>([]);
  const [approval, setApproval] = useState<{ id: string; description: string }>();

  useEffect(() => {
    void getCommandCatalog().then(setCatalog).catch(() => setActivities([{ id: 'catalog-error', label: '菜单加载失败', detail: '请检查桌面服务是否运行。' }]));
  }, []);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    void listenToChatEvents((event) => {
      if (event.kind === 'approval_requested' && event.approval_id) {
        setApproval({ id: event.approval_id, description: event.text ?? '工具请求执行操作' });
      }
    }).then((cleanup) => {
      unlisten = cleanup;
    });
    return () => unlisten?.();
  }, []);

  async function handleApproval(approvalId: string, approved: boolean) {
    await respondToApproval(approvalId, approved);
    setApproval(undefined);
    setActivities((current) => [
      { id: crypto.randomUUID(), label: approved ? '工具操作已允许' : '工具操作已拒绝', detail: approvalId },
      ...current,
    ]);
  }

  async function openTerminalSession() {
    const request = { cwd, model, effort, permissionMode };
    await launchTerminalSession(request);
    setActivities((current) => [
      { id: crypto.randomUUID(), label: '终端会话已启动', detail: `${model} · ${cwd}` },
      ...current,
    ]);
  }

  return (
    <main className="grid min-h-screen grid-cols-[17rem_minmax(0,1fr)_18rem] bg-slate-950 text-slate-100">
      <aside className="space-y-6 border-r border-slate-800 bg-slate-950 p-4">
        <div>
          <p className="text-xs font-semibold uppercase tracking-[0.18em] text-cyan-400">Grok Desktop</p>
          <h1 className="mt-2 text-lg font-semibold">AI 编码工作台</h1>
        </div>
        <SessionList activeId={activeSessionId} onSelect={setActiveSessionId} sessions={[{ id: 's1', title: '当前会话' }]} />
        <WorkspacePicker cwd={cwd} onChange={setCwd} />
        <label className="block text-xs text-slate-400" htmlFor="model-select">
          模型
          <input
            className="mt-1 w-full rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm text-slate-100"
            id="model-select"
            onChange={(event) => setModel(event.target.value)}
            value={model}
          />
        </label>
        <label className="block text-xs text-slate-400" htmlFor="effort-select">
          推理强度
          <select
            className="mt-1 w-full rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm text-slate-100"
            id="effort-select"
            onChange={(event) => setEffort(event.target.value as TerminalLaunchRequest['effort'])}
            value={effort}
          >
            <option value="low">低</option>
            <option value="medium">中</option>
            <option value="high">高</option>
            <option value="xhigh">极高</option>
          </select>
        </label>
        <label className="block text-xs text-slate-400" htmlFor="permission-select">
          权限模式
          <select
            className="mt-1 w-full rounded-lg border border-slate-700 bg-slate-900 px-3 py-2 text-sm text-slate-100"
            id="permission-select"
            onChange={(event) => setPermissionMode(event.target.value as TerminalLaunchRequest['permissionMode'])}
            value={permissionMode}
          >
            <option value="ask">每次询问</option>
            <option value="auto">自动</option>
            <option value="always_approve">始终允许</option>
          </select>
        </label>
        <button className="w-full rounded-xl bg-cyan-500 px-4 py-3 text-sm font-semibold text-slate-950" onClick={() => void openTerminalSession()} type="button">
          打开终端会话
        </button>
        <CommandMenu commands={catalog.commands} onDispatch={(slash, value) => setActivities((current) => [{ id: crypto.randomUUID(), label: slash, detail: value ?? '已请求执行' }, ...current])} />
      </aside>
      <ChatWorkspace sessionId={activeSessionId} />
      <div className="flex min-h-0 flex-col">
        {approval ? <ApprovalDialog onRespond={(id, approved) => void handleApproval(id, approved)} request={approval} /> : null}
        <ActivityPanel items={activities} />
      </div>
    </main>
  );
}
