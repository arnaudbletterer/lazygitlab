import json
import os

log_path = r"C:\Users\abletterer\Projects\lazygitlab\.system_generated\logs\transcript.jsonl"
if not os.path.exists(log_path):
    log_path = r"C:\Users\abletterer\.gemini\antigravity-cli\brain\518f1e1f-999c-4d3e-8539-0c186dea6b5e\.system_generated\logs\transcript.jsonl"

with open(log_path, "r", encoding="utf-8") as f:
    for line in f:
        try:
            data = json.loads(line)
            idx = data.get("step_index", 0)
            if 330 <= idx <= 740:
                src = data.get("source", "")
                tp = data.get("type", "")
                content = data.get("content", "")
                if content:
                    content_summary = content.replace("\n", " ")[:200]
                else:
                    content_summary = ""
                tool_calls = data.get("tool_calls", [])
                tc_summary = ""
                if tool_calls:
                    tc_summary = "Tools: " + ", ".join([tc.get("name", "") for tc in tool_calls])
                print(f"[{idx}] {src} | {tp} | {content_summary} | {tc_summary}")
        except Exception as e:
            print("Error parsing line:", e)
