# Phone-and-terminal walkthrough

Target duration: 2 minutes 46 seconds. The phone side is a labelled
reconstruction of sanitized, recorded agent exchanges; the terminal side
contains the corresponding deterministic Guardian evidence. This keeps the
video legible without claiming a live third-party channel integration.

| Time | Phone view | Terminal evidence |
|---|---|---|
| 0:00-0:15 | Operator asks what the Guardian can do. | Local Ollama, signed plugin, and T0/read-only boundary. |
| 0:15-0:48 | Transparent one-lamport transfer. | System transfer, full decoding coverage, simulation, `allow`. |
| 0:48-1:28 | Payment-shaped request with hidden delegate. | SPL Token approval, policy rules, `block`. |
| 1:28-1:58 | System-only intent against an unknown program. | Incomplete coverage and fail-closed `block`. |
| 1:58-2:25 | Version-0 transaction question. | ALT resolution and decoded transfer. |
| 2:25-2:46 | What to trust before signing. | Tests, model-behavior checks, signed release, repository. |

The video must keep one user question, one Guardian response, and the exact
supporting terminal evidence on screen at a time. Do not add slides, raw
transaction bytes, keys, credentials, or claims that the reconstruction is a
live channel capture.
