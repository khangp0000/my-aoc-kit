//! Configuration resolution from CLI args

use crate::cli::{Args, ParallelizeBy};
use crate::error::CliError;
use std::path::{Path, PathBuf};
use zeroize::Zeroizing;

/// Resolved runtime configuration
pub struct Config {
    /// Year filter (None = all years)
    pub year_filter: Option<u16>,
    /// Day filter (None = all days)
    pub day_filter: Option<u8>,
    /// Part filter (None = all parts)
    pub part_filter: Option<u8>,
    /// Tags to filter solvers
    pub tags: Vec<String>,
    /// Cache directory path
    pub cache_dir: PathBuf,
    /// Number of threads for parallel execution
    pub thread_count: usize,
    /// Parallelization level
    pub parallelize_by: ParallelizeBy,
    /// Whether to submit answers
    pub submit: bool,
    /// User ID for cache organization
    pub user_id: u64,
    /// Whether user ID was explicitly provided (vs derived from session)
    pub user_id_provided: bool,
    /// Session key (zeroized on drop)
    pub session: Zeroizing<String>,
    /// Whether to auto-retry on throttle
    pub auto_retry: bool,
    /// Quiet mode
    pub quiet: bool,
}

impl Config {
    /// Build config from CLI args, resolving session and user ID
    pub fn from_args(args: Args) -> Result<Self, CliError> {
        // Resolve cache directory (expand ~)
        let cache_dir = expand_tilde(&args.cache_dir);

        // Resolve thread count
        let thread_count = args.threads.unwrap_or_else(num_cpus);

        // Resolve session and user ID
        let user_id_provided = args.user_id.is_some();
        let (session, user_id) = resolve_session_and_user_id(args.user_id, args.submit)?;

        Ok(Config {
            year_filter: args.year,
            day_filter: args.day,
            part_filter: args.part,
            tags: args.tags,
            cache_dir,
            thread_count,
            parallelize_by: args.parallelize_by,
            submit: args.submit,
            user_id,
            user_id_provided,
            session,
            auto_retry: args.auto_retry,
            quiet: args.quiet,
        })
    }
}

/// Expand ~ to home directory
fn expand_tilde(path: &Path) -> PathBuf {
    if let Some(path_str) = path.to_str()
        && (path_str.starts_with("~/") || path_str == "~")
        && let Some(home) = dirs::home_dir()
    {
        return home.join(&path_str[2..]);
    }
    path.to_path_buf()
}

/// Get number of CPUs
fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1)
}

/// Prompt user for their AOC user ID
fn prompt_user_id() -> Result<u64, CliError> {
    use std::io::Write;
    println!("No user ID provided. Enter your AOC user ID (found in your profile URL).");
    print!("User ID: ");
    std::io::stdout().flush().ok();

    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .map_err(|e| CliError::Config(format!("Failed to read user ID: {}", e)))?;

    input
        .trim()
        .parse()
        .map_err(|_| CliError::Config("Invalid user ID: must be a number".to_string()))
}

/// Prompt user for session token
pub fn prompt_session(reason: &str) -> Result<Zeroizing<String>, CliError> {
    println!("{}", reason);
    let s = rpassword::prompt_password("Enter AOC session key: ")
        .map_err(|e| CliError::Config(format!("Failed to read session: {}", e)))?;
    if s.is_empty() {
        return Err(CliError::Config("Session token is required.".to_string()));
    }
    Ok(Zeroizing::new(s))
}

/// Verify session and optionally check user ID match
pub fn verify_session(session: &str, expected_user_id: Option<u64>) -> Result<u64, CliError> {
    let client = aoc_http_client::AocClient::new()?;
    let info = client.verify_session(session)?;
    let actual_uid = info
        .user_id
        .ok_or_else(|| CliError::Config("Invalid session: could not fetch user ID".to_string()))?;

    if let Some(expected) = expected_user_id
        && actual_uid != expected
    {
        return Err(CliError::UserIdMismatch {
            expected,
            actual: actual_uid,
        });
    }
    Ok(actual_uid)
}

/// Resolve session key and user ID
fn resolve_session_and_user_id(
    provided_user_id: Option<u64>,
    submit: bool,
) -> Result<(Zeroizing<String>, u64), CliError> {
    let env_session = std::env::var("AOC_SESSION").ok();

    // Determine user ID: from CLI, from env session, or prompt
    // Track if user explicitly provided/entered a user ID (vs derived from session)
    let (user_id, user_provided_or_prompted) = match (provided_user_id, &env_session) {
        (Some(uid), _) => (Some(uid), true),             // CLI provided
        (None, Some(_)) => (None, false),                // Will fetch from session
        (None, None) => (Some(prompt_user_id()?), true), // User prompted
    };

    // Resolve session: from env, prompt if needed for submit
    let session = match env_session {
        Some(s) => Zeroizing::new(s),
        None if submit => prompt_session("Session token required for submission")?,
        None => Zeroizing::new(String::new()),
    };

    // Verify session and resolve final user ID
    let user_id = if !session.is_empty() {
        let expected = if user_provided_or_prompted {
            user_id
        } else {
            None
        };
        verify_session(&session, expected)?
    } else {
        // No session - user_id must have been provided or prompted
        user_id.expect("User ID should be set when no session is available")
    };

    Ok((session, user_id))
}
