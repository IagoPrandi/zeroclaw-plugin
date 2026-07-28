# 02 — Componentes & Classes

Catálogo do que já existe. **Reuse antes de criar.** Classes utilitárias vivem
em `idealer.css`; componentes React vivem em `shared.jsx` (compartilhados) ou no
arquivo da tela. Tudo o que é compartilhado está no `window`.

---

## Classes utilitárias (`idealer.css`)

### `.ide-app`
Raiz de escopo. Define os tokens e a tipografia. Toda tela começa com
`className="ide-app ide-screen"`.

### `.ide-num`
Aplica `--num-font` (Space Grotesk) + dígitos tabulares. **Use em todo número**:
prêmios, contadores, votos, valores, carteiras, IDs de rodada, percentuais.

### `.ide-grad-text`
Texto com gradiente de marca animado (shimmer). Use para o número do prêmio e
valores de destaque. Combine com `.ide-num`.

### `.ide-ambient`
Glows roxo+verde de fundo (pseudo-elementos). Coloque uma vez como primeiro
filho da tela: `<div className="ide-ambient" />`. O conteúdo vai num wrapper com
`position: relative; zIndex: 1` por cima.

### `.ide-scroll`
Container rolável com scrollbar oculta. Use no wrapper de conteúdo das telas
roláveis (`overflowY: auto`).

### `.ide-pill`
Etiqueta arredondada (status/meta). Inline-flex, `gap 6px`, `surface-2` + borda.
Tinja o texto/ícone com a cor semântica via `style={{ color: 'var(--green)' }}`.
```jsx
<span className="ide-pill"><Ico.users style={{color:'var(--purple-soft)'}}/> <span className="ide-num">1.284</span> jogando</span>
```

### `.ide-btn` (primário) e `.ide-btn-ghost` (secundário)
- `.ide-btn`: fundo `--grad-btn`, texto `#08110C`, raio 16, sombra neon. CTA.
- `.ide-btn-ghost`: fundo `--surface-2`, texto `--text`, borda `--line-2`. Secundário.
- Combine: `className="ide-btn-ghost ide-btn"` reaproveita o layout do botão com
  o visual fantasma.
- Largura total: `style={{ width: '100%' }}`. `:active` já dá feedback de toque.
```jsx
<button className="ide-btn" style={{width:'100%', padding:17, fontSize:17}}>
  <Ico.bolt style={{color:'#08110C'}}/> Jogar agora · {fmtSol(TURN.playCost)} USDC
</button>
```
> Ícone dentro de `.ide-btn` deve ter `color:'#08110C'` (contraste no gradiente).

### `.ide-card`
Card padrão: `--surface`, borda `--line`, raio 20. Variações comuns por `style`:
borda colorida + fundo em `linear-gradient` translúcido para "destaque"
(dourado para vitória, verde para sucesso, roxo para info).

### `.ide-tap`
Adiciona feedback de escala ao tocar. Use em linhas/cards clicáveis.

### `.ide-stat`
Tile de estatística (`--surface`, raio 16, padding 14). Base do `StatTile`.

### `.ide-nav` / `.ide-nav-btn` / `.ide-nav-fab`
Barra inferior fixa, botões de aba e o FAB central (gradiente). Renderizada pelo
`App` (em `app.jsx`), não dentro das telas. Botão ativo recebe classe `on`.

### `.ide-numgrid` / `.ide-numcell`
Grade 6 colunas de números (1–60) e a célula quadrada. Selecionada → classe
`sel` (gradiente + `ide-pop`).

### `.ide-seg`
Segmented control (abas). Botão ativo → classe `on` (`--surface-3`).
```jsx
<div className="ide-seg">
  <button className={tab==='a'?'on':''} onClick={()=>setTab('a')}>Aba A</button>
  <button className={tab==='b'?'on':''} onClick={()=>setTab('b')}>Aba B</button>
</div>
```

### `.ide-bubble-ai` / `.ide-bubble-user` / `.ide-typing`
Balões de chat (IA à esquerda em `--surface`; usuário à direita em gradiente
roxo→verde) e o indicador de digitação de 3 pontos.

