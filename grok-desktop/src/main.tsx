import React, { useMemo, useState } from "react";
import { createRoot } from "react-dom/client";
import { invoke } from "@tauri-apps/api/core";
import { AgentClient, type RuntimeConnection } from "./lib/agentClient";
import type { AgentEvent } from "./lib/protocol";
import "./styles.css";

type RuntimeStatus = "stopped" | "starting" | "ready" | "failed";
type Message = { role: "user" | "agent"; text: string };

function App() {
  const client = useMemo(() => new AgentClient(), []);
  const [workspacePath, setWorkspacePath] = useState("");
  const [status, setStatus] = useState<RuntimeStatus>("stopped");
  const [prompt, setPrompt] = useState("");
  const [messages, setMessages] = useState<Message[]>([]);
  const [activity, setActivity] = useState<string[]>([]);
  const [error, setError] = useState("");
  const [sessionId, setSessionId] = useState("");

  const receive = (event: AgentEvent) => {
    if (event.kind === "text-delta") {
      setMessages((items) => {
        const last = items.at(-1);
        return last?.role === "agent"
          ? [...items.slice(0, -1), { role: "agent", text: last.text + event.text }]
          : [...items, { role: "agent", text: event.text }];
      });
    }
    if (event.kind === "tool-activity") setActivity((items) => [`${event.title} - ${event.status}`, ...items]);
    if (event.kind === "turn-complete") setActivity((items) => ["本轮任务已完成", ...items]);
  };

  const chooseWorkspace = async () => {
    const path = await invoke<string | null>("select_workspace");
    if (path) setWorkspacePath(path);
  };

  const start = async () => {
    setStatus("starting"); setError("");
    try {
      const connection = await invoke<RuntimeConnection>("start_agent", { workspacePath });
      await client.connect(connection, receive, (message) => { setStatus("failed"); setError(message); });
      const newSessionId = await client.initialize(workspacePath);
      setSessionId(newSessionId);
      setStatus("ready");
    } catch (cause) {
      setStatus("failed"); setError(cause instanceof Error ? cause.message : String(cause));
    }
  };

  const send = () => {
    const text = prompt.trim();
    if (!text || status !== "ready") return;
    setMessages((items) => [...items, { role: "user", text }]);
    setPrompt("");
    try { client.prompt(sessionId, text); } catch (cause) { setError(cause instanceof Error ? cause.message : String(cause)); }
  };

  const stop = async () => {
    client.disconnect();
    await invoke("stop_agent");
    setSessionId("");
    setStatus("stopped");
  };

  return <main className="shell">
    <aside className="rail">
      <div className="brand"><span>G</span><div>GROK<br /><small>DESKTOP</small></div></div>
      <div className="workspace"><label>工作目录</label><p>{workspacePath || "尚未选择项目"}</p><button onClick={chooseWorkspace}>选择文件夹</button></div>
      <div className="status"><i className={status} />{status === "ready" ? "智能体已连接" : status === "starting" ? "正在启动" : status === "failed" ? "需要处理" : "尚未启动"}</div>
      <div className="notice">本应用不提供登录。模型凭据读取自现有的 <code>.grok</code> 配置或环境变量。</div>
    </aside>
    <section className="conversation">
      <header><div><span className="eyebrow">LOCAL AGENT / ACP</span><h1>工作台</h1></div><div className="actions"><button className="outline" disabled={!workspacePath || status === "starting"} onClick={start}>启动智能体</button><button className="danger" disabled={status === "stopped"} onClick={stop}>结束服务</button></div></header>
      {error && <div className="error">{error}</div>}
      <div className="transcript">{messages.length === 0 ? <div className="empty"><strong>从一个项目开始</strong><span>选择工作目录，启动本地智能体，然后描述你想完成的任务。</span></div> : messages.map((message, index) => <article className={message.role} key={index}><label>{message.role === "user" ? "你" : "Grok"}</label><p>{message.text}</p></article>)}</div>
      <footer><textarea value={prompt} onChange={(event) => setPrompt(event.target.value)} onKeyDown={(event) => { if (event.ctrlKey && event.key === "Enter") { event.preventDefault(); send(); } }} placeholder="描述要完成的开发任务..." disabled={status !== "ready"} /><button onClick={send} disabled={status !== "ready" || !prompt.trim()}>发送 <kbd>Ctrl Enter</kbd></button></footer>
    </section>
    <aside className="activity"><header><span className="eyebrow">LIVE TRACE</span><h2>活动记录</h2></header>{activity.length === 0 ? <p className="muted">工具调用、任务状态和文件变更将在这里出现。</p> : <ol>{activity.map((entry, index) => <li key={index}>{entry}</li>)}</ol>}</aside>
  </main>;
}

createRoot(document.getElementById("root")!).render(<React.StrictMode><App /></React.StrictMode>);
