//! Short codes: `{PREFIX}-{LETTER}-{NNNN}`, e.g. `METIS-T-0042`.
//!
//! The prefix is the project's code (it may itself contain hyphens, so parsing
//! works from the right). The letter is a type's short-code letter; the number
//! is a per-project+type sequence, zero-padded to at least four digits.

/// A parsed short code.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShortCode {
    /// Project prefix (e.g. `"METIS"`).
    pub prefix: String,
    /// Type letter(s) (e.g. `"T"`).
    pub letter: String,
    /// Per-type sequence number.
    pub seq: u32,
}

/// Why a short code could not be parsed.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ShortCodeError {
    /// The string is not in `{PREFIX}-{LETTER}-{NNNN}` form.
    #[error("malformed short code {0:?}")]
    Malformed(String),
}

/// Format a short code, zero-padding the sequence to four digits.
pub fn format_short_code(prefix: &str, letter: &str, seq: u32) -> String {
    format!("{prefix}-{letter}-{seq:04}")
}

/// Parse a short code. Parses from the right so hyphenated prefixes work.
pub fn parse_short_code(code: &str) -> Result<ShortCode, ShortCodeError> {
    let malformed = || ShortCodeError::Malformed(code.to_string());

    // [number, letter, prefix] reading right-to-left.
    let mut parts = code.rsplitn(3, '-');
    let num = parts.next().ok_or_else(malformed)?;
    let letter = parts.next().ok_or_else(malformed)?;
    let prefix = parts.next().ok_or_else(malformed)?;

    if prefix.is_empty() {
        return Err(malformed());
    }
    if letter.is_empty() || letter.len() > 2 || !letter.chars().all(|c| c.is_ascii_uppercase()) {
        return Err(malformed());
    }
    if num.is_empty() || !num.chars().all(|c| c.is_ascii_digit()) {
        return Err(malformed());
    }
    let seq: u32 = num.parse().map_err(|_| malformed())?;

    Ok(ShortCode {
        prefix: prefix.to_string(),
        letter: letter.to_string(),
        seq,
    })
}
