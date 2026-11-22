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

### 2. Ligar replay ao fluxo real de ações Rust

- Localizar o ponto central de execução de ações/scripts e a API atual de RNG no simulador.
- Introduzir uma abstração de RNG que permita escolher entre o gerador nativo e `PascalDrawReplay` sem espalhar condicionais pelo código de combate.
- Passar o mesmo replay sequencial por todo o tick, incluindo scripts aninhados; não reiniciar o cursor ao entrar em helper ou ação filha.
- Consumir draws no ponto exato da execução, preservando ordem, overload e range.
- Ao terminar cenário/tick, verificar draws restantes e reportar o primeiro mismatch com tick, fighter, action/script, API call index, width e range.
- Não fabricar estado ou ação para consumir draws que o Rust ainda não executa. Falha explícita é evidência melhor que alinhamento artificial.

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
