# 04 — Receita: adicionar uma tela ou feature

Passo a passo para estender o Idealer **sem quebrar o padrão**. Siga na ordem.

---

## A. Nova tela

### 1. Crie o arquivo `app-<nome>.jsx`
Comece pelo esqueleto canônico (ver `03-screens-and-state.md`):

```jsx
/* ============================================================
   <NOME> — descrição curta em pt-BR
   ============================================================ */
function ScreenNome({ go, st, setSt }) {
  return (
    <div className="ide-app ide-screen" style={{ height:"100%", position:"relative", overflow:"hidden" }}>
      <div className="ide-ambient" />
      <div className="ide-scroll" style={{ position:"relative", zIndex:1, height:"100%", overflowY:"auto", padding:"52px 18px 96px" }}>
        <ScreenHead go={go} title="Minha tela" />
        {/* conteúdo usando .ide-card, .ide-pill, Ico, SectionLabel... */}
      </div>
    </div>
  );
}
Object.assign(window, { ScreenNome });
```

- Reuse `ScreenHead`, `.ide-card`, `.ide-pill`, `Ico`, `SectionLabel`, `Medal`,
  `StatTile`, `Dealer`. Não escreva markup novo para algo que já existe.
- Números em `.ide-num`; valores via `fmtSol`. Cores só com `var(--…)`.
- Trate gates (`st.connected`, `st.played`) se a tela exigir.

### 2. Registre o `<script>` em `Idealer.html`
Adicione **antes** de `app.jsx`:
```html
<script type="text/babel" src="app-nome.jsx"></script>
```

### 3. Plugue na navegação em `app.jsx`
```jsx
{screen === "nome" && <ScreenNome go={go} st={st} setSt={setSt} />}
```
- Para chegar nela: chame `go("nome")` de um botão.
- Se ela deve aparecer na **bottom nav**, edite o bloco `showNav` e os botões
  da `.ide-nav`. Se for tela interna, use `ScreenHead` (voltar) e **não** a nav.

### 4. Dados
Se precisa de dados mock, **estenda os objetos em `shared.jsx`** (`TURN`,
`USER`, `RULES`, `HISTORY`, etc.) e exporte. Não crie arrays soltos na tela.

---

## B. Novo componente reutilizável

- **Usado por 1 tela só:** defina no arquivo da tela e exporte no `window`.
- **Usado por várias telas / genérico:** coloque em `shared.jsx` e exporte no
  `Object.assign(window, …)` final.
- Siga o estilo: props simples, estilos inline + classes `.ide-*`, tokens via
  var. Para um modal/sheet, copie o padrão de `RuleDetail`.

---

## C. Novo ícone
Adicione uma entrada em `Ico` (em `shared.jsx`), mesmo padrão dos demais:
`viewBox="0 0 24 24"`, `fill="none"`, `stroke="currentColor"`, `strokeWidth`
~2–2.4, `strokeLinecap/Linejoin="round"`, espalhe `{...p}`. Tinja no uso com
`style={{color:'var(--…)'}}`. Nunca cole SVG inline solto nas telas.

---

## D. Novo Tweak (controle visual)
1. Some a chave em `TWEAK_DEFAULTS` (dentro dos marcadores `EDITMODE`).
2. Aplique no `useEffect` que escreve nas CSS vars de `#phone-shell`.
3. Adicione o controle no `<TweaksPanel>` (`TweakColor`/`TweakRadio`/
   `TweakToggle`/`TweakSection`), com label em pt-BR.

---

## E. Nova animação
Defina o `@keyframes` em `idealer.css` com prefixo `ide-`. **Entrada deve ser
transform-only** (sem `opacity:0` inicial). Garanta que `.ide-calm` consiga
neutralizá-la (não use `!important` em duração). Para brilho pulsante, garanta
que `.ide-noglow` o desligue.

---

## Checklist de PR (revise antes de entregar)

- [ ] Tela é `<div className="ide-app ide-screen">` com `.ide-ambient` + `.ide-scroll`.
- [ ] `ScreenHead` em telas internas; bottom nav só onde faz sentido.
- [ ] Todo número usa `.ide-num`; todo valor usa `fmtSol`; moeda exibida = USDC.
- [ ] Nenhuma cor hard-coded — só `var(--…)`. Texto sobre gradiente = `#08110C`.
- [ ] Reusei classes/componentes existentes em vez de recriar.
- [ ] Layout com flex/grid + `gap`; hit targets ≥ 44px; corpo ≥ 13.5px.
- [ ] Exportei no `window`; registrei o `<script>`; pluguei a navegação.
- [ ] Animações transform-only; respeitam `.ide-calm` / `.ide-noglow`.
- [ ] HTML canônico; sem `scrollIntoView`; UI em pt-BR.
- [ ] Abri `Idealer.html` no preview: monta, sem erros de console, Tweaks aplicam.

---

## Exemplo completo (mini-tela "Como funciona")

```jsx
/* COMO FUNCIONA — explica o ciclo da rodada */
function ScreenHowto({ go }) {
  const passos = [
    { ico: "bolt",   t: "Jogue",        d: `Escolha 6 números por ${fmtSol(TURN.playCost)} USDC.` },
    { ico: "spark",  t: "Crie regras",  d: "Descreva por prompt como sua regra vence." },
    { ico: "up",     t: "Vote",         d: "A comunidade vota as regras da rodada." },
    { ico: "trophy", t: "Ganhe",        d: `Criador da regra vencedora leva ${TURN.creatorRewardPct}%.` },
  ];
  return (
    <div className="ide-app ide-screen" style={{ height:"100%", position:"relative", overflow:"hidden" }}>
      <div className="ide-ambient" />
      <div className="ide-scroll" style={{ position:"relative", zIndex:1, height:"100%", overflowY:"auto", padding:"52px 18px 96px" }}>
        <ScreenHead go={go} title="Como funciona" />
        <div style={{ display:"grid", gap:10 }}>
          {passos.map((p, i) => (
            <div key={i} className="ide-card" style={{ padding:"14px 16px", display:"flex", alignItems:"center", gap:13 }}>
              <span style={{ width:40, height:40, borderRadius:13, background:"var(--surface-2)", border:"1px solid var(--line)", display:"grid", placeItems:"center", color:"var(--purple-soft)", flexShrink:0 }}>
                {Ico[p.ico]({})}
              </span>
              <div style={{ flex:1, minWidth:0 }}>
                <div style={{ fontWeight:700, fontSize:15.5 }}>{p.t}</div>
                <div style={{ fontSize:13, color:"var(--muted)" }}>{p.d}</div>
              </div>
              <span className="ide-num" style={{ color:"var(--dim)", fontWeight:700 }}>{i + 1}</span>
            </div>
          ))}
        </div>
        <button onClick={() => go("home")} className="ide-btn" style={{ width:"100%", marginTop:18, padding:15 }}>Entendi</button>
      </div>
    </div>
  );
}
Object.assign(window, { ScreenHowto });
```
Depois: registre `app-howto.jsx` em `Idealer.html` e adicione
`{screen === "howto" && <ScreenHowto go={go} />}` no `App`.
