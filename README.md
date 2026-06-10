<div align="center">

# 🛰️ Claude Code HUD

**Uma barra de status leve, instantânea e à prova de bug para o Claude Code — escrita em Rust.**

![Rust](https://img.shields.io/badge/Rust-1.96-orange?logo=rust&logoColor=white)
![Binary](https://img.shields.io/badge/bin%C3%A1rio-362%20KB-blue)
![Runtime deps](https://img.shields.io/badge/deps%20de%20runtime-0-success)
![Open Source](https://img.shields.io/badge/100%25-open%20source-success)
![Tests](https://img.shields.io/badge/testes-19%20passando-brightgreen)
![Platform](https://img.shields.io/badge/Windows%20%C2%B7%20macOS%20%C2%B7%20Linux-informational)
![License](https://img.shields.io/badge/licen%C3%A7a-MIT-lightgrey)

</div>

---

## A barra

```text
Opus 4.8  ·  xhigh  ·  ctx ▓▓▓▓▓▓▓░░░░░░░░░ 42%  ·  84k/200k  ·  carga LEVE  ·  $0.23  ·  5h 24% 7d 41%  ·  main  ·  12m
```

Tudo numa linha só, com o medidor e o % do contexto coloridos conforme a carga:

> 🟢 **LEVE** `< 50%`  ·  🟡 **MÉDIO** `50–80%`  ·  🔴 **ALTO** `> 80%`

Quando a janela de contexto enche, o medidor vai do verde ao vermelho — um aviso
visual de que o `/compact` está chegando. O rate-limit do plano também fica
vermelho quando você se aproxima da cota.

---

## ✨ Por que esta versão é mais leve (e não buga)

As statuslines populares em Node/TS bugam por três motivos: **parseiam o
transcript** `.jsonl` (e contam tokens duplicados do streaming), **spawnam
processos** (`ccusage`, `jq`, `git`) a cada render (locks travados, lentidão) e
**dependem de runtime externo**. Este HUD não faz nada disso.

| | 🛰️ Este HUD | 🐢 Statuslines típicas (Node/TS) |
|---|---|---|
| **Fonte dos dados** | stdin JSON nativo do Claude Code | parse do transcript `.jsonl` |
| **Contagem de tokens** | pronta e exata | recalculada (duplica no streaming) |
| **Runtime** | binário nativo, **0 dependências** | Node/Bun + às vezes `jq` |
| **Subprocessos por render** | **nenhum** | `ccusage` / `jq` / `git` |
| **Cold start** | **~poucos ms** | ~120 ms a 1,3 s |
| **Em caso de erro** | linha vazia, `exit 0` (barra nunca quebra) | crash / flicker / barra travada |
| **Branch git** | lê `.git/HEAD` direto | shell-out `git` (locks/lentidão) |

O binário inteiro tem **~362 KB** e roda em milissegundos. Como a statusline é
re-executada dezenas de vezes por sessão, esse custo quase-zero é o que faz o HUD
parecer instantâneo em vez de travado.

---

## 📊 O que ele mostra — e de onde puxa cada dado

O Claude Code envia um **JSON no stdin** a cada atualização da barra (debounce de
300 ms). O HUD só lê esse JSON, formata e imprime. Cada segmento:

| Segmento | Exemplo | De onde vem (campo do JSON do stdin) |
|---|---|---|
| 🤖 **Modelo** | `Opus 4.8` | `model.id` → rótulo bonito (fallback `model.display_name`) |
| 🧠 **Effort** | `xhigh` | `effort.level` |
| 📊 **Contexto** | `ctx ▓▓▓░ 42%` | `context_window.used_percentage` |
| 🔢 **Tokens** | `84k/200k` | `context_window.total_input_tokens` / `context_window_size` |
| 🌡️ **Carga** | `LEVE` · `MÉDIO` · `ALTO` | derivado de `used_percentage` (limiares configuráveis) |
| 💲 **Custo** | `$0.23` | `cost.total_cost_usd` |
| ⏳ **Rate-limit** | `5h 24% 7d 41%` | `rate_limits.five_hour` / `seven_day` `.used_percentage` |
| 🌿 **Branch** | `main` | `.git/HEAD` lido direto do disco (sem subprocess) |
| ⏱️ **Timer** | `12m` | `cost.total_duration_ms` |

Segmentos sem dados são **omitidos** silenciosamente (ex.: sem repo git → sem
branch; modelo sem effort → sem effort). Em terminal estreito, os de menor
prioridade somem primeiro (`timer → branch → rate_limit → cost → tokens → load →
effort`); **modelo** e **medidor de contexto** são sempre preservados.

---

## 🦀 Por que Rust

- **Cold start ~ms.** A barra roda a cada mensagem; um binário compilado tem
  custo de inicialização praticamente nulo. Referências em Rust (CCometixLine,
  claude-powerline-rust) medem **8–15× mais rápido** que equivalentes em Node.
- **Zero dependências de runtime.** Um único `.exe` — nada de Node, Bun ou `jq`
  para instalar. Isso elimina de uma vez a classe de bug "faltou a dependência".
- **Impossível de crashar à toa.** Todo campo do JSON é `Option`: dado ausente
  vira segmento omitido, nunca panic. Qualquer falha → linha vazia, `exit 0`.
- **Parse robusto com `serde`.** Campos desconhecidos são ignorados, então
  mudanças de schema do Claude Code não quebram o HUD.

---

## 🤖 Instalação automática (deixe o seu Claude Code instalar)

Não quer fazer na mão? Cole o prompt abaixo no **seu** Claude Code — ele clona,
compila e configura tudo sozinho. Troque a URL pela do repositório:

```text
Instale este HUD de statusline no meu Claude Code: https://github.com/armitagethird/ClaudeCodeHUD

Passos:
1. Clone o repositório num diretório local.
2. Verifique se eu tenho o Rust (`cargo`) instalado. Se não tiver, me explique como instalar e pare aqui.
3. Rode `cargo build --release` na raiz do repo.
4. Faça backup do meu `~/.claude/settings.json` e mescle nele o bloco `statusLine`,
   apontando para o binário gerado em `target/release/hud.exe`. Use barras normais `/`
   no caminho (mesmo no Windows), com "padding": 0 e "refreshInterval": 10.
   Preserve TODO o resto do meu settings.json.
5. Valide que o settings.json continua um JSON válido.
6. Me diga para reiniciar o Claude Code para a barra aparecer.

Tudo é local: não envie nada para a rede.
```

> 💡 É só uma conveniência. O Claude Code vai pedir sua permissão antes de
> compilar e antes de editar o `settings.json` — você revisa cada passo. Prefere
> controle total? Siga a instalação manual abaixo.

## 🚀 Build & instalação manual

Requer o toolchain Rust (`cargo`). Na raiz do projeto:

```powershell
cargo build --release
```

O binário sai em `target/release/hud.exe`. Depois, mescle em
`~/.claude/settings.json` (use **barras normais** `/` no caminho do Windows):

```json
{
  "statusLine": {
    "type": "command",
    "command": "F:/Developments/HUD-claudecode/target/release/hud.exe",
    "padding": 0,
    "refreshInterval": 10
  }
}
```

`refreshInterval: 10` mantém o **timer** vivo durante a ociosidade (re-roda o
binário a cada 10 s — custo desprezível). Reinicie o Claude Code para a barra
aparecer.

---

## ⚙️ Configuração (opcional, sem recompilar)

Copie `hud.toml.example` para `~/.claude/hud.toml` e ajuste segmentos, ordem,
cores, largura do medidor, limiares de carga e rótulos de modelo. Para apontar
outro caminho, defina a env var `HUD_CONFIG`.

Se o seu terminal não renderizar os blocos `▓░`, ligue `ascii_mode = true` para
usar `[#----]` — máxima compatibilidade.

Erro de leitura ou de parse do `hud.toml`? O HUD cai nos **defaults embutidos** —
nunca quebra.

---

## 🔒 É seguro? Segurança & privacidade

**Sim — e é 100% open source.** Todo o código (~300 linhas de Rust) está neste
repositório, auditável linha por linha. Sem caixa-preta.

- **Recurso oficial.** `statusLine` é um ponto de extensão **documentado** do
  Claude Code. Usar o HUD é usar o produto exatamente como foi projetado — não é
  hack nem contorno.
- **Sem rede, sem telemetria.** O binário não faz **nenhuma** chamada de rede.
  Nada sai da sua máquina.
- **Somente leitura + local.** Ele só lê o JSON que o Claude Code entrega no
  stdin, o arquivo `.git/HEAD` e o `hud.toml` opcional. A única escrita é a linha
  de texto na barra.
- **Não toca na API nem em limites.** Não consome tokens, não contorna rate
  limit, não altera o comportamento do modelo. Os números de rate-limit exibidos
  são os que a Anthropic já te envia.
- **À prova de crash.** Qualquer erro → linha vazia, `exit 0`. Nunca derruba a barra.

**Risco de banimento ou punição da Anthropic: essencialmente nulo.** O tool não
viola política de uso — não há abuso, contorno de limite, automação indevida nem
engenharia reversa. É um formatador cosmético de dados que já são seus.

> ⚠️ Em geral, qualquer `statusLine` roda um programa local a cada render — uma
> superfície de confiança. Por isso, só rode binários em que você confia. Aqui o
> código é aberto e auditável: prefira sempre **compilar do fonte** a rodar um
> `.exe` pronto de terceiros.

## 🧪 Teste rápido (sem o Claude Code)

```powershell
Get-Content tests/fixtures/full.json | .\target\release\hud.exe
```

E a suíte completa:

```powershell
cargo test     # 13 unitários (funções puras) + 6 de integração (stdin → binário)
```

---

<div align="center">

Feito para ser **leve, anti-bug e útil de verdade**. 🛰️

**100% open source** · licença MIT

por **[vibesurfdev](https://github.com/armitagethird)** · [armitagethird/ClaudeCodeHUD](https://github.com/armitagethird/ClaudeCodeHUD)

</div>