### Modificadores de Tweak
`.ide-noglow` (mata pulsos de brilho) e `.ide-calm` (mata animações). São
alternados no shell pelo `App`. Não defina à mão; apenas não os contrarie.

---

## Componentes React (`shared.jsx`)

### `Ico` — biblioteca de ícones (objeto de funções SVG)
Stroke `currentColor`, `strokeWidth ~2`, sem fill (salvo `bolt`, `spark`,
`trophy` parcial, `dice`). Passe `style={{color:'…'}}` para tingir e `width/
height` se precisar de outro tamanho. Disponíveis:

`arrow, wallet, bolt, up, chevron, close, clock, users, spark, trophy, send,
home, user, plus, back, check, dice, withdraw, copy, gift, history`

```jsx
<Ico.trophy style={{color:'var(--gold)', width:18, height:18}} />
```
**Precisa de um ícone novo?** Adicione uma entrada em `Ico` no mesmo estilo
(viewBox 0 0 24 24, stroke currentColor, linecap/linejoin round). Não cole SVGs
soltos pelas telas.

### `Dealer({ size })`
Mascote croupiê (SVG, gradiente de marca). Use em estados de espera/conexão,
confirmação e cabeçalho do chat. `size` 30–72.

### `Medal({ pos })`
Selo de posição. 1/2/3 → ouro/prata/bronze com glow; demais → número apagado.
Usado em rankings/listas de regras.

### `RuleDetail({ rule, rank, onClose })`
Bottom-sheet de detalhe de uma regra (título, resumo, "Como funciona", prompt
seed, metadados). Padrão de **overlay**: full-screen com backdrop `blur`, sheet
de baixo com `borderTopRadius 28`, alça (grabber) no topo, fecha no backdrop ou
no X. Use este padrão para qualquer novo modal/sheet.

### `SectionLabel({ children })`
Rótulo de seção maiúsculo, `--dim`, `letter-spacing .08em`. Antecede grupos.

### `Meta({ label, value, mono })`
Mini-card rótulo+valor para grades de metadados (2 colunas). `mono` aplica `.ide-num`.

---

## Componentes de tela (definidos nas telas, exportados no window)

| Componente | Arquivo | Papel |
|---|---|---|
| `ScreenHead({ go, title, right })` | `app-play.jsx` | cabeçalho com botão voltar + título (+ slot direito) |
| `Confetti()` | `app-play.jsx` | chuva de confete para vitórias |
| `WalletBtn`, `PromptInput`, `VoteBarRow`, `PagBtn` | `app-home.jsx` | botão de carteira, input de prompt, linha de barra de voto, paginação |
| `RulePreview({ rule })` | `app-create.jsx` | card de pré-visualização da regra gerada + medidor de tokens |
| `StatTile({ icon, n, unit, l })` | `app-profile.jsx` | tile de estatística |

`ScreenHead` é o cabeçalho canônico de telas internas — **use sempre** em telas
novas que não sejam a home:
```jsx
<ScreenHead go={go} title="Minha tela" />
```

---

## Helpers & hooks (`shared.jsx`)

- `fmtSol(n)` → string com 2 casas, locale pt-BR. Use para **todo valor USDC**.
- `fmtBrl(n)` → inteiro pt-BR (para equivalência em R$, se exibida).
- `fmtCountdown(totalSeg)` → `['hh','mm','ss']`.
- `useCountdown(segundos)` → contador regressivo ao vivo (array hh/mm/ss).
- `useLivePrize(alvo)` → número que sobe suavemente até o alvo (efeito de prêmio
  acumulando). Baseado em `setInterval`.

## Dados mock (`shared.jsx`) — **fonte única de verdade**
`TURN` (rodada atual), `RULES` (13 regras com `seed`/`how`), `USER`,
`LAST_ROUND` (rodada encerrada → home banner + tela de resultado), `HISTORY`
(24 jogos), `MY_RULES`. **Ao adicionar conteúdo, estenda esses objetos** em vez
de criar arrays soltos na tela — mantém o protótipo coerente.
