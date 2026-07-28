# 03 — Telas, Estado & Navegação

## O App shell (`app.jsx`)

`App` é a raiz. Ele mantém **dois estados centrais**:

```js
const [t, setTweak] = useTweaks(TWEAK_DEFAULTS);   // tweaks (visual)
const [st, setSt]   = useState({                    // estado do produto
  connected: false,        // carteira conectada?
  played: false,           // já jogou nesta rodada? (libera criar regra)
  bet: null,               // array de 6 números escolhidos
  seed: "",                // prompt vindo da home → pré-preenche o chat
  rulesCreated: USER.rulesCreated,
});
const [screen, setScreen] = useState(LAST_ROUND.youPlayed ? "result" : "home");
```

### Navegação
Não há router. A tela atual é a string `screen`. A função `go` troca de tela e
opcionalmente mescla parâmetros no estado:

```js
const go = (s, params = {}) => { setSt(cur => ({ ...cur, ...params })); setScreen(s); };
```

Telas existentes: `home`, `play`, `create`, `profile`, `result`. Cada uma é
renderizada condicionalmente e recebe `{ go, st, setSt }` (a `result` recebe só
`{ go, st }`).

```jsx
{screen === "home"    && <ScreenHome    go={go} st={st} setSt={setSt} />}
{screen === "play"    && <ScreenPlay    go={go} st={st} setSt={setSt} />}
{screen === "create"  && <ScreenCreate  go={go} st={st} setSt={setSt} />}
{screen === "profile" && <ScreenProfile go={go} st={st} setSt={setSt} />}
{screen === "result"  && <ScreenResult  go={go} st={st} />}
```

### Bottom nav
Renderizada pelo `App`, **só nas telas `home` e `profile`**:
```js
const showNav = ["home", "profile"].includes(screen);
```
Três alvos: Rodada (home), FAB central → `create`, Conta (profile). Para
adicionar um destino à nav, edite esse bloco no `App`. Telas internas
(`play`, `create`, `result`) **não** mostram nav — usam `ScreenHead` com voltar.

### Tweaks (painel visual)
`TWEAK_DEFAULTS` está demarcado por `/*EDITMODE-BEGIN*/ … /*EDITMODE-END*/` — não
remova esses marcadores. Um `useEffect` aplica os tweaks às CSS vars do shell
(`#phone-shell`): paleta de acento, cor do topo do ranking, fonte dos números,
brilho do prêmio (`ide-noglow`) e modo de animação (`ide-calm`).

O `<TweaksPanel>` (componente starter, `tweaks-panel.jsx`) fica no fim do `App`.
Controles em uso: `TweakColor`, `TweakRadio`, `TweakToggle`, agrupados por
`TweakSection`. **Ao adicionar um tweak novo:** (1) some uma chave em
`TWEAK_DEFAULTS`, (2) aplique no `useEffect`, (3) adicione o controle no painel.

---

## Anatomia de uma tela

Toda tela segue este esqueleto. As três variantes de padding-bottom dependem do
que a tela tem embaixo.

```jsx
function ScreenX({ go, st, setSt }) {
  return (
    <div className="ide-app ide-screen" style={{ height:"100%", position:"relative", overflow:"hidden" }}>
      <div className="ide-ambient" />
      <div className="ide-scroll" style={{
        position:"relative", zIndex:1, height:"100%", overflowY:"auto",
        padding:"52px 18px 96px"   // 52 topo (status bar) · 18 lados · base ↓
      }}>
        <ScreenHead go={go} title="Título" />
        {/* ...conteúdo... */}
      </div>
      {/* overlays/sheets como irmãos, zIndex ≥ 60 */}
    </div>
  );
}
Object.assign(window, { ScreenX });
```

**Padding-bottom por caso:**
- `96px` → tela com bottom nav (home, profile).
- `150px` → tela com barra de ação fixa embaixo (ver `play`).
- `30px` → tela centralizada sem rolagem (estados de conexão/confirmação usam
  `display:flex; justifyContent:center` em vez de `.ide-scroll`).

### Padrões de estado dentro das telas
- **Gate de conexão:** se `!st.connected`, mostrar tela centralizada com
  `Dealer`, headline e botão "Conectar carteira" que faz
  `setSt(s => ({...s, connected:true}))`. Ver `ScreenCreate`/`ScreenProfile`.
- **Gate de jogo:** criar regra exige `st.played`. Se não jogou, mostrar card
  âmbar (dourado) convidando a jogar antes. Ver `app-create.jsx`.
- **Sucesso/confirmação:** tela centralizada com `Dealer` em `ide-pop`,
  `Confetti` quando for vitória, fichas dos números em gradiente.
- **Barra de ação fixa:** `position:absolute; bottom:0` com fundo
  `linear-gradient(180deg, transparent, var(--bg) 36%)` e o `.ide-btn` de
  confirmação. Botão desabilitado: `opacity:0.4; pointerEvents:none`.

### Overlays / sheets
Use o padrão de `RuleDetail`: irmão da tela, `position:absolute; inset:0;
zIndex:60`, backdrop `rgba(6,4,10,.62)` + `backdropFilter: blur(6px)`, sheet
ancorado embaixo com alça e botão fechar. Fecha no clique do backdrop
(`onClick={onClose}`) com `stopPropagation` no conteúdo.

---

## Listas, ranking e paginação

- **Ranking de regras (home):** ordena por `votes` desc, top-3 com `Medal`,
  barra de progresso proporcional (`width: pct%`) atrás do conteúdo. Demais
  páginas em blocos de 10 com `PagBtn`.
- **Histórico (profile):** paginação por rolagem — carrega +10 ao chegar perto
  do fim (`onScroll`).
- Linhas longas: trunque com `overflow:hidden; textOverflow:ellipsis;
  whiteSpace:nowrap`.

## Convenções de código
- `const { useState, useEffect, useRef, useCallback } = React;` já vem via
  `shared.jsx` (re-exportado no window) — não re-declare.
- Sempre `Object.assign(window, { ... })` no fim do arquivo com tudo que outra
  parte usa.
- Comentários e strings de UI em **pt-BR**.
- Nada de `type="module"`. Nada de `scrollIntoView` (use `el.scrollTop`).
