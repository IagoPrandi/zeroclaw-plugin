use std::fmt;

use serde_json::{Value, json};

use crate::{error::GuardianError, limits::Budget};

#[derive(Debug)]
pub struct HttpResponse {
    pub status: u16,
    pub body: Vec<u8>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransportError {
    Timeout,
    Other,
}

pub trait RpcTransport {
    /// Send one bounded HTTP POST.
    ///
    /// # Errors
    ///
    /// Returns a transport category without exposing request contents.
    fn post(
        &mut self,
        endpoint: &str,
        body: &[u8],
        timeout_ms: u64,
        max_response_bytes: usize,
    ) -> Result<HttpResponse, TransportError>;
}

#[derive(Debug)]
pub struct RpcClient<T> {
    endpoint: String,
    timeout_ms: u64,
    transport: T,
    budget: Budget,
    next_id: u64,
}

impl<T: RpcTransport> RpcClient<T> {
    #[must_use]
    pub const fn new(endpoint: String, timeout_ms: u64, transport: T, budget: Budget) -> Self {
        Self {
            endpoint,
            timeout_ms,
            transport,
            budget,
            next_id: 1,
        }
    }

    /// Execute one validated JSON-RPC call.
    ///
    /// # Errors
    ///
    /// Returns a typed error for exhausted budgets, transport/HTTP failures,
    /// oversized bodies, mismatched IDs, invalid envelopes, and RPC errors.
    pub fn call(&mut self, method: &str, params: &Value) -> Result<Value, GuardianError> {
        let id = self.next_id;
        self.next_id = self
            .next_id
            .checked_add(1)
            .ok_or(GuardianError::ArithmeticOverflow)?;
        let request = serde_json::to_vec(&json!({
            "jsonrpc": "2.0",
            "id": id,
            "method": method,
            "params": params
        }))
        .map_err(|_| GuardianError::Internal)?;

        let mut retried = false;
        loop {
            self.budget.charge_rpc()?;
            let response = self
                .transport
                .post(
                    &self.endpoint,
                    &request,
                    self.timeout_ms,
                    self.budget.limits().max_http_response_bytes,
                )
                .map_err(|_| GuardianError::RpcTransport)?;
            if response.body.len() > self.budget.limits().max_http_response_bytes {
                return Err(GuardianError::RpcHttp);
            }
            if response.status == 429 && !retried {
                retried = true;
                continue;
            }
            if response.status == 429 {
                return Err(GuardianError::RpcRateLimited);
            }
            if !(200..300).contains(&response.status) {
                return Err(GuardianError::RpcHttp);
            }
            return validate_envelope(&response.body, id);
        }
    }

    /// Fetch a confirmed transaction and metadata.
    ///
    /// # Errors
    ///
    /// Propagates validated RPC failures and returns `TransactionNotFound` for
    /// a null result.
    pub fn get_transaction(
        &mut self,
        signature: &str,
        commitment: &str,
    ) -> Result<Value, GuardianError> {
        non_null(
            self.call(
                "getTransaction",
                &json!([
                    signature,
                    {
                        "encoding": "base64",
                        "commitment": commitment,
                        "maxSupportedTransactionVersion": 0
                    }
                ]),
            )?,
            GuardianError::TransactionNotFound,
        )
    }

    /// Simulate a serialized transaction without signature verification.
    ///
    /// # Errors
    ///
    /// Propagates validated RPC and protocol failures.
    pub fn simulate_transaction(
        &mut self,
        transaction_base64: &str,
        commitment: &str,
        replace_recent_blockhash: bool,
    ) -> Result<Value, GuardianError> {
        self.call(
            "simulateTransaction",
            &json!([
                transaction_base64,
                {
                    "encoding": "base64",
                    "sigVerify": false,
                    "innerInstructions": true,
                    "replaceRecentBlockhash": replace_recent_blockhash,
                    "commitment": commitment
                }
            ]),
        )
    }

