# RPC contract

The plugin obtains its endpoint only from host-injected configuration. Tool
arguments cannot select or override an endpoint. Production endpoints require
HTTPS; plain HTTP is accepted only for explicit localhost tests.

Every call uses JSON-RPC 2.0 over POST, a monotonic local ID, a timeout, an
execution-wide call budget, and a response-size limit. The client validates the
HTTP status, envelope version, matching ID, mutually exclusive result/error,
and required result. HTTP 429 receives one bounded retry; no other implicit
retry occurs.

Supported methods are `getTransaction`, `simulateTransaction`,
`getMultipleAccounts`, `getAccountInfo`, and `getFeeForMessage`.
`getMultipleAccounts` accepts at most 100 addresses and preserves response
ordering and null entries. Related state calls carry `minContextSlot` when one
is known.

Logs may contain method, duration, status, response byte count, and analysis ID.
They must not contain complete requests, transactions, account data, RPC
credentials, or host-injected configuration.
