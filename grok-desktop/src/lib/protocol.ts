export type AgentEvent =
  | { kind: "text-delta"; sessionId: string; text: string }
  | { kind: "tool-activity"; sessionId: string; title: string; status: "running" | "completed" | "failed" }
  | { kind: "file-change"; sessionId: string; path: string; operation: "create" | "update" | "delete" }
  | { kind: "turn-complete"; sessionId: string }
  | { kind: "unknown"; method: string };

type JsonRecord = Record<string, unknown>;

function record(value: unknown): JsonRecord | undefined {
  return typeof value === "object" && value !== null && !Array.isArray(value)
    ? (value as JsonRecord)
    : undefined;
}

function stringField(value: unknown): string | undefined {
  return typeof value === "string" ? value : undefined;
}

export function parseAgentEvent(message: unknown): AgentEvent {
  const root = record(message);
  const method = stringField(root?.method);
  if (!method) throw new Error("ACP event is missing a method");

  const params = record(root?.params);
  const sessionId = stringField(params?.sessionId) ?? "unknown-session";
  if (method === "session/update") {
    const update = record(params?.update);
    const name = stringField(update?.sessionUpdate);
    const content = record(update?.content);
    const text = stringField(content?.text);
    if (name === "agent_message_chunk" && text) return { kind: "text-delta", sessionId, text };
    if (name === "tool_call") {
      return { kind: "tool-activity", sessionId, title: stringField(update?.title) ?? "Agent tool", status: "running" };
    }
  }
  if (method === "session/turn_complete") return { kind: "turn-complete", sessionId };
  return { kind: "unknown", method };
}
