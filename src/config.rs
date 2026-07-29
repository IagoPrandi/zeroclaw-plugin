use std::collections::{HashMap, HashSet};

use serde::Deserialize;

use crate::{address::Address32, error::GuardianError, limits::Limits};

#[derive(Clone, Debug)]
pub struct GuardianConfig {
    pub rpc_endpoints: HashMap<String, String>,
    pub allowed_clusters: HashSet<String>,
    pub request_timeout_ms: u64,
    pub limits: Limits,
    pub fail_closed: bool,
    pub enable_simulation: bool,
    pub policy_version: String,
    pub policy: Policy,
}

#[derive(Clone, Debug)]
pub struct Policy {
    pub allowed_programs: HashSet<Address32>,
    pub blocked_programs: HashSet<Address32>,
    pub known_recipients: HashMap<Address32, String>,
    pub blocked_recipients: HashSet<Address32>,
    pub sol_out_review_lamports: u64,
    pub sol_out_block_lamports: u64,
    pub priority_fee_review_lamports: u64,
    pub priority_fee_block_lamports: u64,
    pub minimum_sol_reserve_lamports: u64,
    pub unknown_program_policy: PolicyEffect,
    pub unresolved_alt_policy: PolicyEffect,
    pub simulation_unavailable_policy: PolicyEffect,
    pub token2022_transfer_hook_policy: PolicyEffect,
    pub token2022_permanent_delegate_policy: PolicyEffect,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum PolicyEffect {
    None,
    Review,
    Block,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct KnownRecipient {
    address: Address32,
    label: String,
}

impl GuardianConfig {
    /// Parse and validate the host-injected string map.
    ///
    /// # Errors
    ///
    /// Returns `InvalidConfig` for every missing, malformed, unsafe, or
    /// internally inconsistent value.
    #[allow(clippy::too_many_lines)]
    pub fn parse(values: &HashMap<String, String>) -> Result<Self, GuardianError> {
        let rpc_endpoints: HashMap<String, String> = json_required(values, "rpc_endpoints_json")?;
        if rpc_endpoints.is_empty() {
            return Err(GuardianError::invalid_config(
                "rpc_endpoints_json must not be empty",
            ));
        }
        for (cluster, endpoint) in &rpc_endpoints {
            validate_cluster(cluster)?;
            validate_endpoint(endpoint)?;
        }

        let allowed_clusters: Vec<String> = json_required(values, "allowed_clusters_json")?;
        if allowed_clusters.is_empty() {
            return Err(GuardianError::invalid_config(
                "allowed_clusters_json must not be empty",
            ));
        }
        let allowed_clusters: HashSet<_> = allowed_clusters.into_iter().collect();
        for cluster in &allowed_clusters {
            validate_cluster(cluster)?;
            if !rpc_endpoints.contains_key(cluster) {
                return Err(GuardianError::invalid_config(format!(
                    "allowed cluster {cluster} has no RPC endpoint"
                )));
            }
        }

        let known: Vec<KnownRecipient> =
            json_optional(values, "known_recipients_json")?.unwrap_or_default();
        let mut known_recipients = HashMap::new();
        for entry in known {
            if entry.label.is_empty() || entry.label.len() > 100 {
                return Err(GuardianError::invalid_config(
                    "recipient labels must contain 1..=100 bytes",
                ));
            }
            if known_recipients
                .insert(entry.address, entry.label)
                .is_some()
            {
                return Err(GuardianError::invalid_config(
                    "known recipient addresses must be unique",
                ));
            }
        }

        let limits = Limits {
            max_rpc_calls: number(values, "max_rpc_calls", 1, 64)?,
            max_http_response_bytes: number(values, "max_http_response_bytes", 1_024, 16_777_216)?,
            max_transaction_bytes: number(values, "max_transaction_bytes", 1, 16_384)?,
            max_output_bytes: number(values, "max_output_bytes", 1_024, 1_048_576)?,
            max_accounts: 512,
            max_instructions: 256,
        };

        let policy = Policy {
            allowed_programs: json_optional::<Vec<Address32>>(values, "allowed_programs_json")?
                .unwrap_or_default()
                .into_iter()
                .collect(),
            blocked_programs: json_optional::<Vec<Address32>>(values, "blocked_programs_json")?
                .unwrap_or_default()
                .into_iter()
                .collect(),
            known_recipients,
            blocked_recipients: json_optional::<Vec<Address32>>(values, "blocked_recipients_json")?
                .unwrap_or_default()
                .into_iter()
                .collect(),
            sol_out_review_lamports: optional_number(
                values,
                "sol_out_review_lamports",
                100_000_000,
            )?,
            sol_out_block_lamports: optional_number(
                values,
                "sol_out_block_lamports",
                1_000_000_000,
            )?,
            priority_fee_review_lamports: optional_number(
                values,
                "priority_fee_review_lamports",
                100_000,
            )?,
            priority_fee_block_lamports: optional_number(
                values,
                "priority_fee_block_lamports",
                1_000_000,
            )?,
            minimum_sol_reserve_lamports: optional_number(
                values,
                "minimum_sol_reserve_lamports",
                10_000_000,
            )?,
            unknown_program_policy: effect(values, "unknown_program_policy", "review")?,
            unresolved_alt_policy: effect(values, "unresolved_alt_policy", "block")?,
            simulation_unavailable_policy: effect(
                values,
                "simulation_unavailable_policy",
                "block",
            )?,
            token2022_transfer_hook_policy: effect(
                values,
                "token2022_transfer_hook_policy",
                "review",
            )?,
            token2022_permanent_delegate_policy: effect(
                values,
                "token2022_permanent_delegate_policy",
                "block",
            )?,
        };
        if policy.sol_out_review_lamports > policy.sol_out_block_lamports
            || policy.priority_fee_review_lamports > policy.priority_fee_block_lamports
        {
            return Err(GuardianError::invalid_config(
                "review thresholds must not exceed block thresholds",
            ));
        }

        let request_timeout_ms = number(values, "request_timeout_ms", 100, 60_000)?;
        let policy_version = required(values, "policy_version")?.to_owned();
        if policy_version.is_empty() || policy_version.len() > 64 {
            return Err(GuardianError::invalid_config(
                "policy_version must contain 1..=64 bytes",
            ));
        }

        Ok(Self {
            rpc_endpoints,
            allowed_clusters,
            request_timeout_ms,
            limits,
            fail_closed: boolean(values, "fail_closed")?,
            enable_simulation: boolean(values, "enable_simulation")?,
            policy_version,
            policy,
        })
    }
}

fn required<'a>(values: &'a HashMap<String, String>, key: &str) -> Result<&'a str, GuardianError> {
    values
        .get(key)
        .map(String::as_str)
        .ok_or_else(|| GuardianError::invalid_config(format!("missing required key {key}")))
}

