//! Solana Transaction Guardian.
//!
//! Business logic stays target-independent. The component adapter is compiled
//! only for `wasm32-wasip2`.

pub mod address;
pub mod config;
pub mod core;
pub mod decoders;
pub mod error;
pub mod input;
pub mod limits;
pub mod output;
pub mod risk;
pub mod rpc;
pub mod schema;
pub mod simulation;
pub mod spike;
pub mod state;
pub mod transaction;

#[cfg(target_family = "wasm")]
#[allow(unsafe_code)]
mod component {
    wit_bindgen::generate!({
        path: "wit/v0",
        world: "tool-plugin",
        features: ["plugins-wit-v0"],
    });

    use std::collections::HashMap;

    use exports::zeroclaw::plugin::plugin_info::Guest as PluginInfo;
    use exports::zeroclaw::plugin::tool::{Guest as Tool, ToolResult};
    use zeroclaw::plugin::logging::{self, LogLevel, PluginAction, PluginEvent, PluginOutcome};
    struct GuardianPlugin;

    fn log(
        level: LogLevel,
        action: PluginAction,
        outcome: Option<PluginOutcome>,
        message: &str,
        attrs: Option<String>,
    ) {
        logging::log_record(
            level,
            &PluginEvent {
                function_name: "solana_transaction_guardian::execute".to_owned(),
                action,
                outcome,
                duration_ms: None,
                attrs,
                message: message.to_owned(),
            },
        );
    }

    impl PluginInfo for GuardianPlugin {
        fn plugin_name() -> String {
            "solana-transaction-guardian".to_owned()
        }

        fn plugin_version() -> String {
            env!("CARGO_PKG_VERSION").to_owned()
        }
    }

    impl Tool for GuardianPlugin {
        fn name() -> String {
            "solana_transaction_guardian".to_owned()
        }

        fn description() -> String {
            "Analyze a serialized or confirmed Solana transaction before it is trusted. \
             Returns decoded actions, asset deltas, authority changes, simulation results, \
             policy violations, coverage limitations, and an allow/review/block decision. \
             This tool never signs or submits transactions. The agent must preserve the tool \
             decision exactly and must not infer safety when the tool fails."
                .to_owned()
        }

        fn parameters_schema() -> String {
            crate::schema::parameters_schema()
        }

        fn execute(args: String) -> Result<ToolResult, String> {
            log(
                LogLevel::Info,
                PluginAction::Start,
                None,
                "Guardian analysis started.",
                None,
            );
            let mut wire: serde_json::Value = match serde_json::from_str(&args) {
                Ok(value) => value,
                Err(error) => {
                    let guardian_error = crate::error::GuardianError::invalid_input(format!(
                        "invalid arguments: {error}"
                    ));
                    log(
                        LogLevel::Warn,
                        PluginAction::Fail,
                        Some(PluginOutcome::Failure),
                        "Guardian rejected invalid arguments.",
                        Some(format!(r#"{{"error_code":"{}"}}"#, guardian_error.code())),
                    );
                    return Ok(ToolResult {
                        success: false,
                        output: String::new(),
                        error: Some(
                            serde_json::to_string(&guardian_error.envelope()).unwrap_or_else(
                                |_| "{\"error\":{\"code\":\"INTERNAL\"}}".to_owned(),
                            ),
                        ),
                    });
                }
            };
            let config = wire
                .as_object_mut()
                .and_then(|object| object.remove("__config"))
                .and_then(|value| serde_json::from_value::<HashMap<String, String>>(value).ok())
                .unwrap_or_default();
            let input = wire.to_string();
            match crate::core::analyze_with_rpc(&input, &config, crate::rpc::WakiTransport) {
                Ok(report) => match serde_json::to_string(&report) {
                    Ok(output) => {
                        let exceeds_limit = crate::config::GuardianConfig::parse(&config)
                            .is_ok_and(|parsed| output.len() > parsed.limits.max_output_bytes);
                        if exceeds_limit {
                            let error = crate::error::GuardianError::OutputLimit;
                            log(
                                LogLevel::Warn,
                                PluginAction::Fail,
                                Some(PluginOutcome::Failure),
                                "Guardian report exceeded the configured output limit.",
                                Some(format!(r#"{{"error_code":"{}"}}"#, error.code())),
                            );
                            Ok(ToolResult {
                                success: false,
                                output: String::new(),
                                error: Some(
                                    serde_json::to_string(&error.envelope()).unwrap_or_else(|_| {
                                        "{\"error\":{\"code\":\"INTERNAL\"}}".to_owned()
                                    }),
                                ),
                            })
                        } else {
                            log(
                                LogLevel::Info,
                                PluginAction::Complete,
                                Some(PluginOutcome::Success),
                                "Guardian analysis completed.",
                                Some(format!(
                                    r#"{{"analysis_id":"{}","decision":"{:?}"}}"#,
                                    report.analysis_id, report.decision
                                )),
                            );
                            Ok(ToolResult {
                                success: true,
                                output,
                                error: None,
                            })
                        }
                    }
                    Err(_) => Err("failed to serialize Guardian report".to_owned()),
                },
                Err(error) => {
                    log(
                        LogLevel::Warn,
                        PluginAction::Fail,
                        Some(PluginOutcome::Failure),
                        "Guardian analysis failed.",
                        Some(format!(r#"{{"error_code":"{}"}}"#, error.code())),
                    );
                    let serialized = serde_json::to_string(&error.envelope())
                        .unwrap_or_else(|_| "{\"error\":{\"code\":\"INTERNAL\"}}".to_owned());
                    Ok(ToolResult {
                        success: false,
                        output: String::new(),
                        error: Some(serialized),
                    })
                }
            }
        }
    }

    export!(GuardianPlugin);
}
