//! What a stream-json line from claude means for the chat. Only `type`, `subtype`, `event.type`,
//! `event.delta.type/text`, `message.content[]`, `parent_tool_use_id`, `session_id`, `is_error` and
//! `result` are read; a subagent's lines (`parent_tool_use_id` set) are left out.
use crate::json::Value;

pub enum Out {
    Text(String),        // answer text as it is written (text_delta); thinking is left out
    Tool,                // a tool call: the answer text before it was narration
    End(Option<String>), // a turn's result: its error if it failed
    Session(String),     // the conversation's id, when it changes
    Exit(String),        // claude ended: its last line on stderr
}

pub fn outs(v: &Value, session: &mut String) -> Vec<Out> {
    let mut outs = Vec::new();
    if let Some(id) = v.get("session_id").str().filter(|id| id != session) {
        *session = id.into();
        outs.push(Out::Session(id.into()));
    }
    if !matches!(v.get("parent_tool_use_id"), Value::Null) {
        return outs;
    }
    let (event, delta) = (v.get("event"), v.get("event").get("delta"));
    match v.get("type").str() {
        Some("stream_event") if event.get("type").str() == Some("content_block_delta")
                                && delta.get("type").str() == Some("text_delta") => {
            outs.extend(delta.get("text").str().map(|t| Out::Text(t.into())));
        }
        Some("assistant") => {
            if let Value::Arr(blocks) = v.get("message").get("content") {
                if blocks.iter().any(|b| b.get("type").str() == Some("tool_use")) {
                    outs.push(Out::Tool);
                }
            }
        }
        Some("result") => {
            let failed = matches!(v.get("is_error"), Value::Bool(true));
            let error = v.get("result").str().filter(|r| !r.is_empty()).or(v.get("subtype").str()).unwrap_or("failed");
            outs.push(Out::End(failed.then(|| error.into())));
        }
        _ => {}
    }
    outs
}
