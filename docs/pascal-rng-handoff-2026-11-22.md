# 🌈 Handoff: Pascal RNG trace → Rust behavior replay

**Data:** 2026-11-22  
**Objetivo:** retomar o trabalho interrompido sem perder o estado técnico, os limites de evidência nem a ordem das próximas tarefas.

Este é o documento canônico de retomada compartilhado pelos dois PRs. Ele registra o que existe, o que foi validado, o que ainda está apenas preparado como interface e o próximo passo executável.

## 🌈 PRs e branches

| Repositório | PR | Branch → base | HEAD conhecido |
|---|---|---|---|
| [StreetChavesRust](https://github.com/danimesq/StreetChavesRust/pull/9) | [#9](https://github.com/danimesq/StreetChavesRust/pull/9) | `research/pascal-rng-jsonl` → `main` | Implementation/fixture commit at CI launch: `52437bd`; handoff added in `fa7b838` |
| [OpenStreetChaves](https://github.com/danimesq/OpenStreetChaves/pull/2) | [#2](https://github.com/danimesq/OpenStreetChaves/pull/2) | `research/sortearvalor-jsonl-v1` → `main` | `d56ac59b648eab722315a4f51148a2b8b4f65b28` |

Ambos estavam abertos na última verificação. O PR Pascal contém a instrumentação de referência; o PR Rust consome, reconcilia e valida os traces. O HEAD atual do PR Rust pode avançar com commits de handoff/documentação; a execução runtime listada abaixo foi disparada para o commit de implementação `52437bd`. O commit `fa7b838` adicionou este documento e atualizou as descrições dos dois PRs. Não mesclar um lado isoladamente sem considerar o checkout de branch cruzado usado pela CI Rust.

## 🌈 Estado recuperado

A prioridade acordada é converter os eventos observados em comportamento executável e comparável, não aumentar documentação por si só. A sequência de engenharia é:

1. Preservar a ordem global e a largura real de cada chamada RNG Pascal.
2. Fazer a execução Rust consumir esses draws no ponto correto do fluxo de ações.
3. Emitir checkpoints Rust por tick e comparar com os checkpoints Pascal, reportando a primeira divergência.
4. Ampliar os cenários e fixtures sem misturar dados estáticos com eventos realmente executados.
5. Continuar fechando lacunas de instrumentação, especialmente RNG externo e semântica booleana.

### 🌈 Instrumentação Pascal já presente no PR #2

- Schema JSONL v2 para eventos de runtime.
- 55 chamadas diretas `Random(...)` encontradas estaticamente em oito arquivos `.inc`, fora do wrapper original, encaminhadas por overloads `SortearDireto(LongInt)` e `SortearDireto(Int64)`.
- Chamadas diretas instrumentadas compartilham o índice global de chamadas e o índice de palavras MT, preservando a largura do overload.
- Filename, linha e coluna de origem para chamadas Pascal diretas; tokenizer/AST de ROT também acompanha colunas.
- Tipos de origem separados: `rot_expression`, `pascal_helper` e `pascal_direct`.
- Execuções de scripts aninhados e controles concretos de if/case/loops.
- Registros `boolean_subexpression` para operandos de `and/or/xor`, incluindo operandos direitos pulados por short-circuit.
- A captura determinística redefine `RandSeed` para `20261026`; `Randomize` continua no startup normal.

Limite: os 55 call sites são a cobertura estática encontrada no código de jogo. A execução de 3.000 ticks exercitou 23 deles. Chamadas RNG internas de RTL ou bibliotecas externas não estão interceptadas.

### 🌈 Cobertura observada no cenário fixo de 3.000 ticks

Os números abaixo são expectativas do cenário capturado, não afirmações de que todos os lutadores ou todas as ações foram executados:

| Métrica | Valor |
|---|---:|
| Ticks | 3.000 |
| Draws RNG | 11.769 |
| Palavras MT | 18.812 |
| Draws LongInt | 4.726 |
| Draws Int64 | 7.043 |
| Draws com coluna de origem | 11.343 |
| Sites de origem exercitados | 45 |
| Índices de lutador no runtime | 2 |
| IDs de ação no runtime | 24 |
| Execuções de script | 20.437 |
| Scripts aninhados | 5.229 |
| Loops: start / iteration / end | 67 / 201 / 67 |
| Registros de operandos booleanos | 675.222 |
| Short-circuit skips | 309.092 |
| `and_false` / `or_true` | 308.942 / 150 |
| Correspondências simbólicas únicas | 8.415 |
| Correspondências ambíguas / ausentes | 0 / 0 |
| Timelines simbólicas indexadas | 15 lutadores |

A distinção importa: o runtime exerceu dois índices de lutador; o índice simbólico estático contém timelines para quinze. Não descrever o segundo número como quinze lutadores executados.

A decomposição atual do trace tem 3.354 draws fora da categoria simbólica ROT: 3.350 `pascal_direct` e 4 `pascal_helper`. O relatório também preserva `source_kind_counts`.

### 🌈 Rust já implementado no PR #9

- `src/pascal_replay.rs`: replay estrito de resultados capturados, validando sequência, largura (`LongInt/Int64`), range, resultado e consumo completo.
- `PascalRandomSource` e gerador FPC 3.2.2 MTwist independente, com teste que recalcula os draws capturados. Esse recálculo é uma validação separada do adaptador de replay.
- `src/pascal_differential.rs`: parser de checkpoints `tick_start/tick_end` e comparação que aponta a primeira divergência.
- `src/bin/pascal-diff.rs`: CLI para comparar o trace Pascal com JSONL de checkpoints Rust.
- Parser/reconciliação dos controles booleanos, mantendo cada subexpressão em um registro próprio em vez de duplicá-la por draw.
- Fixture dourado `tests/fixtures/pascal-runtime-3k-coverage.json`, consumido pelo workflow para verificar os totais do cenário.

**Limite crucial:** o adaptador `PascalDrawReplay` reproduz os resultados gravados na ordem original; ele não recalcula MTwist durante a simulação de ações. O gerador MTwist independente é usado no teste de validação do capture. O comparador diferencial existe, mas ainda falta um produtor Rust de checkpoints reais gerados pelo avanço do estado do combate.

## 🌈 CI e validação

### Último estado conhecido em 2026-11-22

- [Rust contracts, run 45062424178](https://github.com/danimesq/StreetChavesRust/actions/runs/45062424178): **success** no commit `52437bd59184a368c245753b9c0dca4a54cb53aa`.
- [Pascal .rot normalization, run 45062424238](https://github.com/danimesq/StreetChavesRust/actions/runs/45062424238): **success** no mesmo commit.
- [Pascal runtime trace, run 45062417259](https://github.com/danimesq/StreetChavesRust/actions/runs/45062417259): **success**, run #70, concluído em 2026-11-22 19:49 UTC, no mesmo commit.

A falha anterior [45050708746](https://github.com/danimesq/StreetChavesRust/actions/runs/45050708746) foi corrigida: os campos `fighter_count` e `boolean_subexpression_records` foram adicionados ao fixture dourado. A execução completa posterior confirmou a correção. O workflow compara a saída contra o fixture e verifica larguras, lutadores, ações, scripts aninhados, loops, colunas de origem e razões de short-circuit.

**Validação do fixture encerrada: os três workflows acima estão verdes.** A execução runtime compila FPC 3.2.2, faz duas capturas determinísticas de 3.000 ticks, compara os JSONL e `logica.txt`, valida schema, reconcilia os draws e recalcula cada resultado com o MTwist Rust.

O commit validado pela CI é o commit de implementação `52437bd`; o HEAD atual do PR #9 avançou depois com documentação/handoff. Não confundir o SHA testado pela execução com o HEAD atual do PR.

Artifact anterior validado: [run 45049647139](https://github.com/danimesq/StreetChavesRust/actions/runs/45049647139), artifact ID `14765291483`. É evidência histórica útil, mas não substitui a execução do HEAD atual.

## 🌈 Próxima execução: ordem de trabalho

### 1. Fechar a validação do fixture atual

- Conferir a conclusão de [run 45062417259](https://github.com/danimesq/StreetChavesRust/actions/runs/45062417259).
- Se falhar, ler o primeiro erro real dos logs; não alterar os números esperados só para fazer a CI passar.
- Se passar, registrar o run/artifact no PR #9 e neste handoff.
- Conferir que o workflow usou o HEAD atual do branch Pascal `research/sortearvalor-jsonl-v1`.

### 2. Criar o primeiro runtime executável antes de integrar o replay

**Reconhecimento do branch em 2026-11-22:** ainda não existe um loop de combate executável ao qual conectar o replay. A árvore atual contém um crate de contratos/evidências: `src/lib.rs` define tipos como `ActionTimeline`, `TickEvents` e eventos ordenados; `tests/combat_contracts.rs` usa fixtures sintéticas. Não há `GameState`, `CombatSimulator`, executor de ações/scripts ou produtor de checkpoints de estado de combate em `src/`. Portanto, não há um call site real de RNG de gameplay Rust para substituir neste momento.

**Não tratar os contratos de timeline como simulador.** Conectar `PascalDrawReplay` a um teste sintético ou consumir draws artificialmente não demonstraria execução real nem equivalência de comportamento.

Ordem necessária:

1. Fazer o mapeamento de uma ação real no código Pascal, desde a seleção/entrada da ação até o avanço de um tick, incluindo guards, chamadas helper, efeitos e RNG que realmente ocorram nesse caminho.
2. Escolher um recorte pequeno e totalmente rastreável, sem inventar campos ou transições que não estejam apoiados no código-fonte/trace.
3. Implementar no Rust um primeiro estado de combate mínimo e um executor real para esse recorte. O runtime deve possuir o estado que evolui; não deve apenas reproduzir uma lista de eventos já gravada.
4. Injetar uma única instância de `PascalRandomSource` no executor e compartilhá-la por todo o tick e por chamadas aninhadas. Não reiniciar o cursor ao entrar em helper ou ação filha.
5. Consumir draws no ponto exato da execução, preservando ordem, overload e range. Em caso de mismatch, reportar tick, fighter, action/script, API call index, width e range.
6. Emitir checkpoints Rust a partir do estado efetivamente calculado e só então comparar com Pascal usando `pascal-diff`.
7. Expandir o recorte apenas depois que o primeiro caminho vertical estiver validado.

**Critério de saída:** pelo menos uma ação/script real executada pelo runtime Rust, com RNG compartilhado, checkpoints próprios e comparação diferencial reproduzível. Não fabricar estado ou ação para consumir draws que o Rust ainda não executa. Falha explícita é evidência melhor que alinhamento artificial.

### 3. Produzir checkpoints Rust reais

- Definir o mesmo conjunto de campos que o harness Pascal efetivamente registra.
- Emitir `tick_start` e `tick_end` do estado Rust após executar a lógica, não copiar os valores Pascal para um arquivo com rótulo Rust.
- Rodar `pascal-diff` e preservar a primeira divergência: tick, fase, campo, valor Pascal e valor Rust.
- Começar por um cenário pequeno reproduzível; expandir depois que o primeiro ciclo fecha.

### 4. Expandir fixtures de comportamento

- Incluir mais lutadores, ações e sequências de input.
- Cobrir loops, helpers, scripts aninhados e ambos os overloads RNG.
- Separar claramente cobertura estática do catálogo simbólico e cobertura dinâmica exercitada.
- Manter fixtures determinísticos, versionados e com asserts de CI.

### 5. Fechar lacunas de instrumentação

- Investigar chamadas internas de `System.Random`/RTL e bibliotecas, sem presumir que os 55 call sites de jogo cobrem tudo.
- Investigar `not` unário e demais construções booleanas além de `and/or/xor`.
- Preservar source line/column e short-circuit efetivo; não inferir avaliação de operandos que foram pulados.
- Manter o índice global monotônico e compartilhado para todos os draws instrumentados.

## 🌈 Regras de integridade

- Não inventar ticks, índices de draw, chamadas ou transições de estado.
- Não confundir inventário estático com execução dinâmica.
- Não afirmar paridade integral com o executável original: os checkpoints atuais são campos selecionados, não snapshot completo de memória.
- Não chamar replay de resultados gravados de recálculo MTwist dentro do simulador.
- Não declarar o simulador de combate completo enquanto não executar ações reais e emitir checkpoints Rust independentes.
- Preservar rastreabilidade: commits pequenos, fixtures versionados, CI ligada ao branch e documentação atualizada quando o escopo mudar.

## 🌈 Arquivos de entrada principais

- `src/pascal_replay.rs`
- `src/pascal_differential.rs`
- `src/bin/pascal-diff.rs`
- `src/trace_join.rs`
- `src/bin/trace-join.rs`
- `tests/fixtures/pascal-runtime-3k-coverage.json`
- `.github/workflows/pascal-runtime-trace.yml`
- `docs/pascal-reference-trace-harness.md`
- `docs/evidence-and-open-questions.md`

**Próximo marco real:** um cenário Rust executando pelo menos uma ação/script real com o replay compartilhado, produzindo checkpoints próprios e permitindo que `pascal-diff` aponte uma divergência real (ou confirme igualdade) por tick. Até lá, o trabalho está na camada de evidência, replay RNG e interfaces de comparação, não na simulação integral do combate.


## First executable Rust runtime slice and expansion (2026-11-22, PR #9)

The first Rust runtime now executes a bounded portion of `Dados/Lutadores/Comum.rot [IA]`: target selection, movement and the close-range attack probability chain.

Implemented on the PR #9 branch:
- `src/combat_runtime.rs`: both `Random(13)` branches, `Random(6)` target selection, ordered movement predicates, close-range probability draws, common basic action state changes, and an injectable character-action callback.
- `src/bin/pascal-replay-tick.rs`: reconstructs tick 316 from the actual captured AI snapshot and emits Rust-owned checkpoints.
- `tests/combat_runtime.rs`: snapshot-driven tests for ticks 316, 317 and 381; a shared-cursor child callback test; and a CLI round-trip through `pascal-diff`.
- `docs/first-rust-runtime-slice.md`: source-level behavior, actual input snapshots, evidence boundaries and reproduction.

### Captured input snapshots

OpenStreetChaves PR #3 adds trace-only `ai_inputs` to `.rot [IA]` `script_start` JSONL records. The fresh 3,000-tick capture contains these actual entry inputs:

| Tick | Dist | DistProj | OppY | Ameaca | Fator | IA P.DistanciaIA | IA facing |
|---:|---:|---:|---:|---:|---:|---:|---|
| 316 | 132 | 0 | 300 | 0 | 3.6 | 0 | left |
| 317 | 124 | 0 | 300 | 0 | 3.6 | 0 | left |
| 381 | 78 | 0 | 300 | 0 | 3.6 | 300 | left |

The snapshots also include both fighters' character/action/frame/duration, facing, defending state, X/Y, vertical velocity, airborne state, speed, health and body boxes. The tests parse these values from the fixtures rather than inventing convenient state.

Tick 316 starts with the IA fighter at X=366, the opponent at X=234, IA speed 8, body boxes `[-19,35,13,164]` and `[-18,30,13,170]`. The Rust executor walks left to X=358, action 2/frame 1. Its independent tick checkpoints match the two available Pascal checkpoints using `pascal-diff`.

Tick 317 records `Random(13)=0`, followed by `Random(6)=2`, selecting target distance 200. With the captured Dist=124 and facing left, the IA walks right.

Tick 381 records the ten real draws at global API call indices 423-432. The captured factor is 3.6. The runtime preserves the exact LongInt/Int64 widths, ranges, results and source order, then reproduces the weak-kick and crouch command decisions for the recorded sequence.

The AI snapshot does not yet include `CameraX` or `RelogioParado`; those globals are needed before claiming parity for screen-edge or paused-clock overlap cases.

### Nested RNG cursor

A `CharacterActionScript` callback represents the character-specific `[Acao]` call inside Pascal `AcaoJogador`. Parent AI and child callback receive the same mutable `PascalRandomSource`. A deterministic integration test makes the child consume the next draw and verifies the cursor is not reset.

**Evidence boundary:** the captured Pascal 3,000-tick trace contains nested script invocations, but no RNG draw with a non-null parent script execution ID. The Rust test proves the shared-cursor architecture, not historical child-script RNG parity. Actual character-specific scripts still need to be ported, and a Pascal child-script RNG draw should be captured before claiming behavioral validation for that case.

### PRs and CI

- Rust implementation: https://github.com/danimesq/StreetChavesRust/pull/9
- Trace-only Pascal input snapshot PR: https://github.com/danimesq/OpenStreetChaves/pull/3
- Pascal FPC 3.2.2 compile, deterministic 3,000-tick capture, snapshot validation and artifact upload: https://github.com/danimesq/OpenStreetChaves/actions/runs/45069244306
- Full Pascal trace artifact: `ai-input-snapshot-trace`, artifact ID `14772348640`.
- Rust contracts on the snapshot-driven runtime head: https://github.com/danimesq/StreetChavesRust/actions/runs/45070221592 (success; 33 library tests, 2 binary tests, 5 runtime integration tests).
- Pascal .rot normalization on the same head: https://github.com/danimesq/StreetChavesRust/actions/runs/45070221627 (success).

The selected tick-316 sample is a complete two-checkpoint differential regression. Ticks 317 and 381 currently validate the captured RNG path and selected IA-script outcome; they are not yet full tick-level differential matches. This runtime is not a full combat simulator or a general Pascal .rot interpreter.

### Next work

1. Port and test actual character-specific `[Acao]` scripts against the common runtime.
2. Obtain a Pascal capture where a child script itself consumes RNG; compare that exact nested sequence with Rust.
3. Add `CameraX`, `RelogioParado` and remaining global state required for edge/collision cases.
4. Expand full-state checkpoints and gameplay phases incrementally, only when backed by real Pascal captures.
