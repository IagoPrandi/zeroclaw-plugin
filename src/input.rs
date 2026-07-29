use std::collections::HashSet;

use serde::Deserialize;

use crate::{address::Address32, error::GuardianError};

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct GuardianInput {
    pub source: TransactionSource,
    pub cluster: Cluster,
    #[serde(default)]
    pub observed_wallets: Vec<Address32>,
    pub expected_intent: Option<ExpectedIntent>,
    #[serde(default)]
    pub output_language: OutputLanguage,
}

impl GuardianInput {
    /// Enforce constraints that cannot be expressed by Serde alone.
    ///
    /// # Errors
    ///
    /// Returns `InvalidInput` when a length, uniqueness, or decimal constraint
    /// is violated.
    pub fn validate(&self) -> Result<(), GuardianError> {
        if self.observed_wallets.len() > 10 {
            return Err(GuardianError::invalid_input(
                "observed_wallets may contain at most 10 addresses",
            ));
        }
        if self.observed_wallets.iter().collect::<HashSet<_>>().len() != self.observed_wallets.len()
        {
            return Err(GuardianError::invalid_input(
                "observed_wallets must be unique",
            ));
        }
        if let Some(intent) = &self.expected_intent {
            intent.validate()?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum TransactionSource {
    Serialized { transaction_base64: String },
    Confirmed { signature: String },
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
pub enum Cluster {
    #[serde(rename = "mainnet-beta")]
    MainnetBeta,
    #[serde(rename = "devnet")]
    Devnet,
    #[serde(rename = "testnet")]
    Testnet,
}

impl Cluster {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::MainnetBeta => "mainnet-beta",
            Self::Devnet => "devnet",
            Self::Testnet => "testnet",
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq)]
pub enum OutputLanguage {
    #[default]
    #[serde(rename = "en")]
    English,
    #[serde(rename = "pt-BR")]
    PortugueseBrazil,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExpectedIntent {
    pub description: Option<String>,
    #[serde(default)]
    pub allowed_programs: Vec<Address32>,
    #[serde(default)]
    pub allowed_recipients: Vec<Address32>,
    pub max_sol_out_lamports: Option<String>,
    #[serde(default)]
    pub token_limits: Vec<TokenLimit>,
}

impl ExpectedIntent {
    fn validate(&self) -> Result<(), GuardianError> {
        if self.description.as_ref().is_some_and(|v| v.len() > 500) {
            return Err(GuardianError::invalid_input(
                "expected_intent.description exceeds 500 bytes",
            ));
        }
        if self.allowed_programs.len() > 20
            || self.allowed_recipients.len() > 20
            || self.token_limits.len() > 20
        {
            return Err(GuardianError::invalid_input(
                "expected_intent arrays may contain at most 20 items",
            ));
        }
        if let Some(value) = &self.max_sol_out_lamports {
            parse_decimal_u64(value, "max_sol_out_lamports")?;
        }
        for limit in &self.token_limits {
            if let Some(value) = &limit.max_out_raw {
                parse_decimal_u128(value, "token max_out_raw")?;
            }
            if let Some(value) = &limit.min_in_raw {
                parse_decimal_u128(value, "token min_in_raw")?;
            }
        }
        Ok(())
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TokenLimit {
    pub mint: Address32,
    pub max_out_raw: Option<String>,
    pub min_in_raw: Option<String>,
}

/// Parse a canonical unsigned decimal string.
///
/// # Errors
///
/// Returns `InvalidInput` for non-decimal or overflowing values.
pub fn parse_decimal_u64(value: &str, field: &str) -> Result<u64, GuardianError> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(GuardianError::invalid_input(format!(
            "{field} must be an unsigned decimal string"
        )));
    }
    value
        .parse()
        .map_err(|_| GuardianError::invalid_input(format!("{field} exceeds u64")))
}

/// Parse a canonical unsigned decimal string up to `u128`.
///
/// # Errors
///
/// Returns `InvalidInput` for non-decimal or overflowing values.
pub fn parse_decimal_u128(value: &str, field: &str) -> Result<u128, GuardianError> {
    if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
        return Err(GuardianError::invalid_input(format!(
            "{field} must be an unsigned decimal string"
        )));
    }
    value
        .parse()
        .map_err(|_| GuardianError::invalid_input(format!("{field} exceeds u128")))
}