    /// Fetch at most 100 accounts while preserving null entries and order.
    ///
    /// # Errors
    ///
    /// Returns `InvalidInput` for oversized batches and propagates RPC errors.
    pub fn get_multiple_accounts(
        &mut self,
        addresses: &[String],
        commitment: &str,
        min_context_slot: Option<u64>,
    ) -> Result<Value, GuardianError> {
        if addresses.len() > 100 {
            return Err(GuardianError::invalid_input(
                "getMultipleAccounts accepts at most 100 addresses",
            ));
        }
        self.call(
            "getMultipleAccounts",
            &json!([
                addresses,
                {
                    "encoding": "base64",
                    "commitment": commitment,
                    "minContextSlot": min_context_slot
                }
            ]),
        )
    }

    /// Fetch one account.
    ///
    /// # Errors
    ///
    /// Propagates RPC failures and returns `AccountNotFound` for null.
    pub fn get_account_info(
        &mut self,
        address: &str,
        commitment: &str,
        min_context_slot: Option<u64>,
    ) -> Result<Value, GuardianError> {
        non_null(
            self.call(
                "getAccountInfo",
                &json!([
                    address,
                    {
                        "encoding": "base64",
                        "commitment": commitment,
                        "minContextSlot": min_context_slot
                    }
                ]),
            )?,
            GuardianError::AccountNotFound,
        )
    }

    /// Estimate the base fee for a Base64-encoded message.
    ///
    /// # Errors
    ///
    /// Propagates validated RPC failures.
    pub fn get_fee_for_message(
        &mut self,
        message_base64: &str,
        commitment: &str,
    ) -> Result<Value, GuardianError> {
        self.call(
            "getFeeForMessage",
            &json!([
                message_base64,
                {"commitment": commitment}
            ]),
        )
    }

    #[must_use]
    pub const fn calls_used(&self) -> u16 {
        self.budget.rpc_calls()
    }
}

fn non_null(value: Value, error: GuardianError) -> Result<Value, GuardianError> {
    if value.is_null() {
        Err(error)
    } else {
        Ok(value)
    }
}

fn validate_envelope(bytes: &[u8], expected_id: u64) -> Result<Value, GuardianError> {
    let envelope: Value = serde_json::from_slice(bytes).map_err(|_| GuardianError::RpcProtocol)?;
    let object = envelope.as_object().ok_or(GuardianError::RpcProtocol)?;
    if object.get("jsonrpc").and_then(Value::as_str) != Some("2.0")
        || object.get("id").and_then(Value::as_u64) != Some(expected_id)
    {
        return Err(GuardianError::RpcProtocol);
    }
    if object.contains_key("error") {
        return Err(GuardianError::RpcProtocol);
    }
    object
        .get("result")
        .cloned()
        .ok_or(GuardianError::RpcProtocol)
}

#[cfg(target_family = "wasm")]
pub struct WakiTransport;

#[cfg(target_family = "wasm")]
impl RpcTransport for WakiTransport {
    fn post(
        &mut self,
        endpoint: &str,
        body: &[u8],
        timeout_ms: u64,
        max_response_bytes: usize,
    ) -> Result<HttpResponse, TransportError> {
        use std::time::Duration;

        let response = waki::Client::new()
            .post(endpoint)
            .header("Content-Type", "application/json")
            .connect_timeout(Duration::from_millis(timeout_ms))
            .body(body)
            .send()
            .map_err(|_| TransportError::Other)?;
        let status = response.status_code();
        let body = response.body().map_err(|_| TransportError::Other)?;
        if body.len() > max_response_bytes {
            return Err(TransportError::Other);
        }
        Ok(HttpResponse { status, body })
    }
}

impl fmt::Display for TransportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            Self::Timeout => "transport timeout",
            Self::Other => "transport error",
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::VecDeque;

    use super::{HttpResponse, RpcClient, RpcTransport, TransportError};
    use crate::limits::{Budget, Limits};

    struct MockTransport {
        responses: VecDeque<Result<HttpResponse, TransportError>>,
    }

    impl RpcTransport for MockTransport {
        fn post(
            &mut self,
            _endpoint: &str,
            _body: &[u8],
            _timeout_ms: u64,
            _max_response_bytes: usize,
        ) -> Result<HttpResponse, TransportError> {
            self.responses
                .pop_front()
                .unwrap_or(Err(TransportError::Other))
        }
    }

