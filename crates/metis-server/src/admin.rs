//! Admin operations run directly against the database (the bootstrap path).
//!
//! Creating the first user and token can't go through the authenticated API —
//! there's no token yet. These commands connect with the same `DATABASE_URL`
//! and operate directly. Everything else should go through the API.

use metis_core::db;
use metis_core::service::user;

use crate::auth::generate_token;

/// Create a user. Returns the created user's id.
pub fn create_user(
    database_url: &str,
    username: &str,
    display_name: &str,
    email: Option<&str>,
    is_admin: bool,
) -> anyhow::Result<uuid::Uuid> {
    let pool = db::connect_and_migrate(database_url)?;
    let mut conn = pool.get()?;
    let u = user::create_user(&mut conn, username, display_name, email, is_admin)?;
    Ok(u.id.0)
}

/// Issue a token for a user (by username), optionally tied to a named agent.
/// Returns the plaintext token — shown exactly once.
pub fn create_token(
    database_url: &str,
    username: &str,
    token_name: &str,
    agent_name: Option<&str>,
) -> anyhow::Result<String> {
    let pool = db::connect_and_migrate(database_url)?;
    let mut conn = pool.get()?;
    let user = user::get_user_by_name(&mut conn, username)?;
    let plaintext = generate_token();
    user::issue_token(&mut conn, user.id.0, token_name, agent_name, &plaintext)?;
    Ok(plaintext)
}
