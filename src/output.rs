use std::collections::BTreeMap;

use serde::Serialize;

use crate::address::Address32;

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Decision {
    Allow,
    Review,
    Block,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Severity {
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionStatus {
    NotSimulated,
    SimulationSucceeded,
    SimulationFailed,
    ConfirmedSucceeded,
    ConfirmedFailed,
    RpcUnavailable,
}

#[derive(Debug, Serialize)]
pub struct GuardianReport {
    pub schema_version: &'static str,
    pub plugin_version: &'static str,
    pub policy_version: String,
    pub analysis_id: String,
    pub source: SourceReport,
    pub decision: Decision,
    pub risk_level: Severity,
    pub risk_score: u8,
    pub confidence: f64,
    pub analysis_complete: bool,
    pub execution: ExecutionReport,
    pub fees: FeeReport,
    pub participants: Participants,
    pub actions: Vec<Action>,
    pub asset_deltas: Vec<AssetDelta>,
    pub authority_changes: Vec<AuthorityChange>,
    pub findings: Vec<Finding>,
    pub coverage: Coverage,
    pub summary: String,
    pub limitations: Vec<String>,
    pub generated_at: String,
}

#[derive(Debug, Serialize)]
pub struct SourceReport {
    #[serde(rename = "type")]
    pub source_type: &'static str,
    pub cluster: String,
    pub signature: Option<String>,
    pub slot: Option<u64>,
    pub transaction_version: String,
}

#[derive(Debug, Serialize)]
pub struct ExecutionReport {
    pub status: ExecutionStatus,
    pub error: Option<String>,
    pub units_consumed: Option<u64>,
    pub logs_truncated: bool,
    pub return_data: Option<ReturnDataReport>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct ReturnDataReport {
    pub program_id: Address32,
    pub encoding: String,
    pub data_length: usize,
    pub data_sha256: String,
}

#[derive(Debug, Default, Serialize)]
pub struct FeeReport {
    pub fee_payer: Option<Address32>,
    pub base_fee_lamports: Option<String>,
    pub priority_fee_lamports: Option<String>,
    pub total_estimated_fee_lamports: Option<String>,
    pub compute_unit_limit: Option<String>,
    pub compute_unit_price_micro_lamports: Option<String>,
}

#[derive(Debug, Default, Serialize)]
pub struct Participants {
    pub signers: Vec<Address32>,
    pub writable_accounts: Vec<Address32>,
    pub programs: Vec<Address32>,
    pub unknown_programs: Vec<Address32>,
}

#[derive(Debug, Serialize)]
pub struct Action {
    pub instruction_index: usize,
    pub inner_index: Option<usize>,
    pub kind: String,
    pub program_id: Address32,
    pub accounts: Vec<Address32>,
    pub details: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Serialize)]
pub struct AssetDelta {
    pub account: Address32,
    pub owner: Option<Address32>,
    pub mint: Option<Address32>,
    pub program_id: Option<Address32>,
    pub asset: String,
    pub raw_delta: String,
    pub decimals: Option<u8>,
}

#[derive(Debug, Serialize)]
pub struct AuthorityChange {
    pub instruction_index: usize,
    pub account: Address32,
    pub authority_type: String,
    pub old_authority: Option<Address32>,
    pub new_authority: Option<Address32>,
}

#[derive(Debug, Serialize)]
pub struct Finding {
    pub rule_id: String,
    pub severity: Severity,
    pub category: String,
    pub title: String,
    pub explanation: String,
    pub evidence: BTreeMap<String, serde_json::Value>,
    pub decision_effect: DecisionEffect,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DecisionEffect {
    Block,
    Review,
    None,
}

#[derive(Debug, Serialize)]
pub struct Coverage {
    pub top_level_instructions: InstructionCoverage,
    pub inner_instructions_available: bool,
    pub address_lookup_tables_resolved: bool,
    pub simulation_available: bool,
    pub unresolved_components: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct InstructionCoverage {
    pub decoded: usize,
    pub total: usize,
}