    fn client(responses: Vec<Result<HttpResponse, TransportError>>) -> RpcClient<MockTransport> {
        let limits = Limits {
            max_rpc_calls: 8,
            max_http_response_bytes: 1024,
            max_transaction_bytes: 1232,
            max_output_bytes: 4096,
            max_accounts: 512,
            max_instructions: 256,
        };
        RpcClient::new(
            "https://rpc.invalid".to_owned(),
            5000,
            MockTransport {
                responses: responses.into(),
            },
            Budget::new(limits),
        )
    }

    #[allow(clippy::unnecessary_wraps)]
    fn response(status: u16, body: &str) -> Result<HttpResponse, TransportError> {
        Ok(HttpResponse {
            status,
            body: body.as_bytes().to_vec(),
        })
    }

    #[test]
    fn validates_success_envelope() {
        let mut rpc = client(vec![response(
            200,
            r#"{"jsonrpc":"2.0","id":1,"result":{"value":7}}"#,
        )]);
        assert_eq!(
            rpc.call("getAccountInfo", &serde_json::json!([])),
            Ok(serde_json::json!({"value": 7}))
        );
    }

    #[test]
    fn rejects_wrong_id_and_rpc_error() {
        let mut wrong_id = client(vec![response(
            200,
            r#"{"jsonrpc":"2.0","id":2,"result":null}"#,
        )]);
        assert!(wrong_id.call("x", &serde_json::json!([])).is_err());
        let mut rpc_error = client(vec![response(
            200,
            r#"{"jsonrpc":"2.0","id":1,"error":{"code":-1}}"#,
        )]);
        assert!(rpc_error.call("x", &serde_json::json!([])).is_err());
    }

    #[test]
    fn retries_429_once() {
        let mut rpc = client(vec![
            response(429, "{}"),
            response(200, r#"{"jsonrpc":"2.0","id":1,"result":"ok"}"#),
        ]);
        assert_eq!(
            rpc.call("x", &serde_json::json!([])),
            Ok(serde_json::json!("ok"))
        );
        assert_eq!(rpc.calls_used(), 2);
    }

    #[test]
    fn rejects_second_429_transport_oversize_and_null() {
        let mut limited = client(vec![response(429, "{}"), response(429, "{}")]);
        assert!(limited.call("x", &serde_json::json!([])).is_err());

        let mut transport = client(vec![Err(TransportError::Timeout)]);
        assert!(transport.call("x", &serde_json::json!([])).is_err());

        let mut oversized = client(vec![Ok(HttpResponse {
            status: 200,
            body: vec![b'x'; 1025],
        })]);
        assert!(oversized.call("x", &serde_json::json!([])).is_err());

        let mut null = client(vec![response(
            200,
            r#"{"jsonrpc":"2.0","id":1,"result":null}"#,
        )]);
        assert!(
            null.get_account_info("11111111111111111111111111111111", "confirmed", None)
                .is_err()
        );
    }

    #[test]
    fn method_contracts_emit_calls_and_batch_limit_is_enforced() {
        let success = |id| {
            response(
                200,
                &format!(r#"{{"jsonrpc":"2.0","id":{id},"result":{{}}}}"#),
            )
        };
        let mut rpc = client(vec![
            success(1),
            success(2),
            success(3),
            success(4),
            success(5),
        ]);
        assert!(rpc.get_transaction("sig", "confirmed").is_ok());
        assert!(rpc.simulate_transaction("AQ==", "processed", true).is_ok());
        assert!(rpc.get_multiple_accounts(&[], "confirmed", None).is_ok());
        assert!(
            rpc.get_account_info("11111111111111111111111111111111", "confirmed", None)
                .is_ok()
        );
        assert!(rpc.get_fee_for_message("AQ==", "confirmed").is_ok());

        let addresses = vec!["1".to_owned(); 101];
        assert!(
            rpc.get_multiple_accounts(&addresses, "confirmed", None)
                .is_err()
        );
    }
}
