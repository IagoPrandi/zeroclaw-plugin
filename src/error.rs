use std::collections::BTreeMap;

use serde::Serialize;
use thiserror::Error;

#[derive(Clone, Debug, Error, PartialEq, Eq)]
pub enum GuardianError {
    #[error("{message}")]
    InvalidInput { message: String },
    #[error("{message}")]
    InvalidConfig { message: String },
    #[error("The transaction is not valid Base64.")]
    Base64Decode,
    #[error("The transaction exceeds the configured byte limit.")]
    TransactionTooLarge,
    #[error("The transaction wire format is invalid.")]
    TransactionDeserialize,
    #[error("The transaction version is not supported.")]
    UnsupportedVersion,
    #[error("The transaction contains an invalid account index.")]
    InvalidAccountIndex,
    #[error("An address lookup table could not be resolved.")]
    AddressLookupTable,
    #[error("The Solana RPC transport failed.")]
    RpcTransport,
    #[error("The Solana RPC returned an invalid HTTP response.")]
    RpcHttp,
    #[error("The Solana RPC returned an invalid protocol response.")]
    RpcProtocol,
    #[error("The Solana RPC rate limit was reached.")]
    RpcRateLimited,
    #[error("The confirmed transaction was not found.")]
    TransactionNotFound,
    #[error("A required account was not found.")]
    AccountNotFound,
    #[error("Transaction simulation failed.")]
    Simulation,
    #[error("Instruction decoding failed.")]
    Decoder,
    #[error("An arithmetic operation overflowed.")]
    ArithmeticOverflow,
    #[error("The report exceeds the configured output limit.")]
    OutputLimit,
    #[error("An internal analysis error occurred.")]
    Internal,
}

impl GuardianError {
    #[must_use]
    pub fn invalid_input(message: impl Into<String>) -> Self {
        Self::InvalidInput {
            message: message.into(),
        }
    }

    #[must_use]
    pub fn invalid_config(message: impl Into<String>) -> Self {
        Self::InvalidConfig {
            message: message.into(),
        }
    }

    #[must_use]
    pub const fn code(&self) -> &'static str {
        match self {
            Self::InvalidInput { .. } => "INVALID_INPUT",
            Self::InvalidConfig { .. } => "INVALID_CONFIG",
            Self::Base64Decode => "BASE64_DECODE",
            Self::TransactionTooLarge => "TRANSACTION_TOO_LARGE",
            Self::TransactionDeserialize => "TRANSACTION_DESERIALIZE",
            Self::UnsupportedVersion => "UNSUPPORTED_TRANSACTION_VERSION",
            Self::InvalidAccountIndex => "INVALID_ACCOUNT_INDEX",
            Self::AddressLookupTable => "UNRESOLVED_ADDRESS_LOOKUP_TABLE",
            Self::RpcTransport => "RPC_TRANSPORT",
            Self::RpcHttp => "RPC_HTTP",
            Self::RpcProtocol => "RPC_PROTOCOL",
            Self::RpcRateLimited => "RPC_RATE_LIMITED",
            Self::TransactionNotFound => "TX_NOT_FOUND",
            Self::AccountNotFound => "ACCOUNT_NOT_FOUND",
            Self::Simulation => "SIMULATION",
            Self::Decoder => "DECODER",
            Self::ArithmeticOverflow => "ARITHMETIC_OVERFLOW",
            Self::OutputLimit => "OUTPUT_LIMIT",
            Self::Internal => "INTERNAL",
        }
    }

    #[must_use]
    pub const fn retryable(&self) -> bool {
        matches!(
            self,
            Self::RpcTransport
                | Self::RpcRateLimited
                | Self::TransactionNotFound
                | Self::AccountNotFound
                | Self::AddressLookupTable
        )
    }

    #[must_use]
    pub const fn rule_id(&self) -> Option<&'static str> {
        match self {
            Self::UnsupportedVersion => Some("COV-001"),
            Self::AddressLookupTable => Some("COV-002"),
            Self::AccountNotFound => Some("COV-006"),
            Self::RpcProtocol => Some("COV-007"),
            _ => None,
        }
    }

    #[must_use]
    pub fn envelope(&self) -> ErrorEnvelope {
        let mut details = BTreeMap::new();
        if let Some(rule_id) = self.rule_id() {
            details.insert("rule_id".to_owned(), serde_json::json!(rule_id));
        }
        ErrorEnvelope {
            schema_version: "1.0.0",
            error: ErrorBody {
                code: self.code(),
                message: self.to_string(),
                retryable: self.retryable(),
                details,
            },
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ErrorEnvelope {
    pub schema_version: &'static str,
    pub error: ErrorBody,
}

#[derive(Debug, Serialize)]
pub struct ErrorBody {
    pub code: &'static str,
    pub message: String,
    pub retryable: bool,
    pub details: BTreeMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::GuardianError;

    #[test]
    fn fail_closed_prerequisite_errors_expose_rule_ids() {
        for (error, rule_id) in [
            (GuardianError::UnsupportedVersion, "COV-001"),
            (GuardianError::AddressLookupTable, "COV-002"),
            (GuardianError::AccountNotFound, "COV-006"),
            (GuardianError::RpcProtocol, "COV-007"),
        ] {
            assert_eq!(
                error.envelope().error.details.get("rule_id"),
                Some(&serde_json::json!(rule_id))
            );
        }
    }
}
