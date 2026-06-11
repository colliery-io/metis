//! Who is performing an operation.
//!
//! Every mutation records an [`Actor`] in the `events` audit trail. Auth (token
//! resolution) happens in the server; the service layer just carries and
//! records attribution — a human user, optionally acting through a named agent,
//! so history reads "dylan via claude-code".

/// The principal behind a request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Actor {
    /// The human user responsible for the action.
    pub user: uuid::Uuid,
    /// The agent acting on the user's behalf, if any (e.g. `"claude-code"`).
    pub agent: Option<String>,
}

impl Actor {
    /// A human acting directly (no agent).
    pub fn human(user: uuid::Uuid) -> Self {
        Self { user, agent: None }
    }

    /// A human acting through a named agent.
    pub fn agent(user: uuid::Uuid, agent: impl Into<String>) -> Self {
        Self {
            user,
            agent: Some(agent.into()),
        }
    }
}
