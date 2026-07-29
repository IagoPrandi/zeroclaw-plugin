# Reference hardware observation

- Date: 2026-07-27
- OS: Windows 11 Home Single Language, build 10.0.26200
- CPU: Intel Core Ultra 5 125H, 18 logical processors
- Installed RAM: 15.5 GiB
- Ollama: 0.32.0
- Model: `qwen3.5:9b`, digest prefix `6488c96fa5fa`
- Model storage reported by Ollama: 6.6 GB
- Loaded processor: 100% CPU
- Context used for the compatibility probe: 8,192 tokens
- Native tool-call cold-run time: approximately 57 seconds

The reference machine can run the required model locally, but CPU-only cold
responses are slow. The observed practical minimum is 16 GiB RAM and 8 GB free
storage for the model plus runtime overhead. More RAM and a supported GPU are
recommended for the demo. The Guardian system prompt and tool contract are
designed to stay comfortably inside an 8,192-token context.
