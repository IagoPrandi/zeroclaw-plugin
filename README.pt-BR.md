# Solana Transaction Guardian

O Solana Transaction Guardian é um plugin read-only para o ZeroClaw que
decodifica, simula e avalia transações Solana antes que um agente ou usuário
confie nelas. Ele aceita uma transação serializada ou uma assinatura confirmada
e retorna um relatório JSON determinístico com ações, participantes, deltas de
saldo, taxas, cobertura, findings e uma decisão `allow`, `review` ou `block`.

O plugin Rust/WASM é a autoridade de segurança. O modelo local `qwen3.5:9b`
apenas seleciona o tool, monta os argumentos e apresenta o relatório; ele não
pode alterar a decisão canônica.

> Custódia T0: o Guardian não possui chave privada nem capacidade de assinar ou
> enviar transações.

[English](README.md)

## Por que ele existe

Uma transação pode parecer um pagamento e, ao mesmo tempo, aprovar um delegate,
alterar uma authority, chamar um programa desconhecido ou gastar além da
intenção declarada. Exploradores explicam o histórico confirmado. O Guardian
também analisa candidatas antes da assinatura, compara o comportamento com a
intenção estruturada, aplica a política do operador e falha de forma
conservadora quando a evidência é incompleta.

## Cobertura

- transações legacy e versão 0, incluindo Address Lookup Tables;
- instruções System, Compute Budget, SPL Token e Token-2022;
- simulação ou efeitos confirmados, instruções internas, taxas, logs, return
  data e deltas de SOL/tokens;
- política e intenção determinísticas com 54 rule IDs estáveis;
- cobertura, confiança e erros controlados explícitos;
- exatamente um tool ZeroClaw: `solana_transaction_guardian`.

Consulte [regras de risco](docs/RISK_RULES.md),
[arquitetura](docs/ARCHITECTURE.md) e
[limitações](docs/LIMITATIONS.md).

## Execução local

```text
Usuário
  -> ZeroClaw v0.8.3
     -> Ollama 0.32.0 local / qwen3.5:9b
        -> solana_transaction_guardian (WASM)
           -> JSON-RPC Solana configurado
           -> relatório determinístico
        -> apresentação fiel do relatório
```

A configuração de referência não possui provider de LLM cloud nem fallback.
Se o Ollama ou o modelo fixado estiver indisponível, o fluxo é interrompido com
um erro acionável.

## Início rápido

Pré-requisitos:

- ZeroClaw v0.8.3 no commit
  `24476b71d33eb1672a9495a7ce3d155377a60ce8`;
- Ollama 0.32.0 em `127.0.0.1:11434`;
- `qwen3.5:9b` com digest
  `6488c96fa5faab64bb65cbd30d4289e20e6130ef535a93ef9a49f42eda893ea7`;
- o arquivo da release ou Rust 1.96.1 com target `wasm32-wasip2`.

```bash
ollama pull qwen3.5:9b
ollama list
```

Extraia a release e instale o diretório do plugin:

```bash
zeroclaw plugin install ./solana-transaction-guardian-0.1.0 \
  --config-dir /caminho/para/perfil-guardian
zeroclaw plugin list --config-dir /caminho/para/perfil-guardian
```

Copie
[config/zeroclaw.guardian.example.toml](config/zeroclaw.guardian.example.toml)
como `config.toml` do perfil, confira a chave do publisher em
[docs/PUBLISHER_KEY.md](docs/PUBLISHER_KEY.md) e copie
[prompts/GUARDIAN_SYSTEM.md](prompts/GUARDIAN_SYSTEM.md) para
`agents/guardian/workspace/SOUL.md`. Depois execute:

```bash
zeroclaw agent --agent guardian \
  --config-dir /caminho/para/perfil-guardian \
  --message "Analise esta transação devnet: <BASE64>"
```

O [guia de instalação](docs/INSTALLATION.md) inclui build do source,
verificação strict da assinatura, comandos por plataforma e hashes. A
[referência de configuração](docs/CONFIGURATION.md) descreve política e
limites.

## Segurança e evidências

A release candidate passou por 60 testes nativos, build WASI reproduzível,
limites reais do host, assinatura strict, scanners de segurança, 30 conversas
controladas com o Qwen e 20 análises devnet com p95 de 1.653 ms sob orçamento
de seis RPCs. Consulte [a matriz de testes](docs/TEST_MATRIX.md).

O relatório é consultivo. Simulação e estado RPC são observações pontuais,
protocolos desconhecidos podem reduzir a cobertura e uma assinatura realizada
fora do ZeroClaw não pode ser impedida pelo plugin. Mantenha
`fail_closed=true`, use assinatura strict, confira os SHA-256 publicados e
nunca forneça seed ou chave privada.

O projeto usa licença MIT. Atribuições estão em
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).
