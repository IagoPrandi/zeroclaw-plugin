---
name: idealer-design-system
description: >
  Sistema de design e convenções de engenharia do app Idealer — um protótipo
  mobile (React + Babel via CDN, sem build) de um jogo on-chain de loteria com
  regras criadas por IA. Use esta skill SEMPRE que for criar, editar ou estender
  telas, componentes ou fluxos do Idealer, para que tudo siga o padrão visual
  (escuro premium, neon sutil roxo→verde), a tipografia, os tokens e a
  arquitetura de telas já existentes. Acione-a ao receber pedidos como
  "adicione uma tela de…", "crie um componente de…", "novo fluxo de…" no Idealer.
---

# Idealer — Design System & Guia de Engenharia

> Documento-fonte para continuar construindo o Idealer **sem sair do padrão**.
> Leia este arquivo primeiro; depois abra o arquivo de referência relevante.

## O que é o Idealer

App mobile (protótipo) de um **jogo de loteria on-chain** onde a comunidade
**cria e vota as regras de vitória** de cada rodada usando linguagem natural /
IA. Conceitos centrais do produto:

- **Rodada (TURN):** janela de tempo com prêmio acumulado em USDC, contagem
  regressiva, nº de jogadores.
- **Jogar:** escolher 6 números de 1–60 (custo `playCost`). Jogar uma vez na
  rodada **libera** a criação de regras.
- **Criar regra:** conversar com o "Croupiê IA" (chat) e descrever, por prompt,
  como sua regra decide o vencedor. Tem custo (`ruleCost`) e limite de tokens.
- **Votação:** as regras são votadas; a mais votada é sorteada (VRF) e aplicada.
- **Resultado:** números sorteados, regra vencedora, ganhos creditados.
- **Perfil/Conta:** carteira, regras criadas, ganhos, saque, histórico.
- **Recompensa do criador:** quem criou a regra vencedora ganha `creatorRewardPct`
  do prêmio.

Idioma da interface: **português do Brasil**. Tom: direto, animado, levemente
lúdico (croupiê, "Surpresinha", emojis pontuais 🎲🏆🎉 — usar com parcimônia).

## Stack & arquitetura (sem build)

- **React 18.3.1 + ReactDOM + Babel Standalone**, carregados por `<script>` com
  versões fixas e `integrity` (ver `Idealer.html`). Não trocar versões.
- **Sem bundler, sem JSX modules.** Cada arquivo é um `<script type="text/babel">`
  separado. Eles **não compartilham escopo** automaticamente.
- Para expor algo a outros arquivos, anexe ao `window` no fim do arquivo:
  ```js
  Object.assign(window, { ScreenX, ComponenteY });
  ```
- `shared.jsx` exporta dados mock, ícones, hooks e overlays comuns para o
  `window` (e re-exporta `useState`, `useEffect`, etc. — por isso os outros
  arquivos os usam sem declarar).
- O CSS é **uma folha única** `idealer.css`, escopada sob `.ide-app` com prefixo
  de classe `ide-`. Tokens são **CSS custom properties** em `.ide-app`.

### Estrutura de arquivos

```
Idealer.html        ← shell: device frame iOS, fontes, ordem dos <script>
idealer.css         ← tokens (CSS vars), classes utilitárias .ide-*, animações
shared.jsx          ← DADOS mock + ÍCONES (Ico) + hooks + Dealer/Medal/RuleDetail
tweaks-panel.jsx    ← componente starter do painel de Tweaks (não editar à toa)
app.jsx             ← App shell: estado global, navegação, bottom nav, Tweaks
app-home.jsx        ← tela "Rodada" (home conectada)
app-play.jsx        ← tela "Jogar" (grade de números + confirmação)
app-create.jsx      ← tela "Criar regra" (chat com Croupiê IA)
app-profile.jsx     ← tela "Conta" (carteira, saque, regras, histórico)
app-result.jsx      ← tela "Resultado" da rodada anterior
```

**Ordem de carregamento dos scripts importa** (em `Idealer.html`): shared →
tweaks-panel → telas → app.jsx (que faz o `render`). Ao criar um novo arquivo de
tela, adicione seu `<script type="text/babel" src="...">` **antes** de `app.jsx`.

## Arquivos de referência (leia conforme a tarefa)

| Tarefa | Leia |
|---|---|
| Cores, fontes, espaçamento, raios, sombras, animações | `references/01-tokens.md` |
| Botões, cards, pills, nav, grade de números, chat, etc. | `references/02-components.md` |
| Como as telas montam, estado global, navegação, Tweaks | `references/03-screens-and-state.md` |
| **Passo a passo para adicionar uma tela ou feature nova** | `references/04-new-screen-recipe.md` |

## Regras de ouro (não negociáveis)

1. **Nunca invente cores.** Use as CSS vars (`var(--purple)`, `var(--green)`,
   `var(--surface)`, …). Se precisar de um tom novo, derive em `oklch` mantendo
   o croma e leveza dos vizinhos — mas prefira sempre os tokens existentes.
2. **Tipografia:** `Outfit` para UI/corpo; `Space Grotesk` (classe `.ide-num`)
   para **todos os números** (prêmios, contadores, votos, carteiras, valores).
   `letter-spacing: -0.01em` no corpo, `-0.02em` em títulos/números.
3. **Moeda exibida = `USDC`** (apesar de variáveis legadas chamadas `prizeSol`/
   `fmtSol`). Sempre formate via `fmtSol(n)` (2 casas) e `fmtBrl(n)`.
4. **Animações de entrada são transform-only** (sem `opacity:0` inicial) para
   sobreviver a iframes sem foco / captura. Use `.ide-screen`, `.ide-rise`,
   `ide-pop`. Respeite os modos de Tweak `ide-calm`/`ide-noglow`.
5. **Reuse, não recrie.** Antes de escrever markup novo, procure a classe
   `.ide-*` ou o componente (`Ico`, `Dealer`, `Medal`, `RuleDetail`,
   `ScreenHead`, `SectionLabel`, `Meta`, `StatTile`) que já resolve.
6. **Layout sempre com flex/grid + `gap`.** Nada de margens soltas entre irmãos.
7. **Toda tela** é um `<div className="ide-app ide-screen">` com `<div
   className="ide-ambient" />` ao fundo e um `.ide-scroll` rolável por dentro.
   Padding-top ≈ `52px` (status bar), padding-bottom ≈ `96px` se houver bottom
   nav, ou `150px` se houver barra fixa de ação.
8. **Hit targets ≥ 44px.** Texto nunca < 11px (labels), corpo ≥ 13.5px.
9. Ao adicionar uma tela nova, **exporte-a no `window`**, **registre o
   `<script>`** em `Idealer.html` e **plugue na navegação** em `app.jsx`.
10. **HTML canônico:** feche todo elemento, aspas duplas em todo atributo, nada
    de auto-fechar não-void. Não use `scrollIntoView`.

## Verificação

Após mudar algo, abra `Idealer.html` no preview e confirme: sem erros de
console, a tela monta, números em `Space Grotesk`, gradiente roxo→verde
correto, bottom nav só em "home"/"profile", e os Tweaks ainda aplicam.
