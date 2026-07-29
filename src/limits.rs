use crate::error::GuardianError;

#[derive(Clone, Debug)]
pub struct Limits {
    pub max_rpc_calls: u16,
    pub max_http_response_bytes: usize,
    pub max_transaction_bytes: usize,
    pub max_output_bytes: usize,
    pub max_accounts: usize,
    pub max_instructions: usize,
}

#[derive(Debug)]
pub struct Budget {
    limits: Limits,
    rpc_calls: u16,
}

impl Budget {
    #[must_use]
    pub const fn new(limits: Limits) -> Self {
        Self {
            limits,
            rpc_calls: 0,
        }
    }

    /// Consume one RPC call from the per-analysis budget.
    ///
    /// # Errors
    ///
    /// Returns an error when the counter overflows or the configured budget is
    /// exhausted.
    pub fn charge_rpc(&mut self) -> Result<u16, GuardianError> {
        self.rpc_calls = self
            .rpc_calls
            .checked_add(1)
            .ok_or(GuardianError::ArithmeticOverflow)?;
        if self.rpc_calls > self.limits.max_rpc_calls {
            return Err(GuardianError::RpcTransport);
        }
        Ok(self.rpc_calls)
    }

    #[must_use]
    pub const fn limits(&self) -> &Limits {
        &self.limits
    }

    #[must_use]
    pub const fn rpc_calls(&self) -> u16 {
        self.rpc_calls
    }
}
