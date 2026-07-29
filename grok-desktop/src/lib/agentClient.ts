import { parseAgentEvent, type AgentEvent } from "./protocol";

export interface RuntimeConnection {
  wsUrl: string;
  workspacePath: string;
}

export class AgentClient {
  private socket?: WebSocket;
  private requestId = 0;
  private pending = new Map<number, { resolve: (value: unknown) => void; reject: (reason: Error) => void }>();

  connect(connection: RuntimeConnection, onEvent: (event: AgentEvent) => void, onError: (message: string) => void): Promise<void> {
    return new Promise((resolve, reject) => {
      const socket = new WebSocket(connection.wsUrl);
      this.socket = socket;
      socket.onopen = () => resolve();
      socket.onerror = () => reject(new Error("无法连接本地智能体服务。"));
      socket.onclose = () => onError("本地智能体服务已断开。");
      socket.onmessage = (message) => {
        try {
          const payload: unknown = JSON.parse(String(message.data));
          const response = payload as { id?: unknown; error?: { message?: unknown }; result?: unknown };
          if (typeof response.id === "number" && this.pending.has(response.id)) {
            const request = this.pending.get(response.id)!;
            this.pending.delete(response.id);
            if (response.error) request.reject(new Error(typeof response.error.message === "string" ? response.error.message : "智能体请求失败。"));
            else request.resolve(response.result);
            return;
          }
          onEvent(parseAgentEvent(payload));
        } catch (error) {
          onError(error instanceof Error ? error.message : "无法读取智能体事件。");
        }
      };
    });
  }

  async initialize(workspacePath: string): Promise<string> {
    await this.request("initialize", {
      protocolVersion: 1,
      clientInfo: { name: "grok-desktop", version: "0.1.0" },
      capabilities: {},
      _meta: { clientVersion: "0.1.0", clientTerminal: false }
    });
    const session = await this.request("session/new", { cwd: workspacePath, mcpServers: [] }) as { sessionId?: unknown };
    if (typeof session.sessionId !== "string") throw new Error("智能体没有返回会话 ID。");
    return session.sessionId;
  }

  prompt(sessionId: string, text: string): void {
    void this.request("session/prompt", { sessionId, prompt: [{ type: "text", text }] });
  }

  cancel(sessionId: string): void {
    void this.request("session/cancel", { sessionId });
  }

  disconnect(): void {
    this.socket?.close();
    this.socket = undefined;
    for (const request of this.pending.values()) request.reject(new Error("智能体连接已关闭。"));
    this.pending.clear();
  }

  private request(method: string, params: unknown): Promise<unknown> {
    if (this.socket?.readyState !== WebSocket.OPEN) throw new Error("智能体尚未连接。");
    const id = ++this.requestId;
    return new Promise((resolve, reject) => {
      this.pending.set(id, { resolve, reject });
      this.socket!.send(JSON.stringify({ jsonrpc: "2.0", id, method, params }));
    });
  }
}
