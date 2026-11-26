//! ZeroClaw API layer — trait definitions and shared types.

pub mod a2a_wire;
pub mod agent;
pub mod attribution;
pub mod channel;
pub mod conversation_binding;
pub mod elicitation;
pub mod grants;
pub mod hook;
pub mod ingress;
pub mod jsonrpc;
pub mod lifecycle;
pub mod media;
pub mod memory_traits;
pub mod model_provider;
pub mod observability_traits;
pub mod peripherals_traits;
pub mod plan;
pub mod platform;
pub mod plugin;
pub mod plugin_egress;
pub mod plugin_key;
pub mod principal;
pub mod runtime_status;
pub mod runtime_traits;
pub mod schema;
pub mod session_keys;
pub mod tool;
pub mod tool_carrier;
pub mod turn_stop;
pub mod vad;
pub mod webhook;

/// Who a channel turn is answering; see [`TOOL_LOOP_CALLER`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TurnCaller {
    /// Channel name without the alias, e.g. `msteams`.
    pub channel: String,
    /// The channel's sender id (for Teams, the Entra object id).
    pub sender: String,
}

/// Per-turn attribution forwarded as `X-Agent-*` HTTP headers to the platform
/// AI model gateway; see [`TOOL_LOOP_TURN_CONTEXT`].
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TurnGatewayContext {
    pub conversation_id: String,
    pub turn_id: String,
    pub purpose: String,
    pub channel: String,
}

/// Read the active [`TurnGatewayContext`], falling back to `TOOL_LOOP_SESSION_KEY`
/// / `TOOL_LOOP_CALLER` when a caller only scoped session/caller task-locals.
pub fn current_turn_gateway_context() -> Option<TurnGatewayContext> {
    if let Ok(Some(ctx)) = TOOL_LOOP_TURN_CONTEXT.try_with(|c| c.clone()) {
        return Some(ctx);
    }
    let session = TOOL_LOOP_SESSION_KEY
        .try_with(|s| s.clone())
        .ok()
        .flatten()?;
    let channel = TOOL_LOOP_CALLER
        .try_with(|c| c.as_ref().map(|tc| tc.channel.clone()))
        .ok()
        .flatten()
        .unwrap_or_default();
    Some(TurnGatewayContext {
        conversation_id: session,
        turn_id: String::new(),
        purpose: String::new(),
        channel,
    })
}

tokio::task_local! {
    /// Current thread/sender ID for per-sender rate limiting.
    /// Set by the agent loop, read by SecurityPolicy.
    pub static TOOL_LOOP_THREAD_ID: Option<String>;

    /// Override for tool choice mode, set by the agent loop.
    /// Read by model_providers that support native tool calling.
    pub static TOOL_CHOICE_OVERRIDE: Option<String>;

    /// Session key for the currently active session.
    /// Scoped by gateway and channel turns, read by SessionsCurrentTool.
    pub static TOOL_LOOP_SESSION_KEY: Option<String>;

    /// The conversation the current turn is serving, when its surface
    /// supports conversation binding. Scoped beside `TOOL_LOOP_SESSION_KEY`
    /// by that surface; read by tools that record a binding.
    pub static TOOL_LOOP_ACTIVE_CONVERSATION:
        Option<crate::conversation_binding::ActiveConversation>;

    /// Channel and sender of the inbound message a channel turn answers.
    /// Scoped beside `TOOL_LOOP_SESSION_KEY` by the channel orchestrator;
    /// forwarded to MCP servers that opt in with `forward_context`.
    pub static TOOL_LOOP_CALLER: Option<TurnCaller>;

    /// Conversation, turn, purpose (`chat` / `memory` / `draft` / `classify`),
    /// and channel metadata forwarded to the platform AI model gateway.
    pub static TOOL_LOOP_TURN_CONTEXT: Option<TurnGatewayContext>;

    /// Native extended thinking parameters, set by the outer orchestration
    /// functions and read by `run_tool_call_loop` when building `ChatRequest`.
    pub static NATIVE_THINKING_OVERRIDE: Option<crate::model_provider::NativeThinkingParams>;
}
