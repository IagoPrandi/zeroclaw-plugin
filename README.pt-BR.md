# Solana Transaction Guardian

O Solana Transaction Guardian é um plugin read-only para o ZeroClaw que
decodifica, simula e avalia transações Solana antes que um agente ou usuário
confie nelas. Ele aceita uma transação serializada ou uma assinatura confirmada
e retorna um relatório JSON determinístico com ações, participantes, deltas de
saldo, taxas, cobertura, findings e uma decisão `allow`, `review` ou `block`.

O plugin Rust/WASM é a autoridade de segurança. Ele funciona com o agente e o
modelo que o usuário já escolheu no ZeroClaw: o modelo pode selecionar o tool,
montar os argumentos e apresentar o relatório, mas não pode alterar a decisão
canônica.

> Custódia T0: o Guardian não possui chave privada nem capacidade de assinar ou
> enviar transações.

[English](README.md)

[Baixar v0.1.0](https://github.com/IagoPrandi/zeroclaw-plugin/releases/tag/v0.1.0)
·
[Assistir ao walkthrough de 2:46 com telefone e terminal](https://github.com/IagoPrandi/zeroclaw-plugin/releases/download/v0.1.0/guardian-demo-walkthrough.mp4)

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

## Modelo de execução

```text
Usuário
  -> ZeroClaw v0.8.3
     -> modelo/provider configurado pelo usuário
        -> solana_transaction_guardian (WASM)
           -> RPC Solana devnet padrão ou configurado pelo operador
           -> relatório determinístico
        -> apresentação fiel do relatório
```

O Guardian não seleciona nem configura um provider de LLM. Ollama/Qwen é
somente o ambiente de referência reproduzível; não é um pré-requisito. Veja
[as evidências do runtime](docs/LLM_RUNTIME.md).

## Início rápido

Pré-requisitos: um perfil ZeroClaw funcional com o modelo/provider escolhido
pelo usuário e uma release compatível do Guardian. Extraia o arquivo e rode o
instalador incluído:

```powershell
.\solana-transaction-guardian-<VERSION>\install-guardian.ps1 `
  -PluginPath .\solana-transaction-guardian-<VERSION>
```

Não é necessário configurar LLM, copiar prompt ou criar perfil Guardian. O
padrão está pronto para análise devnet fail-closed. Em seguida, use o agente
que o usuário já possui:

```bash
zeroclaw agent --agent <seu-agente> \
  --message "Analise esta transação devnet: <BASE64>"
```

O [guia de instalação](docs/INSTALLATION.md) inclui assinaturas strict, build
do source e perfis alternativos. Mainnet, RPC privado e políticas
personalizadas são opt-in e estão em
[CONFIGURATION.md](docs/CONFIGURATION.md).

## Segurança e evidências

A release v0.1.0 passou por 60 testes nativos, build WASI reproduzível,
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
