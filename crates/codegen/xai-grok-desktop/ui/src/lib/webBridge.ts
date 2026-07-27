/**
 * @author glkj_pj <glkj@glkj.com>
 * @date 2026-07-27
 */

import { chatEventSchema, validateCatalog, type ChatEvent, type CommandCatalog, type TerminalLaunchRequest } from './contracts';

export function createWebBridge() {
  const hash = new URLSearchParams(window.location.hash.slice(1));
  const token = hash.get('token');
  window.history.replaceState(null, '', `${window.location.pathname}${window.location.search}`);
  if (!token) {
    throw new Error('缺少本地访问令牌，请从 grok-web 输出的地址重新打开页面');
  }
  const localToken = token;

  async function request(path: string, init: RequestInit = {}): Promise<Response> {
    const headers = new Headers(init.headers);
    headers.set('X-Grok-Local-Token', localToken);
    if (init.body) headers.set('Content-Type', 'application/json');
    const response = await fetch(path, { ...init, headers });
    if (!response.ok) throw new Error(`本地服务请求失败 (${response.status})`);
    return response;
  }

  return {
    async getCommandCatalog(): Promise<CommandCatalog> {
      return validateCatalog(await (await request('/api/commands')).json());
    },
    async startSession(cwd: string): Promise<string> {
      const result = await (await request('/api/sessions', { method: 'POST', body: JSON.stringify({ cwd }) })).json() as { session_id: string };
      return result.session_id;
    },
    async sendMessage(sessionId: string, message: string): Promise<void> {
      await request(`/api/sessions/${encodeURIComponent(sessionId)}/messages`, { method: 'POST', body: JSON.stringify({ message }) });
    },
    async respondToApproval(approvalId: string, approved: boolean): Promise<void> {
      await request(`/api/approvals/${encodeURIComponent(approvalId)}`, { method: 'POST', body: JSON.stringify({ approved }) });
    },
    async launchTerminalSession(value: TerminalLaunchRequest): Promise<void> {
      await request('/api/terminal-sessions', { method: 'POST', body: JSON.stringify({ ...value, permission_mode: value.permissionMode }) });
    },
    async listenToChatEvents(handler: (event: ChatEvent) => void): Promise<() => void> {
      const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
      const socket = new WebSocket(`${protocol}//${window.location.host}/api/events`, [`grok-local.${localToken}`]);
      socket.addEventListener('message', (message) => handler(chatEventSchema.parse(JSON.parse(String(message.data)))));
      return () => socket.close();
    },
  };
}
