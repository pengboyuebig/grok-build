import { describe, expect, it } from "vitest";

import { parseAgentEvent } from "./protocol";

describe("parseAgentEvent", () => {
  it("classifies an ACP text delta notification", () => {
    expect(
      parseAgentEvent({
        method: "session/update",
        params: {
          sessionId: "session-1",
          update: { sessionUpdate: "agent_message_chunk", content: { text: "Hello" } }
        }
      })
    ).toEqual({ kind: "text-delta", sessionId: "session-1", text: "Hello" });
  });

  it("rejects an event without a method", () => {
    expect(() => parseAgentEvent({ params: {} })).toThrow("ACP event is missing a method");
  });
});
