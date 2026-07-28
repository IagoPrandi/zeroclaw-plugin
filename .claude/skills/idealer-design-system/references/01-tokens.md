# 01 — Tokens de Design

Todos os tokens vivem como **CSS custom properties** em `.ide-app` (arquivo
`idealer.css`). Use sempre `var(--token)`. Alguns são sobrescritos em runtime
pelos Tweaks (ver `app.jsx`) — por isso **referencie a var, nunca o hex literal**.

## Cores

### Superfícies (do mais escuro ao mais claro)
| Token | Hex | Uso |
|---|---|---|
| `--bg` | `#0B0910` | fundo base do app |
| `--bg-2` | `#100C18` | fundo de overlays / sheets |
| `--surface` | `#17121F` | cards padrão (`.ide-card`), tiles |
| `--surface-2` | `#1E1830` | botões fantasma, pills, inputs elevados |
| `--surface-3` | `#271F3B` | estado "on" de segmented, trilhos de barra |
| `--line` | `rgba(255,255,255,0.075)` | borda padrão |
| `--line-2` | `rgba(255,255,255,0.13)` | borda de destaque |

### Texto
| Token | Hex | Uso |
|---|---|---|
| `--text` | `#F4F1FA` | texto principal |
| `--muted` | `#A39CB6` | secundário / descrições |
| `--dim` | `#6C667C` | terciário / labels apagados |

### Marca & acentos
| Token | Hex | Uso |
|---|---|---|
| `--purple` | `#9945FF` | acento primário |
| `--purple-soft` | `#C77DFF` | acento claro, ícones, ênfase |
| `--green` | `#14F195` | sucesso, valores positivos, "ativo" |
| `--yellow` / `--gold` | `#FFD23F` | destaque/troféu/topo do ranking (1º) |
| `--silver` | `#C9D2E0` | 2º lugar |
| `--bronze` | `#E8945A` | 3º lugar |

### Gradientes (a assinatura visual)
- `--grad` → `linear-gradient(122deg, #C77DFF, #9945FF, #2ED6A6, #14F195)`
  — usado em **texto gradiente** (`.ide-grad-text`), avatares, fundos de marca.
- `--grad-btn` → `linear-gradient(120deg, #9945FF 0%, #14F195 100%)`
  — usado em **botões primários** (`.ide-btn`), células selecionadas, FAB.

> **Texto sobre gradiente/verde** usa `#08110C` (quase-preto esverdeado) ou
> `#0B0910`. Nunca branco puro sobre o gradiente do botão.

### Sobrescrita por Tweaks (runtime)
`app.jsx` reescreve `--grad`, `--grad-btn`, `--green`, `--purple-soft`,
`--gold`/`--yellow` e `--num-font` a partir do estado de Tweaks. Consequência
prática: **se você hard-codar `#14F195` em vez de `var(--green)`, o Tweak de
paleta não afeta seu elemento.** Sempre use a var.

## Tipografia

Fontes carregadas em `Idealer.html` (Google Fonts):
- **Outfit** (400–800) — UI e corpo. `font-family: 'Outfit', system-ui, sans-serif`.
- **Space Grotesk** (500–700) — **números e display**. Aplicada via classe
  `.ide-num` (com `font-feature-settings: "tnum" 1` para dígitos tabulares).
- **Sora** (500–700) — alternativa para números (opção de Tweak `numFont`).

A fonte dos números é a var `--num-font` (default `'Space Grotesk'`), trocável
por Tweak. `.ide-num` já lê essa var.

### Escala de tamanho observada (px)
| Papel | Tamanho / peso |
|---|---|
| Display de prêmio (inteiro) | 72 / 700 |
| Título de tela grande | 24–27 / 800 |
| Título de tela (`ScreenHead`) | 21 / 800 |
| Título de card/regra | 16–20 / 700–800 |
| Corpo | 14–16 / 400–600 |
| Descrição secundária | 12.5–13.5 / 400–500 |
| Label maiúsculo (`SectionLabel`) | 11.5–12 / 700, `letter-spacing .08–.12em`, `uppercase` |
| Pill / nav | 11–14 / 600 |

`letter-spacing`: corpo `-0.01em`; títulos e `.ide-num` `-0.02em`; labels
maiúsculos positivos (`.08–.14em`).

## Raios (border-radius)
- Botões / inputs / `ScreenHead` voltar: **13–16px**
- Cards (`.ide-card`): **20px**; stat tiles: **16px**
- Pills: **999px**
- Células de número: **13px** (`.ide-numcell`), fichas de aposta **11–12px**
- Sheets/overlays (topo): **28px**
- Device frame: **52px**

## Sombras & brilhos
- Botão primário: `0 8px 24px -8px rgba(20,241,149,0.5), inset 0 1px 0 rgba(255,255,255,0.4)`
- Glow de destaque (dourado): `0 0 26px–36px -12px rgba(255,210,63,0.6)`
- Glow neon (verde/roxo): `radial-gradient(...)` + `filter: blur(...)` (ver
  `.ide-ambient` e os halos de hero). Não use `box-shadow` colorido pesado em
  cards comuns — reserve brilho para prêmio, vitória e CTA.

## Espaçamento
Sem escala formal de tokens; o ritmo observado usa múltiplos de ~2/4:
- Padding interno de card: `13–20px`
- `gap` entre itens de lista: `8–10px`
- `gap` em linhas de conteúdo: `8–13px`
- Margens entre seções: `14–26px`
- Padding de tela: `52px` topo / `18–22px` lados / `96–150px` base

Mantenha esses valores; eles dão a densidade característica do app.

## Animações (keyframes em `idealer.css`)
| Nome / classe | Efeito | Onde usar |
|---|---|---|
| `.ide-screen` / `ide-screen-in` | entrada da tela (translateY 12→0) | raiz de toda tela |
| `.ide-rise` / `ide-rise` | sobe 12px suave | cards/overlays que entram |
| `ide-pop` | escala 0.7→1.1→1 | seleção, confirmação, mascote |
| `ide-float` | flutua + gira leve | mascote em telas de espera |
| `ide-pulse` | pulsa opacidade/escala | halos de prêmio |
| `ide-shimmer` | desliza o gradiente | `.ide-grad-text` |
| `ide-drift` | deriva vertical | glow ambiente |
| `ide-confetti` | cai e gira | componente `Confetti` (vitória) |
| `ide-typing` | 3 pontos saltando | `.ide-typing` (chat) |

**Princípios:**
- Entradas **transform-only** (nunca começar em `opacity:0`) — sobrevive a
  preview sem foco e à exportação.
- Contadores "ao vivo" via `setInterval` (não `requestAnimationFrame`), pois
  rodam mesmo com iframe sem foco — ver `useLivePrize` / `useCountdown`.
- **Modos de Tweak:** `.ide-calm` neutraliza durações de animação; `.ide-noglow`
  desliga pulsos de brilho. Não burle isso com `!important`.
- Evite loops decorativos infinitos em conteúdo essencial.