fn json_required<T: for<'de> Deserialize<'de>>(
    values: &HashMap<String, String>,
    key: &str,
) -> Result<T, GuardianError> {
    serde_json::from_str(required(values, key)?)
        .map_err(|_| GuardianError::invalid_config(format!("{key} is invalid JSON")))
}

fn json_optional<T: for<'de> Deserialize<'de>>(
    values: &HashMap<String, String>,
    key: &str,
) -> Result<Option<T>, GuardianError> {
    values
        .get(key)
        .map(|value| {
            serde_json::from_str(value)
                .map_err(|_| GuardianError::invalid_config(format!("{key} is invalid JSON")))
        })
        .transpose()
}

fn number<T>(
    values: &HashMap<String, String>,
    key: &str,
    min: T,
    max: T,
) -> Result<T, GuardianError>
where
    T: Copy + Ord + std::str::FromStr,
{
    let value = required(values, key)?
        .parse::<T>()
        .map_err(|_| GuardianError::invalid_config(format!("{key} is not a valid number")))?;
    if value < min || value > max {
        return Err(GuardianError::invalid_config(format!(
            "{key} is outside its allowed range"
        )));
    }
    Ok(value)
}

fn optional_number(
    values: &HashMap<String, String>,
    key: &str,
    default: u64,
) -> Result<u64, GuardianError> {
    values.get(key).map_or(Ok(default), |value| {
        value
            .parse()
            .map_err(|_| GuardianError::invalid_config(format!("{key} is not a valid u64")))
    })
}

fn boolean(values: &HashMap<String, String>, key: &str) -> Result<bool, GuardianError> {
    required(values, key)?
        .parse()
        .map_err(|_| GuardianError::invalid_config(format!("{key} must be true or false")))
}

fn effect(
    values: &HashMap<String, String>,
    key: &str,
    default: &str,
) -> Result<PolicyEffect, GuardianError> {
    serde_json::from_value(serde_json::Value::String(
        values.get(key).map_or(default, String::as_str).to_owned(),
    ))
    .map_err(|_| GuardianError::invalid_config(format!("{key} is not a valid policy effect")))
}

fn validate_cluster(value: &str) -> Result<(), GuardianError> {
    if matches!(value, "mainnet-beta" | "devnet" | "testnet") {
        Ok(())
    } else {
        Err(GuardianError::invalid_config(format!(
            "unsupported cluster {value}"
        )))
    }
}

fn validate_endpoint(value: &str) -> Result<(), GuardianError> {
    if value.starts_with("https://") {
        return Ok(());
    }
    if value.starts_with("http://127.0.0.1:")
        || value.starts_with("http://localhost:")
        || value == "http://127.0.0.1"
        || value == "http://localhost"
    {
        return Ok(());
    }
    Err(GuardianError::invalid_config(
        "RPC endpoints must use HTTPS; HTTP is limited to localhost tests",
    ))
}

#[cfg(test)]
pub(crate) fn valid_test_config() -> HashMap<String, String> {
    [
        (
            "rpc_endpoints_json",
            r#"{"devnet":"https://api.devnet.solana.com"}"#,
        ),
        ("allowed_clusters_json", r#"["devnet"]"#),
        ("request_timeout_ms", "5000"),
        ("max_rpc_calls", "8"),
        ("max_http_response_bytes", "2097152"),
        ("max_transaction_bytes", "1232"),
        ("max_output_bytes", "262144"),
        ("fail_closed", "true"),
        ("enable_simulation", "true"),
        ("policy_version", "default-1"),
    ]
    .into_iter()
    .map(|(key, value)| (key.to_owned(), value.to_owned()))
    .collect()
}

#[cfg(test)]
mod tests {
    use super::{GuardianConfig, valid_test_config};

    #[test]
    fn parses_valid_config() {
        assert!(GuardianConfig::parse(&valid_test_config()).is_ok());
    }

    #[test]
    fn rejects_missing_mandatory_value() {
        let mut values = valid_test_config();
        values.remove("fail_closed");
        assert!(GuardianConfig::parse(&values).is_err());
    }

    #[test]
    fn rejects_remote_plain_http() {
        let mut values = valid_test_config();
        values.insert(
            "rpc_endpoints_json".to_owned(),
            r#"{"devnet":"http://example.com"}"#.to_owned(),
        );
        assert!(GuardianConfig::parse(&values).is_err());
    }
}
