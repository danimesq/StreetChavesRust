# 🌈 Ordered frame-script event ledger — 15 fighters

## 🌈 Scope and method

This document expands the timer/frame index into **source-ordered event ledgers** for each character-specific `[Quadro]` block in the pinned Pascal/SDL2 snapshot. It records every line that writes a frame, tests an animation/timer predicate, queues sound, creates an effect, consumes RNG, or mutates simulation state/motion.

This is an intermediate timeline artifact, not a fully evaluated per-tick execution trace. The source is a Pascal-like interpreted script with nested guards, loops, and ordered statements. Each row preserves its original source line and order; read consecutive rows together to retain surrounding nested branch context. A future normalized Rust specification must still resolve exact guard nesting, loop expansion, same-tick call order, and interactions with common `Comum.rot`.

Pinned source: [OpenStreetChaves @ e1b025c35ecbc2b14d8635564aafd23c2107d3d1](https://github.com/danimesq/OpenStreetChaves/tree/e1b025c35ecbc2b14d8635564aafd23c2107d3d1)

Evidence label: **[port-specific]** for the Pascal script's exact implementation. The C port and original executable remain separate evidence sources. Do not treat a Pascal-only script detail as original-game behavior without cross-source/runtime validation.

## 🌈 Event categories

- `timer/branch`: animation-clock, duration, combo-timer, or branch predicates.
- `frame`: sprite/frame assignment.
- `effect`: effect spawn call.
- `sound`: pending sound assignment.
- `RNG`: random-number call. The exact call order is significant.
- `simulation mutation`: player/opponent state, position, velocity, damage, invulnerability, hit counters, push, or transition calls.

## 🌈 1. Chaves

- Source: [`C01.rot` lines 85–182](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L85)
- SHA/blob: `4043ba2780318d05da8e7589a5ca6997408b58bd`
- `[Quadro]` source lines: 98
- Indexed event statements: 66
- Category counts: `timer/branch` 38; `frame` 28; `effect` 9; `sound` 5; `RNG` 1; `simulation mutation` 19

| Source line | Category | Ordered source statement |
|---:|---|---|
| [90](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L90) | timer/branch, frame | `if P.QuadroAcao = 0 then P.Quadro := Desloc + 33;` |
| [91](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L91) | timer/branch, frame | `if P.QuadroAcao = 7 then P.Quadro := Desloc + 34;` |
| [92](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L92) | timer/branch, frame | `if P.QuadroAcao = 14 then P.Quadro := Desloc + 35;` |
| [93](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L93) | timer/branch, frame | `if P.QuadroAcao = 21 then P.Quadro := Desloc + 34;` |
| [94](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L94) | timer/branch, frame | `if P.QuadroAcao = 28 then P.Quadro := Desloc + 33;` |
| [95](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L95) | timer/branch | `if P.QuadroAcao = 28 then P.QuadroAcao := 0;` |
| [97](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L97) | timer/branch, frame | `if (P.Acao = acPose) and (P.QuadroAcao = 0) then P.Quadro := Desloc + 12;` |
| [98](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L98) | timer/branch, frame | `if (P.Acao = acPose) and (P.QuadroAcao = 2) then P.Quadro := Desloc + 30;` |
| [101](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L101) | timer/branch, frame | `if P.QuadroAcao = 0 then P.Quadro := Desloc + 36;` |
| [102](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L102) | timer/branch, frame | `if P.QuadroAcao = 4 then P.Quadro := Desloc + 37;` |
| [103](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L103) | timer/branch, frame | `if P.QuadroAcao = 7 then P.Quadro := Desloc + 38;` |
| [104](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L104) | timer/branch | `if P.QuadroAcao = 10 then` |
| [106](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L106) | frame | `P.Quadro := Desloc + 39;` |
| [107](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L107) | effect | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgFraco) then CriaEfeito(7, 'A', Jog, P.X + 45, P.Y);` |
| [108](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L108) | effect | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgFraco) then CriaEfeito(7, 'A', Jog, P.X - 45, P.Y);` |
| [109](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L109) | effect | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgForte) then CriaEfeito(7, 'B', Jog, P.X + 45, P.Y);` |
| [110](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L110) | effect | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgForte) then CriaEfeito(7, 'B', Jog, P.X - 45, P.Y);` |
| [111](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L111) | sound | `SomPendente := 14;` |
| [116](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L116) | timer/branch, sound | `if P.QuadroAcao = 0 then SomPendente := 15;` |
| [117](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L117) | timer/branch, frame | `if P.QuadroAcao = 0 then P.Quadro := Desloc + 40;` |
| [118](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L118) | timer/branch, frame | `if (P.QuadroAcao = 6) and (P.ForcaGolpe = fgFraco) then P.Quadro := Desloc + 41;` |
| [119](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L119) | timer/branch, frame | `if (P.QuadroAcao = 10) and (P.ForcaGolpe = fgForte) then P.Quadro := Desloc + 41;` |
| [120](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L120) | timer/branch, frame | `if (P.QuadroAcao = 12) and (P.ForcaGolpe = fgFraco) then P.Quadro := Desloc + 42;` |
| [121](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L121) | timer/branch, frame | `if (P.QuadroAcao = 19) and (P.ForcaGolpe = fgForte) then P.Quadro := Desloc + 42;` |
| [124](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L124) | effect, RNG | `for Cont := 1 to 3 do CriaEfeito(2, '', 0, Random(40) + (P.X - 20), Random(10) + P.Y);` |
| [125](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L125) | simulation mutation | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgFraco) then Empurra(Jog, 19);` |
| [126](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L126) | simulation mutation | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgFraco) then Empurra(Jog, -19);` |
| [127](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L127) | simulation mutation | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgForte) then Empurra(Jog, 21);` |
| [128](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L128) | simulation mutation | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgForte) then Empurra(Jog, -21);` |
| [130](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L130) | timer/branch, effect | `if (P.Lado = ldDireita) and (P.QuadroAcao >= 9) then CriaEfeito(16, '', Jog, P.X + 60, P.Y - 85);` |
| [131](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L131) | timer/branch, effect | `if (P.Lado = ldEsquerda) and (P.QuadroAcao >= 9) then CriaEfeito(16, '', Jog, P.X - 60, P.Y - 85);` |
| [132](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L132) | simulation mutation | `P.EhChute := True;` |
| [136](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L136) | simulation mutation | `if P.Acertos >= 1 then O.Invulneravel := False;` |
| [137](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L137) | simulation mutation | `if P.Acertos >= 4 then O.Invulneravel := True;` |
| [138](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L138) | timer/branch, simulation mutation | `if P.QuadroAcao > 7 then O.Invulneravel := True;` |
| [139](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L139) | timer/branch, sound | `if P.QuadroAcao = 0 then SomPendente := 16;` |
| [140](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L140) | timer/branch | `if P.QuadroAcao mod 3 = 0 then` |
| [142](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L142) | frame | `if P.Quadro = Desloc + 43 then P.Quadro := Desloc + 44` |
| [143](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L143) | frame | `else P.Quadro := Desloc + 43;` |
| [145](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L145) | simulation mutation | `P.Y := P.Y - 10;` |
| [146](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L146) | simulation mutation | `if P.Lado = ldDireita then Empurra(Jog, 5);` |
| [147](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L147) | simulation mutation | `if P.Lado = ldEsquerda then Empurra(Jog, -5);` |
| [148](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L148) | timer/branch, simulation mutation | `if P.QuadroAcao = 20 then P.NoAr := True;` |
| [149](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L149) | simulation mutation | `P.EhChute := True;` |
| [153](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L153) | timer/branch, sound | `if P.QuadroAcao = 0 then SomPendente := 17;` |
| [154](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L154) | timer/branch, frame | `if P.QuadroAcao = 0 then P.Quadro := Desloc + 45;` |
| [155](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L155) | timer/branch, frame | `if P.QuadroAcao = 4 then P.Quadro := Desloc + 46;` |
| [156](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L156) | timer/branch, frame | `if P.QuadroAcao = 8 then P.Quadro := Desloc + 47;` |
| [157](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L157) | timer/branch, frame | `if P.QuadroAcao = 12 then P.Quadro := Desloc + 48;` |
| [158](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L158) | timer/branch, frame | `if P.QuadroAcao = 15 then P.Quadro := Desloc + 49;` |
| [159](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L159) | timer/branch, frame | `if P.QuadroAcao = 18 then P.Quadro := Desloc + 12;` |
| [160](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L160) | timer/branch | `if (P.QuadroAcao >= 4) and (P.QuadroAcao <= 13) then` |
| [162](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L162) | simulation mutation | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgFraco) then Empurra(Jog, 17);` |
| [163](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L163) | simulation mutation | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgFraco) then Empurra(Jog, -17);` |
| [164](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L164) | simulation mutation | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgForte) then Empurra(Jog, 21);` |
| [165](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L165) | simulation mutation | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgForte) then Empurra(Jog, -21);` |
| [167](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L167) | timer/branch, simulation mutation | `if P.QuadroAcao >= 12 then P.EhChute := True;` |
| [171](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L171) | timer/branch, frame | `if P.QuadroAcao = 0 then P.Quadro := Desloc + 36;` |
| [172](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L172) | timer/branch, frame | `if P.QuadroAcao = 3 then P.Quadro := Desloc + 37;` |
| [173](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L173) | timer/branch, frame | `if P.QuadroAcao = 6 then P.Quadro := Desloc + 38;` |
| [174](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L174) | timer/branch, sound | `if P.QuadroAcao = 9 then SomPendente := 18;` |
| [175](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L175) | timer/branch | `if P.QuadroAcao = 9 then` |
| [177](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L177) | frame | `P.Quadro := Desloc + 39;` |
| [178](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L178) | effect | `if P.Lado = ldDireita then CriaEfeito(8, 'A', Jog, P.X + 110, P.Y);` |
| [179](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L179) | effect | `if P.Lado = ldEsquerda then CriaEfeito(8, 'A', Jog, P.X - 110, P.Y);` |
| [180](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C01.rot#L180) | simulation mutation | `TremorTela := 20;` |

## 🌈 2. Seu Madruga

- Source: [`C02.rot` lines 78–240](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L78)
- SHA/blob: `27b66fdde28e6b57542c2c9bc28c3908ac31c1ef`
- `[Quadro]` source lines: 163
- Indexed event statements: 106
- Category counts: `frame` 17; `RNG` 25; `timer/branch` 21; `effect` 7; `sound` 12; `simulation mutation` 57

| Source line | Category | Ordered source statement |
|---:|---|---|
| [82](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L82) | frame | `if P.Acao = acVitoria then P.Quadro := Desloc + 33;` |
| [83](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L83) | frame, RNG | `if (P.Acao = acVitoria) and (Random(2) = 0) then P.Quadro := Desloc + 34;` |
| [84](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L84) | timer/branch, frame | `if (P.Acao = acPose) and (P.QuadroAcao = 0) then P.Quadro := Desloc + 12;` |
| [85](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L85) | timer/branch, frame | `if (P.Acao = acPose) and (P.QuadroAcao = 2) then P.Quadro := Desloc + 30;` |
| [88](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L88) | timer/branch, frame | `if P.QuadroAcao = 0 then P.Quadro := Desloc + 35;` |
| [89](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L89) | timer/branch, frame | `if P.QuadroAcao = 8 then P.Quadro := Desloc + 36;` |
| [90](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L90) | timer/branch | `if P.QuadroAcao = 10 then` |
| [92](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L92) | frame | `P.Quadro := Desloc + 37;` |
| [93](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L93) | effect | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgFraco) then CriaEfeito(9, 'A', Jog, P.X + 55, P.Y - 100);` |
| [94](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L94) | effect | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgFraco) then CriaEfeito(9, 'A', Jog, P.X - 55, P.Y - 100);` |
| [95](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L95) | effect | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgForte) then CriaEfeito(9, 'B', Jog, P.X + 55, P.Y - 100);` |
| [96](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L96) | effect | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgForte) then CriaEfeito(9, 'B', Jog, P.X - 55, P.Y - 100);` |
| [97](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L97) | sound | `SomPendente := 20;` |
| [102](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L102) | timer/branch, frame | `if P.QuadroAcao = 0 then P.Quadro := Desloc + 38;` |
| [103](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L103) | timer/branch, frame | `if P.QuadroAcao = 5 then P.Quadro := Desloc + 39;` |
| [104](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L104) | timer/branch | `if ((P.QuadroAcao = 11) and (P.ForcaGolpe = fgFraco)) or ((P.QuadroAcao = 15) and (P.ForcaGolpe = fgForte)) then` |
| [106](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L106) | frame | `P.Quadro := Desloc + 40;` |
| [107](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L107) | simulation mutation | `P.NoAr := True;` |
| [108](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L108) | simulation mutation | `P.VelY := -6;` |
| [114](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L114) | simulation mutation | `P.Y := P.Y - 11;` |
| [115](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L115) | simulation mutation | `if P.Lado = ldDireita then Empurra(Jog, 4);` |
| [116](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L116) | simulation mutation | `if P.Lado = ldEsquerda then Empurra(Jog, -4);` |
| [120](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L120) | simulation mutation | `P.Y := P.Y - 12;` |
| [121](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L121) | timer/branch | `if P.QuadroAcao < 10 then` |
| [123](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L123) | simulation mutation | `if P.Lado = ldDireita then Empurra(Jog, 9);` |
| [124](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L124) | simulation mutation | `if P.Lado = ldEsquerda then Empurra(Jog, -9);` |
| [126](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L126) | timer/branch | `if P.QuadroAcao >= 10 then` |
| [128](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L128) | simulation mutation | `if P.Lado = ldDireita then Empurra(Jog, 3);` |
| [129](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L129) | simulation mutation | `if P.Lado = ldEsquerda then Empurra(Jog, -3);` |
| [133](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L133) | simulation mutation | `if P.Quadro = Desloc + 40 then P.NoAr := True;` |
| [134](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L134) | timer/branch, simulation mutation | `if P.QuadroAcao >= 5 then P.EhChute := True;` |
| [135](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L135) | simulation mutation | `if P.Acertos = 1 then O.Invulneravel := False;` |
| [139](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L139) | timer/branch | `if P.QuadroAcao = 0 then` |
| [141](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L141) | frame | `P.Quadro := Desloc + 24;` |
| [142](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L142) | simulation mutation | `P.VelY := 0;` |
| [143](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L143) | simulation mutation | `P.DirPulo := puloNenhum;` |
| [145](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L145) | timer/branch | `if (P.Acao = acEspecial3) and (P.QuadroAcao >= 3) then` |
| [147](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L147) | frame | `P.Quadro := Desloc + 25;` |
| [148](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L148) | simulation mutation | `if P.ForcaGolpe = fgFraco then P.Y := P.Y + 20;` |
| [149](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L149) | simulation mutation | `if P.ForcaGolpe = fgForte then P.Y := P.Y + 13;` |
| [150](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L150) | simulation mutation | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgFraco) then Empurra(Jog, 11);` |
| [151](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L151) | simulation mutation | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgFraco) then Empurra(Jog, -11);` |
| [152](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L152) | simulation mutation | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgForte) then Empurra(Jog, 15);` |
| [153](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L153) | simulation mutation | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgForte) then Empurra(Jog, -15);` |
| [154](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L154) | simulation mutation | `if P.Y > 270 then P.VelY := 2;` |
| [157](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L157) | timer/branch | `if (P.Acao in [acEspecial4, acEspecial5]) and (P.QuadroAcao >= 0) then` |
| [159](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L159) | timer/branch, sound | `if (P.Acao = acEspecial4) and (P.QuadroAcao = 0) then SomPendente := 22;` |
| [161](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L161) | RNG | `S_0 := Random(3) + (Desloc + 41);` |
| [163](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L163) | frame | `P.Quadro := S_0;` |
| [164](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L164) | sound, RNG | `if Random(2) = 0 then SomPendente := sndGolpeFraco;` |
| [165](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L165) | sound, RNG | `if Random(2) = 0 then SomPendente := sndGolpeForte;` |
| [166](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L166) | sound, RNG | `if Random(2) = 0 then SomPendente := sndGolpeFraco;` |
| [167](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L167) | sound, RNG | `if Random(2) = 0 then SomPendente := sndGolpeForte;` |
| [170](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L170) | RNG, simulation mutation | `if Flash and (Random(6) = 0) then FlashVermelho := 3;` |
| [171](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L171) | simulation mutation | `P.Recuo := 0;` |
| [172](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L172) | RNG | `O.Quadro := Random(2) + 26 + (50 - Desloc);` |
| [175](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L175) | simulation mutation | `O.VelY := 0;` |
| [176](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L176) | simulation mutation | `O.Y := O.Y + 10;` |
| [177](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L177) | simulation mutation | `if ChaoY < O.Y then O.Y := ChaoY;` |
| [178](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L178) | RNG, simulation mutation | `if P.Lado = ldDireita then O.X := Random(2) shl 3 + (P.X + 40);` |
| [179](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L179) | RNG, simulation mutation | `if P.Lado = ldEsquerda then O.X := P.X - 40 - Random(2) shl 3;` |
| [180](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L180) | simulation mutation | `P.Acertos := P.Acertos + 1;` |
| [181](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L181) | timer/branch, simulation mutation | `P.TempoAcertos := 20;` |
| [182](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L182) | sound, RNG | `if Random(4) = 0 then SomPendente := sndAcertoFraco;` |
| [183](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L183) | sound, RNG | `if Random(4) = 0 then SomPendente := sndAcertoForte;` |
| [184](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L184) | sound, RNG | `if Random(4) = 0 then SomPendente := sndAcertoEspecial;` |
| [185](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L185) | effect, RNG | `CriaEfeito(1, '', 0, Random(20) + (O.X - 30), Random(120) + (O.Y - 150));` |
| [186](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L186) | timer/branch | `if P.QuadroAcao > P.DuracaoAcao - 2 then` |
| [188](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L188) | simulation mutation | `P.DanoGolpe := 20;` |
| [189](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L189) | simulation mutation | `O.Vida := O.Vida - P.DanoGolpe * NivelDano;` |
| [190](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L190) | simulation mutation | `AcaoJogador(3 - Jog, cmDerrubar);` |
| [191](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L191) | simulation mutation | `O.Recuo := 30;` |
| [195](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L195) | timer/branch, frame | `if (P.Acao = acEspecial6) and (P.QuadroAcao = 0) then P.Quadro := Desloc + 44;` |
| [196](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L196) | timer/branch | `if (P.Acao = acEspecial6) and (P.QuadroAcao > 2) then` |
| [198](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L198) | effect, RNG | `for Cont := 1 to 8 do CriaEfeito(2, '', 0, Random(40) + (P.X - 20), Random(10) + P.Y);` |
| [199](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L199) | frame | `P.Quadro := Desloc + 45;` |
| [200](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L200) | simulation mutation | `if P.Lado = ldDireita then Empurra(Jog, 23);` |
| [201](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L201) | simulation mutation | `if P.Lado = ldEsquerda then Empurra(Jog, -23);` |
| [205](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L205) | simulation mutation | `P.Recuo := 0;` |
| [206](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L206) | frame, RNG | `P.Quadro := Random(45) + 1 + Desloc;` |
| [207](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L207) | RNG, simulation mutation | `if Flash and (Random(5) = 0) then FlashVermelho := 3;` |
| [208](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L208) | simulation mutation | `if P.Lado = ldDireita then Empurra(Jog, 3);` |
| [209](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L209) | simulation mutation | `if P.Lado = ldEsquerda then Empurra(Jog, -3);` |
| [210](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L210) | RNG | `O.Quadro := Random(2) + 26 + (50 - Desloc);` |
| [213](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L213) | simulation mutation | `O.VelY := 0;` |
| [214](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L214) | simulation mutation | `O.Y := O.Y + 10;` |
| [215](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L215) | simulation mutation | `if ChaoY < O.Y then O.Y := ChaoY;` |
| [216](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L216) | RNG, simulation mutation | `if P.Lado = ldDireita then O.X := Random(2) shl 3 + (P.X + 40);` |
| [217](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L217) | RNG, simulation mutation | `if P.Lado = ldEsquerda then O.X := P.X - 40 - Random(2) shl 3;` |
| [218](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L218) | simulation mutation | `P.Acertos := P.Acertos + 2;` |
| [219](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L219) | timer/branch, simulation mutation | `P.TempoAcertos := 43;` |
| [220](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L220) | RNG, simulation mutation | `if (Random(30) = 0) and (O.Vida > 0) then PausaImpacto := 5;` |
| [221](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L221) | sound, RNG | `if Random(4) = 0 then SomPendente := sndAcertoFraco;` |
| [222](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L222) | sound, RNG | `if Random(4) = 0 then SomPendente := sndAcertoForte;` |
| [223](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L223) | sound, RNG | `if Random(4) = 0 then SomPendente := sndAcertoEspecial;` |
| [224](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L224) | effect, RNG | `CriaEfeito(1, '', 0, Random(20) + (O.X - 10), Random(100) + (O.Y - 130));` |
| [225](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L225) | timer/branch | `if P.QuadroAcao > P.DuracaoAcao - 2 then` |
| [227](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L227) | simulation mutation | `P.DanoGolpe := 47;` |
| [228](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L228) | simulation mutation | `O.Vida := O.Vida - P.DanoGolpe * NivelDano;` |
| [230](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L230) | frame | `P.Quadro := Desloc + 38;` |
| [231](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L231) | simulation mutation | `PausaImpacto := 5;` |
| [232](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L232) | simulation mutation | `AcaoJogador(Jog, cmEsp2Forte);` |
| [233](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L233) | simulation mutation | `AcaoJogador(3 - Jog, cmDerrubar);` |
| [234](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L234) | simulation mutation | `P.GolpeAtivo := False;` |
| [235](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L235) | simulation mutation | `O.Recuo := 28;` |
| [236](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C02.rot#L236) | simulation mutation | `O.VelY := -30;` |

## 🌈 3. Chiquinha

- Source: [`C03.rot` lines 88–206](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L88)
- SHA/blob: `d69dc88e1d898583e4571c9e1f123add9045ba87`
- `[Quadro]` source lines: 119
- Indexed event statements: 79
- Category counts: `timer/branch` 30; `frame` 28; `effect` 8; `simulation mutation` 32; `RNG` 12; `sound` 3

| Source line | Category | Ordered source statement |
|---:|---|---|
| [93](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L93) | timer/branch, frame | `if (P.Acao = acVitoria) and (P.QuadroAcao < 5) then P.Quadro := Desloc + 33;` |
| [94](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L94) | timer/branch, frame | `if (P.Acao = acVitoria) and (P.QuadroAcao = 5) then P.Quadro := Desloc + 34;` |
| [95](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L95) | timer/branch, frame | `if (P.Acao = acVitoria) and (P.QuadroAcao = 10) then P.Quadro := Desloc + 35;` |
| [96](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L96) | timer/branch, frame | `if (P.Acao = acVitoria) and (P.QuadroAcao = 15) then P.Quadro := Desloc + 34;` |
| [97](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L97) | timer/branch, frame | `if (P.Acao = acVitoria) and (P.QuadroAcao = 20) then P.Quadro := Desloc + 33;` |
| [98](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L98) | timer/branch, frame | `if (P.Acao = acVitoria) and (P.QuadroAcao = 25) then P.Quadro := Desloc + 34;` |
| [99](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L99) | timer/branch, frame | `if (P.Acao = acVitoria) and (P.QuadroAcao = 30) then P.Quadro := Desloc + 35;` |
| [100](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L100) | timer/branch, frame | `if (P.Acao = acVitoria) and (P.QuadroAcao = 35) then P.Quadro := Desloc + 34;` |
| [101](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L101) | timer/branch, frame | `if (P.Acao = acPose) and (P.QuadroAcao = 0) then P.Quadro := Desloc + 12;` |
| [102](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L102) | timer/branch, frame | `if (P.Acao = acPose) and (P.QuadroAcao = 2) then P.Quadro := Desloc + 30;` |
| [105](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L105) | timer/branch, frame | `if P.QuadroAcao = 0 then P.Quadro := Desloc + 36;` |
| [106](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L106) | timer/branch, frame | `if P.QuadroAcao = 5 then P.Quadro := Desloc + 37;` |
| [107](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L107) | timer/branch | `if P.QuadroAcao = 10 then` |
| [109](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L109) | frame | `P.Quadro := Desloc + 38;` |
| [110](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L110) | effect | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgFraco) then CriaEfeito(9, 'A', Jog, P.X + 40, P.Y - 80);` |
| [111](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L111) | effect | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgFraco) then CriaEfeito(9, 'A', Jog, P.X - 40, P.Y - 80);` |
| [112](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L112) | effect | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgForte) then CriaEfeito(9, 'B', Jog, P.X + 40, P.Y - 80);` |
| [113](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L113) | effect | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgForte) then CriaEfeito(9, 'B', Jog, P.X - 40, P.Y - 80);` |
| [118](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L118) | timer/branch, frame | `if P.QuadroAcao = 0 then P.Quadro := Desloc + 39;` |
| [119](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L119) | timer/branch, frame | `if P.QuadroAcao = 1 then P.Quadro := Desloc + 40;` |
| [120](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L120) | timer/branch, frame | `if P.QuadroAcao = 3 then P.Quadro := Desloc + 41;` |
| [121](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L121) | timer/branch, frame | `if P.QuadroAcao = 8 then P.Quadro := Desloc + 42;` |
| [122](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L122) | simulation mutation | `if (P.ForcaGolpe = fgFraco) and (P.Lado = ldDireita) then Empurra(Jog, 6);` |
| [123](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L123) | simulation mutation | `if (P.ForcaGolpe = fgFraco) and (P.Lado = ldEsquerda) then Empurra(Jog, -6);` |
| [124](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L124) | simulation mutation | `if (P.ForcaGolpe = fgForte) and (P.Lado = ldDireita) then Empurra(Jog, 8);` |
| [125](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L125) | simulation mutation | `if (P.ForcaGolpe = fgForte) and (P.Lado = ldEsquerda) then Empurra(Jog, -8);` |
| [129](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L129) | timer/branch, frame | `if P.QuadroAcao = 0 then P.Quadro := Desloc + 43;` |
| [130](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L130) | timer/branch, frame | `if P.QuadroAcao = 3 then P.Quadro := Desloc + 44;` |
| [131](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L131) | timer/branch | `if P.QuadroAcao >= 5 then` |
| [133](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L133) | effect, RNG | `for Cont := 1 to 5 do CriaEfeito(2, '', 0, Random(40) + (P.X - 20), Random(10) + P.Y);` |
| [134](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L134) | frame | `P.Quadro := Desloc + 45;` |
| [135](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L135) | simulation mutation | `if (P.ForcaGolpe = fgFraco) and (P.Lado = ldDireita) then Empurra(Jog, 17);` |
| [136](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L136) | simulation mutation | `if (P.ForcaGolpe = fgFraco) and (P.Lado = ldEsquerda) then Empurra(Jog, -17);` |
| [137](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L137) | simulation mutation | `if (P.ForcaGolpe = fgForte) and (P.Lado = ldDireita) then Empurra(Jog, 16);` |
| [138](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L138) | simulation mutation | `if (P.ForcaGolpe = fgForte) and (P.Lado = ldEsquerda) then Empurra(Jog, -16);` |
| [143](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L143) | timer/branch, frame | `if P.QuadroAcao = 0 then P.Quadro := Desloc + 46;` |
| [144](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L144) | timer/branch, frame | `if P.QuadroAcao = 2 then P.Quadro := Desloc + 47;` |
| [145](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L145) | timer/branch, frame | `if P.QuadroAcao = 4 then P.Quadro := Desloc + 46;` |
| [146](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L146) | timer/branch, frame | `if P.QuadroAcao = 6 then P.Quadro := Desloc + 47;` |
| [147](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L147) | timer/branch, frame | `if P.QuadroAcao = 8 then P.Quadro := Desloc + 46;` |
| [148](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L148) | simulation mutation | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgFraco) then Empurra(Jog, 10);` |
| [149](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L149) | simulation mutation | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgFraco) then Empurra(Jog, -10);` |
| [150](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L150) | simulation mutation | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgForte) then Empurra(Jog, 14);` |
| [151](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L151) | simulation mutation | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgForte) then Empurra(Jog, -14);` |
| [152](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L152) | timer/branch, simulation mutation | `if P.QuadroAcao >= 5 then P.EhChute := True;` |
| [156](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L156) | effect, RNG | `for Cont := 1 to 3 do CriaEfeito(2, '', 0, Random(40) + (P.X - 20), Random(10) + P.Y);` |
| [157](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L157) | frame | `P.Quadro := Desloc + 48;` |
| [158](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L158) | simulation mutation | `if P.Lado = ldDireita then Empurra(Jog, 17);` |
| [159](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L159) | simulation mutation | `if P.Lado = ldEsquerda then Empurra(Jog, -17);` |
| [163](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L163) | simulation mutation | `P.Recuo := 0;` |
| [164](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L164) | frame | `if ContPisca mod 4 = 0 then P.Quadro := Desloc + 49` |
| [165](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L165) | frame | `else P.Quadro := Desloc + 50;` |
| [166](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L166) | simulation mutation | `P.Y := P.Y - 3;` |
| [167](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L167) | RNG, simulation mutation | `if Flash and (Random(5) = 0) then FlashVermelho := 3;` |
| [168](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L168) | RNG | `O.Quadro := Random(2) + 26 + (50 - Desloc);` |
| [171](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L171) | simulation mutation | `O.VelY := 0;` |
| [172](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L172) | simulation mutation | `O.Y := O.Y + 10;` |
| [173](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L173) | simulation mutation | `if ChaoY < O.Y then O.Y := ChaoY;` |
| [176](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L176) | RNG | `r194 := Random(3);` |
| [177](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L177) | simulation mutation | `O.X := r194 + r194 * 4 + (P.X + 65);` |
| [181](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L181) | RNG | `r195 := Random(3);` |
| [182](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L182) | simulation mutation | `O.X := P.X - 65 - (r195 + r195 * 4);` |
| [184](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L184) | simulation mutation | `P.Acertos := P.Acertos + 1;` |
| [185](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L185) | timer/branch, simulation mutation | `P.TempoAcertos := 43;` |
| [186](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L186) | RNG, simulation mutation | `if (Random(20) = 0) and (O.Vida > 0) then PausaImpacto := 5;` |
| [187](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L187) | sound, RNG | `if Random(6) = 0 then SomPendente := sndAcertoFraco;` |
| [188](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L188) | sound, RNG | `if Random(6) = 0 then SomPendente := sndAcertoForte;` |
| [189](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L189) | sound, RNG | `if Random(6) = 0 then SomPendente := sndAcertoEspecial;` |
| [190](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L190) | effect, RNG | `if P.Lado = ldDireita then CriaEfeito(1, '', 0, Random(10) + (P.X + 40), Random(30) + (P.Y - 30));` |
| [191](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L191) | effect, RNG | `if P.Lado = ldEsquerda then CriaEfeito(1, '', 0, Random(10) + (P.X - 50), Random(30) + (P.Y - 30));` |
| [192](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L192) | timer/branch | `if P.QuadroAcao > P.DuracaoAcao - 2 then` |
| [194](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L194) | simulation mutation | `P.DanoGolpe := 36;` |
| [195](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L195) | simulation mutation | `O.Vida := O.Vida - P.DanoGolpe * NivelDano;` |
| [196](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L196) | simulation mutation | `P.VelY := -28;` |
| [197](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L197) | simulation mutation | `P.NoAr := True;` |
| [199](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L199) | timer/branch | `P.QuadroAcao := 0;` |
| [200](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L200) | timer/branch | `P.DuracaoAcao := 99;` |
| [201](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L201) | simulation mutation | `AcaoJogador(3 - Jog, cmDerrubar);` |
| [202](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C03.rot#L202) | simulation mutation | `O.Recuo := 30;` |

## 🌈 4. Quico

- Source: [`C04.rot` lines 88–166](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C04.rot#L88)
- SHA/blob: `94cfe1bb5fc8983700a02cf80f74e809325cb999`
- `[Quadro]` source lines: 79
- Indexed event statements: 50
- Category counts: `timer/branch` 25; `frame` 22; `effect` 7; `simulation mutation` 15; `RNG` 3; `sound` 1

| Source line | Category | Ordered source statement |
|---:|---|---|
| [91](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C04.rot#L91) | timer/branch, frame | `if (P.Acao = acVitoria) and (P.QuadroAcao < 13) then P.Quadro := Desloc + 33;` |
| [92](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C04.rot#L92) | timer/branch, frame | `if (P.Acao = acVitoria) and (P.QuadroAcao = 13) then P.Quadro := Desloc + 34;` |
| [93](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C04.rot#L93) | timer/branch, frame | `if (P.Acao = acPose) and (P.QuadroAcao = 0) then P.Quadro := Desloc + 12;` |
| [94](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C04.rot#L94) | timer/branch, frame | `if (P.Acao = acPose) and (P.QuadroAcao = 2) then P.Quadro := Desloc + 30;` |
| [97](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C04.rot#L97) | timer/branch, frame | `if P.QuadroAcao = 0 then P.Quadro := Desloc + 35;` |
| [98](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C04.rot#L98) | timer/branch, frame | `if P.QuadroAcao = 5 then P.Quadro := Desloc + 36;` |
| [99](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C04.rot#L99) | timer/branch | `if P.QuadroAcao = 10 then` |
| [101](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C04.rot#L101) | frame | `P.Quadro := Desloc + 37;` |
| [102](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C04.rot#L102) | effect | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgFraco) then CriaEfeito(12, 'A', Jog, P.X + 40, P.Y - 110);` |
| [103](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C04.rot#L103) | effect | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgFraco) then CriaEfeito(12, 'A', Jog, P.X - 40, P.Y - 110);` |
| [104](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C04.rot#L104) | effect | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgForte) then CriaEfeito(12, 'B', Jog, P.X + 40, P.Y - 110);` |
| [105](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C04.rot#L105) | effect | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgForte) then CriaEfeito(12, 'B', Jog, P.X - 40, P.Y - 110);` |
| [110](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C04.rot#L110) | timer/branch, frame | `if P.QuadroAcao = 0 then P.Quadro := Desloc + 35;` |
| [111](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C04.rot#L111) | timer/branch, frame | `if P.QuadroAcao = 4 then P.Quadro := Desloc + 36;` |
| [112](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C04.rot#L112) | timer/branch | `if P.QuadroAcao >= 7 then` |
| [114](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C04.rot#L114) | frame | `P.Quadro := Desloc + 38;` |
| [115](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C04.rot#L115) | simulation mutation | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgFraco) then Empurra(Jog, 26);` |
| [116](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C04.rot#L116) | simulation mutation | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgFraco) then Empurra(Jog, -26);` |
| [117](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C04.rot#L117) | simulation mutation | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgForte) then Empurra(Jog, 25);` |
| [118](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C04.rot#L118) | simulation mutation | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgForte) then Empurra(Jog, -25);` |
| [120](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C04.rot#L120) | effect, RNG | `for Cont := 1 to 3 do CriaEfeito(2, '', 0, Random(40) + (P.X - 20), Random(10) + P.Y);` |
| [124](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C04.rot#L124) | timer/branch, frame | `if P.QuadroAcao = 0 then P.Quadro := Desloc + 12;` |
| [125](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C04.rot#L125) | timer/branch, frame | `if P.QuadroAcao = 2 then P.Quadro := Desloc + 39;` |
| [126](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C04.rot#L126) | timer/branch, frame | `if P.QuadroAcao = 6 then P.Quadro := Desloc + 40;` |
| [127](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C04.rot#L127) | timer/branch, frame | `if P.QuadroAcao = 9 then P.Quadro := Desloc + 41;` |
| [128](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C04.rot#L128) | simulation mutation | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgFraco) then Empurra(Jog, 10);` |
| [129](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C04.rot#L129) | simulation mutation | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgFraco) then Empurra(Jog, -10);` |
| [130](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C04.rot#L130) | simulation mutation | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgForte) then Empurra(Jog, 13);` |
| [131](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C04.rot#L131) | simulation mutation | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgForte) then Empurra(Jog, -13);` |
| [132](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C04.rot#L132) | timer/branch, simulation mutation | `if P.QuadroAcao >= 6 then P.EhChute := True;` |
| [136](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C04.rot#L136) | timer/branch, frame | `if P.QuadroAcao = 0 then P.Quadro := Desloc + 42;` |
| [137](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C04.rot#L137) | timer/branch, frame | `if P.QuadroAcao = 2 then P.Quadro := Desloc + 43;` |
| [138](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C04.rot#L138) | timer/branch, frame | `if P.QuadroAcao = 4 then P.Quadro := Desloc + 44;` |
| [139](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C04.rot#L139) | timer/branch, frame | `if P.QuadroAcao = 6 then P.Quadro := Desloc + 45;` |
| [140](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C04.rot#L140) | timer/branch | `if (P.QuadroAcao >= 2) and (P.QuadroAcao < 6) then` |
| [142](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C04.rot#L142) | simulation mutation | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgFraco) then Empurra(Jog, 9);` |
| [143](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C04.rot#L143) | simulation mutation | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgFraco) then Empurra(Jog, -9);` |
| [144](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C04.rot#L144) | simulation mutation | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgForte) then Empurra(Jog, 18);` |
| [145](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C04.rot#L145) | simulation mutation | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgForte) then Empurra(Jog, -18);` |
| [150](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C04.rot#L150) | timer/branch | `if P.QuadroAcao = 0 then` |
| [152](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C04.rot#L152) | frame | `P.Quadro := Desloc + 12;` |
| [153](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C04.rot#L153) | sound | `SomPendente := 28;` |
| [155](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C04.rot#L155) | timer/branch, frame | `if P.QuadroAcao = 1 then P.Quadro := Desloc + 39;` |
| [156](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C04.rot#L156) | timer/branch, frame | `if P.QuadroAcao = 5 then P.Quadro := Desloc + 40;` |
| [157](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C04.rot#L157) | timer/branch, frame | `if P.QuadroAcao = 9 then P.Quadro := Desloc + 41;` |
| [158](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C04.rot#L158) | simulation mutation | `if P.Lado = ldDireita then Empurra(Jog, 17);` |
| [159](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C04.rot#L159) | simulation mutation | `if P.Lado = ldEsquerda then Empurra(Jog, -17);` |
| [160](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C04.rot#L160) | timer/branch | `if (P.QuadroAcao > 1) and (P.QuadroAcao < 10) then` |
| [163](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C04.rot#L163) | effect, RNG | `if P.Lado = ldDireita then CriaEfeito(11, '', 0, Random(30) + (P.X + 20), Random(40) + (P.Y - 110));` |
| [164](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C04.rot#L164) | effect, RNG | `if P.Lado = ldEsquerda then CriaEfeito(11, '', 0, P.X - 20 - Random(30), Random(40) + (P.Y - 110));` |

## 🌈 5. Dona Florinda

- Source: [`C05.rot` lines 87–173](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L87)
- SHA/blob: `27a221792cb8ac4fc685a8fe510dbc871afeef5a`
- `[Quadro]` source lines: 87
- Indexed event statements: 61
- Category counts: `timer/branch` 45; `frame` 36; `effect` 4; `simulation mutation` 15; `RNG` 3; `sound` 2

| Source line | Category | Ordered source statement |
|---:|---|---|
| [90](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L90) | timer/branch, frame | `if (P.Acao = acVitoria) and (P.QuadroAcao < 5) then P.Quadro := Desloc + 33;` |
| [91](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L91) | timer/branch, frame | `if (P.Acao = acVitoria) and (P.QuadroAcao = 9) then P.Quadro := Desloc + 34;` |
| [92](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L92) | timer/branch, frame | `if (P.Acao = acVitoria) and (P.QuadroAcao = 13) then P.Quadro := Desloc + 33;` |
| [93](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L93) | timer/branch, frame | `if (P.Acao = acVitoria) and (P.QuadroAcao = 17) then P.Quadro := Desloc + 34;` |
| [94](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L94) | timer/branch, frame | `if (P.Acao = acVitoria) and (P.QuadroAcao = 21) then P.Quadro := Desloc + 33;` |
| [95](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L95) | timer/branch, frame | `if (P.Acao = acVitoria) and (P.QuadroAcao = 25) then P.Quadro := Desloc + 34;` |
| [96](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L96) | timer/branch, frame | `if (P.Acao = acVitoria) and (P.QuadroAcao = 29) then P.Quadro := Desloc + 33;` |
| [97](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L97) | timer/branch, frame | `if (P.Acao = acVitoria) and (P.QuadroAcao = 33) then P.Quadro := Desloc + 34;` |
| [98](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L98) | timer/branch, frame | `if (P.Acao = acVitoria) and (P.QuadroAcao = 37) then P.Quadro := Desloc + 33;` |
| [99](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L99) | timer/branch, frame | `if (P.Acao = acVitoria) and (P.QuadroAcao = 41) then P.Quadro := Desloc + 35;` |
| [100](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L100) | timer/branch, frame | `if (P.Acao = acPose) and (P.QuadroAcao = 0) then P.Quadro := Desloc + 12;` |
| [101](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L101) | timer/branch, frame | `if (P.Acao = acPose) and (P.QuadroAcao = 2) then P.Quadro := Desloc + 30;` |
| [104](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L104) | timer/branch, frame | `if P.QuadroAcao = 0 then P.Quadro := Desloc + 36;` |
| [105](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L105) | timer/branch, frame | `if P.QuadroAcao = 5 then P.Quadro := Desloc + 37;` |
| [106](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L106) | timer/branch | `if P.QuadroAcao = 10 then` |
| [108](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L108) | frame | `P.Quadro := Desloc + 38;` |
| [109](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L109) | effect | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgFraco) then CriaEfeito(13, 'A', Jog, P.X + 40, P.Y - 103);` |
| [110](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L110) | effect | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgFraco) then CriaEfeito(13, 'A', Jog, P.X - 40, P.Y - 103);` |
| [111](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L111) | effect | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgForte) then CriaEfeito(13, 'B', Jog, P.X + 40, P.Y - 103);` |
| [112](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L112) | effect | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgForte) then CriaEfeito(13, 'B', Jog, P.X - 40, P.Y - 103);` |
| [117](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L117) | timer/branch, frame | `if P.QuadroAcao = 0 then P.Quadro := Desloc + 39;` |
| [118](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L118) | timer/branch, frame | `if P.QuadroAcao = 5 then P.Quadro := Desloc + 40;` |
| [119](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L119) | timer/branch, frame | `if P.QuadroAcao = 10 then P.Quadro := Desloc + 41;` |
| [120](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L120) | timer/branch, frame | `if P.QuadroAcao = 15 then P.Quadro := Desloc + 42;` |
| [121](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L121) | timer/branch, frame | `if P.QuadroAcao = 20 then P.Quadro := Desloc + 12;` |
| [122](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L122) | timer/branch | `if P.QuadroAcao < 13 then` |
| [124](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L124) | simulation mutation | `if (P.ForcaGolpe = fgFraco) and (P.Lado = ldDireita) then Empurra(Jog, 6);` |
| [125](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L125) | simulation mutation | `if (P.ForcaGolpe = fgFraco) and (P.Lado = ldEsquerda) then Empurra(Jog, -6);` |
| [126](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L126) | simulation mutation | `if (P.ForcaGolpe = fgForte) and (P.Lado = ldDireita) then Empurra(Jog, 10);` |
| [127](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L127) | simulation mutation | `if (P.ForcaGolpe = fgForte) and (P.Lado = ldEsquerda) then Empurra(Jog, -10);` |
| [129](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L129) | timer/branch, simulation mutation | `if P.QuadroAcao >= 15 then P.EhChute := True;` |
| [133](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L133) | timer/branch, frame | `if P.QuadroAcao mod 2 = 0 then P.Quadro := Desloc + 43;` |
| [134](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L134) | timer/branch, frame | `if P.QuadroAcao mod 2 <> 0 then P.Quadro := Desloc + 44;` |
| [135](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L135) | timer/branch, frame | `if P.QuadroAcao = 0 then P.Quadro := Desloc + 39;` |
| [136](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L136) | timer/branch, frame | `if P.QuadroAcao = 2 then P.Quadro := Desloc + 40;` |
| [137](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L137) | timer/branch, frame | `if P.QuadroAcao = 35 then P.Quadro := Desloc + 40;` |
| [138](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L138) | timer/branch, frame | `if P.QuadroAcao = 38 then P.Quadro := Desloc + 39;` |
| [139](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L139) | timer/branch | `if (P.QuadroAcao >= 4) and (P.QuadroAcao < 35) then` |
| [141](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L141) | simulation mutation | `if (P.ForcaGolpe = fgFraco) and (P.Lado = ldDireita) then Empurra(Jog, 7);` |
| [142](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L142) | simulation mutation | `if (P.ForcaGolpe = fgFraco) and (P.Lado = ldEsquerda) then Empurra(Jog, -7);` |
| [143](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L143) | simulation mutation | `if (P.ForcaGolpe = fgForte) and (P.Lado = ldDireita) then Empurra(Jog, 9);` |
| [144](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L144) | simulation mutation | `if (P.ForcaGolpe = fgForte) and (P.Lado = ldEsquerda) then Empurra(Jog, -9);` |
| [146](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L146) | timer/branch, simulation mutation | `if P.QuadroAcao >= 26 then P.EhChute := True;` |
| [151](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L151) | RNG | `S_0 := Random(3) + (Desloc + 45);` |
| [153](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L153) | frame | `P.Quadro := S_0;` |
| [154](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L154) | sound, RNG | `if Random(5) = 0 then SomPendente := 32;` |
| [158](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L158) | timer/branch, frame | `if P.QuadroAcao = 0 then P.Quadro := Desloc + 7;` |
| [159](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L159) | timer/branch, frame | `if P.QuadroAcao = 1 then P.Quadro := Desloc + 10;` |
| [160](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L160) | timer/branch, frame | `if P.QuadroAcao = 2 then P.Quadro := Desloc + 11;` |
| [161](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L161) | timer/branch, frame | `if P.QuadroAcao = 3 then P.Quadro := Desloc + 6;` |
| [162](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L162) | timer/branch, frame | `if P.QuadroAcao = 4 then P.Quadro := Desloc + 7;` |
| [163](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L163) | timer/branch, frame | `if P.QuadroAcao = 5 then P.Quadro := Desloc + 10;` |
| [164](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L164) | timer/branch, frame | `if P.QuadroAcao = 6 then P.Quadro := Desloc + 11;` |
| [165](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L165) | timer/branch, frame, RNG | `if (P.QuadroAcao >= 8) and (P.QuadroAcao < 28) then P.Quadro := Random(3) + (Desloc + 45);` |
| [166](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L166) | timer/branch, frame | `if P.QuadroAcao = 28 then P.Quadro := Desloc + 6;` |
| [167](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L167) | timer/branch, simulation mutation | `if P.QuadroAcao >= 15 then P.EhChute := True;` |
| [168](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L168) | timer/branch, sound | `if P.QuadroAcao in [1, 8, 12] then SomPendente := 32;` |
| [169](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L169) | timer/branch, simulation mutation | `if (P.Lado = ldDireita) and (P.QuadroAcao < 8) then Empurra(Jog, 5);` |
| [170](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L170) | timer/branch, simulation mutation | `if (P.Lado = ldDireita) and (P.QuadroAcao >= 8) and (P.QuadroAcao < 28) then Empurra(Jog, 10);` |
| [171](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L171) | timer/branch, simulation mutation | `if (P.Lado = ldEsquerda) and (P.QuadroAcao < 8) then Empurra(Jog, -5);` |
| [172](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C05.rot#L172) | timer/branch, simulation mutation | `if (P.Lado = ldEsquerda) and (P.QuadroAcao >= 8) and (P.QuadroAcao < 28) then Empurra(Jog, -10);` |

## 🌈 6. Prof. Girafales

- Source: [`C06.rot` lines 85–199](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L85)
- SHA/blob: `238faecff436d188c96ddd9d7865ceaabfb289cc`
- `[Quadro]` source lines: 115
- Indexed event statements: 75
- Category counts: `timer/branch` 33; `frame` 22; `effect` 8; `simulation mutation` 32; `sound` 1

| Source line | Category | Ordered source statement |
|---:|---|---|
| [86](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L86) | timer/branch | `if (P.Acao = acVitoria) and (P.QuadroAcao mod 4 = 0) then` |
| [88](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L88) | frame | `if P.Quadro = Desloc + 33 then P.Quadro := Desloc + 34` |
| [89](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L89) | frame | `else P.Quadro := Desloc + 33;` |
| [91](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L91) | timer/branch, frame | `if (P.Acao = acPose) and (P.QuadroAcao = 0) then P.Quadro := Desloc + 12;` |
| [92](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L92) | timer/branch, frame | `if (P.Acao = acPose) and (P.QuadroAcao = 2) then P.Quadro := Desloc + 30;` |
| [95](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L95) | timer/branch, frame | `if P.QuadroAcao = 0 then P.Quadro := Desloc + 35;` |
| [96](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L96) | timer/branch, frame | `if P.QuadroAcao = 5 then P.Quadro := Desloc + 36;` |
| [97](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L97) | timer/branch | `if P.QuadroAcao = 10 then` |
| [99](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L99) | frame | `P.Quadro := Desloc + 37;` |
| [100](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L100) | effect | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgFraco) then CriaEfeito(15, 'A', Jog, P.X + 50, P.Y - 130);` |
| [101](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L101) | effect | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgFraco) then CriaEfeito(15, 'A', Jog, P.X - 50, P.Y - 130);` |
| [102](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L102) | effect | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgForte) then CriaEfeito(15, 'B', Jog, P.X + 50, P.Y - 130);` |
| [103](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L103) | effect | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgForte) then CriaEfeito(15, 'B', Jog, P.X - 50, P.Y - 130);` |
| [108](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L108) | timer/branch, frame | `if P.QuadroAcao = 0 then P.Quadro := Desloc + 38;` |
| [109](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L109) | timer/branch, frame | `if P.QuadroAcao = 5 then P.Quadro := Desloc + 39;` |
| [110](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L110) | timer/branch | `if P.QuadroAcao = 10 then` |
| [112](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L112) | frame | `P.Quadro := Desloc + 40;` |
| [113](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L113) | effect | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgFraco) then CriaEfeito(15, 'A', Jog, P.X + 50, P.Y - 75);` |
| [114](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L114) | effect | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgFraco) then CriaEfeito(15, 'A', Jog, P.X - 50, P.Y - 75);` |
| [115](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L115) | effect | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgForte) then CriaEfeito(15, 'B', Jog, P.X + 50, P.Y - 75);` |
| [116](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L116) | effect | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgForte) then CriaEfeito(15, 'B', Jog, P.X - 50, P.Y - 75);` |
| [121](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L121) | timer/branch, frame | `if P.QuadroAcao = 0 then P.Quadro := Desloc + 41;` |
| [122](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L122) | timer/branch, frame | `if P.QuadroAcao = 3 then P.Quadro := Desloc + 42;` |
| [123](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L123) | timer/branch, frame | `if P.QuadroAcao = 9 then P.Quadro := Desloc + 43;` |
| [124](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L124) | timer/branch | `if (P.QuadroAcao >= 3) and (P.QuadroAcao <= 12) then` |
| [126](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L126) | simulation mutation | `if (P.ForcaGolpe = fgFraco) and (P.Lado = ldDireita) then Empurra(Jog, 6);` |
| [127](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L127) | simulation mutation | `if (P.ForcaGolpe = fgFraco) and (P.Lado = ldEsquerda) then Empurra(Jog, -6);` |
| [128](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L128) | simulation mutation | `if (P.ForcaGolpe = fgForte) and (P.Lado = ldDireita) then Empurra(Jog, 9);` |
| [129](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L129) | simulation mutation | `if (P.ForcaGolpe = fgForte) and (P.Lado = ldEsquerda) then Empurra(Jog, -9);` |
| [130](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L130) | simulation mutation | `if P.ForcaGolpe = fgFraco then P.Y := P.Y - 10;` |
| [131](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L131) | simulation mutation | `if P.ForcaGolpe = fgForte then P.Y := P.Y - 13;` |
| [133](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L133) | timer/branch | `if P.QuadroAcao = 13 then` |
| [135](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L135) | simulation mutation | `if P.ForcaGolpe = fgFraco then P.VelY := -3;` |
| [136](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L136) | simulation mutation | `if P.ForcaGolpe = fgForte then P.VelY := -6;` |
| [137](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L137) | simulation mutation | `P.NoAr := True;` |
| [139](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L139) | simulation mutation | `P.EhChute := True;` |
| [143](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L143) | simulation mutation | `if P.Lado = ldDireita then Empurra(Jog, 2);` |
| [144](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L144) | simulation mutation | `if P.Lado = ldEsquerda then Empurra(Jog, -2);` |
| [145](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L145) | timer/branch, frame | `if P.QuadroAcao = 0 then P.Quadro := Desloc + 41;` |
| [146](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L146) | timer/branch, frame | `if P.QuadroAcao = 2 then P.Quadro := Desloc + 44;` |
| [147](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L147) | timer/branch, frame | `if P.QuadroAcao = 7 then P.Quadro := Desloc + 41;` |
| [148](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L148) | timer/branch | `if (P.QuadroAcao >= 2) and (P.QuadroAcao <= 5) then` |
| [150](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L150) | simulation mutation | `if (P.ForcaGolpe = fgFraco) and (P.Lado = ldDireita) then Empurra(Jog, 20);` |
| [151](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L151) | simulation mutation | `if (P.ForcaGolpe = fgFraco) and (P.Lado = ldEsquerda) then Empurra(Jog, -20);` |
| [152](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L152) | simulation mutation | `if (P.ForcaGolpe = fgForte) and (P.Lado = ldDireita) then Empurra(Jog, 27);` |
| [153](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L153) | simulation mutation | `if (P.ForcaGolpe = fgForte) and (P.Lado = ldEsquerda) then Empurra(Jog, -27);` |
| [154](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L154) | simulation mutation | `P.Y := P.Y - 15;` |
| [156](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L156) | timer/branch, simulation mutation | `if P.QuadroAcao >= 3 then P.EhChute := True;` |
| [157](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L157) | timer/branch, simulation mutation | `if P.QuadroAcao = 6 then P.NoAr := True;` |
| [161](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L161) | timer/branch | `if P.QuadroAcao <= 8 then` |
| [163](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L163) | simulation mutation | `if P.Lado = ldDireita then Empurra(Jog, 1);` |
| [164](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L164) | simulation mutation | `if P.Lado = ldEsquerda then Empurra(Jog, -1);` |
| [166](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L166) | timer/branch, frame | `if P.QuadroAcao = 0 then P.Quadro := Desloc + 41;` |
| [167](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L167) | timer/branch, frame | `if P.QuadroAcao = 3 then P.Quadro := Desloc + 44;` |
| [168](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L168) | timer/branch, frame | `if P.QuadroAcao = 7 then P.Quadro := Desloc + 41;` |
| [169](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L169) | timer/branch | `if (P.QuadroAcao >= 3) and (P.QuadroAcao <= 6) then` |
| [171](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L171) | simulation mutation | `if P.Lado = ldDireita then Empurra(Jog, 21);` |
| [172](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L172) | simulation mutation | `if P.Lado = ldEsquerda then Empurra(Jog, -21);` |
| [174](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L174) | timer/branch | `if P.QuadroAcao = 8 then` |
| [176](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L176) | simulation mutation | `P.NoAr := False;` |
| [177](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L177) | simulation mutation | `P.Y := ChaoY;` |
| [179](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L179) | timer/branch, sound | `if P.QuadroAcao = 8 then SomPendente := 34;` |
| [180](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L180) | timer/branch, frame | `if P.QuadroAcao = 8 then P.Quadro := Desloc + 41;` |
| [181](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L181) | timer/branch, frame | `if P.QuadroAcao = 9 then P.Quadro := Desloc + 42;` |
| [182](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L182) | timer/branch, frame | `if P.QuadroAcao = 25 then P.Quadro := Desloc + 43;` |
| [183](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L183) | timer/branch | `if (P.QuadroAcao >= 9) and (P.QuadroAcao <= 25) then` |
| [185](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L185) | simulation mutation | `if P.Lado = ldDireita then Empurra(Jog, 9);` |
| [186](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L186) | simulation mutation | `if P.Lado = ldEsquerda then Empurra(Jog, -9);` |
| [187](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L187) | simulation mutation | `P.Y := P.Y - 10;` |
| [189](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L189) | timer/branch | `if P.QuadroAcao = 26 then` |
| [191](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L191) | simulation mutation | `P.NoAr := True;` |
| [192](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L192) | simulation mutation | `P.VelY := -5;` |
| [194](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L194) | timer/branch | `if (P.QuadroAcao >= 9) and (O.Acao in [acApanhaAlto, acDerrubado]) then` |
| [196](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L196) | simulation mutation | `P.EhChute := True;` |
| [197](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C06.rot#L197) | simulation mutation | `O.Invulneravel := False;` |

## 🌈 7. Sr. Barriga

- Source: [`C07.rot` lines 96–200](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L96)
- SHA/blob: `3cef84b09fce2d677b88e15d47edc926e7ca8ff5`
- `[Quadro]` source lines: 105
- Indexed event statements: 76
- Category counts: `timer/branch` 32; `frame` 28; `simulation mutation` 38; `effect` 2; `RNG` 11; `sound` 3

| Source line | Category | Ordered source statement |
|---:|---|---|
| [99](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L99) | timer/branch, frame | `if (P.Acao = acVitoria) and (P.QuadroAcao mod 2 = 0) then P.Quadro := Desloc + 33;` |
| [100](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L100) | timer/branch, frame | `if (P.Acao = acVitoria) and (P.QuadroAcao mod 2 <> 0) then P.Quadro := Desloc + 34;` |
| [101](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L101) | timer/branch, frame | `if (P.Acao = acPose) and (P.QuadroAcao = 0) then P.Quadro := Desloc + 12;` |
| [102](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L102) | timer/branch, frame | `if (P.Acao = acPose) and (P.QuadroAcao = 2) then P.Quadro := Desloc + 30;` |
| [105](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L105) | timer/branch, frame | `if P.QuadroAcao = 0 then P.Quadro := Desloc + 35;` |
| [106](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L106) | timer/branch, frame | `if P.QuadroAcao = 2 then P.Quadro := Desloc + 36;` |
| [107](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L107) | timer/branch, frame | `if P.QuadroAcao = 4 then P.Quadro := Desloc + 37;` |
| [108](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L108) | timer/branch, frame | `if P.QuadroAcao = 6 then P.Quadro := Desloc + 35;` |
| [109](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L109) | timer/branch, frame | `if P.QuadroAcao = 8 then P.Quadro := Desloc + 36;` |
| [110](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L110) | timer/branch, frame | `if P.QuadroAcao = 10 then P.Quadro := Desloc + 37;` |
| [111](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L111) | timer/branch, frame | `if P.QuadroAcao = 12 then P.Quadro := Desloc + 35;` |
| [112](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L112) | timer/branch, frame | `if P.QuadroAcao = 14 then P.Quadro := Desloc + 36;` |
| [113](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L113) | timer/branch, frame | `if P.QuadroAcao = 16 then P.Quadro := Desloc + 37;` |
| [114](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L114) | simulation mutation | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgFraco) then Empurra(Jog, 2);` |
| [115](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L115) | simulation mutation | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgFraco) then Empurra(Jog, -2);` |
| [116](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L116) | simulation mutation | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgForte) then Empurra(Jog, 4);` |
| [117](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L117) | simulation mutation | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgForte) then Empurra(Jog, -4);` |
| [121](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L121) | timer/branch, frame | `if P.QuadroAcao = 0 then P.Quadro := Desloc + 38;` |
| [122](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L122) | timer/branch, frame | `if P.QuadroAcao = 2 then P.Quadro := Desloc + 39;` |
| [123](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L123) | timer/branch, frame | `if P.QuadroAcao = 4 then P.Quadro := Desloc + 40;` |
| [124](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L124) | timer/branch, frame | `if P.QuadroAcao = 6 then P.Quadro := Desloc + 41;` |
| [125](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L125) | timer/branch, frame | `if P.QuadroAcao = 8 then P.Quadro := Desloc + 40;` |
| [126](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L126) | timer/branch, frame | `if P.QuadroAcao = 10 then P.Quadro := Desloc + 39;` |
| [127](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L127) | timer/branch, simulation mutation | `if P.QuadroAcao >= 5 then P.EhChute := True;` |
| [131](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L131) | timer/branch, frame | `if P.QuadroAcao = 0 then P.Quadro := Desloc + 42;` |
| [132](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L132) | timer/branch, frame | `if P.QuadroAcao = 2 then P.Quadro := Desloc + 43;` |
| [133](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L133) | timer/branch, frame | `if P.QuadroAcao = 4 then P.Quadro := Desloc + 44;` |
| [134](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L134) | timer/branch | `if (P.QuadroAcao > 0) and (P.QuadroAcao < 5) then` |
| [136](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L136) | simulation mutation | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgFraco) then Empurra(Jog, 7);` |
| [137](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L137) | simulation mutation | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgFraco) then Empurra(Jog, -7);` |
| [138](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L138) | simulation mutation | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgForte) then Empurra(Jog, 12);` |
| [139](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L139) | simulation mutation | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgForte) then Empurra(Jog, -12);` |
| [141](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L141) | timer/branch, simulation mutation | `if P.QuadroAcao >= 3 then P.EhChute := True;` |
| [145](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L145) | timer/branch, frame | `if P.QuadroAcao = 0 then P.Quadro := Desloc + 45;` |
| [146](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L146) | timer/branch, frame | `if P.QuadroAcao = 2 then P.Quadro := Desloc + 46;` |
| [147](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L147) | simulation mutation | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgFraco) then Empurra(Jog, 6);` |
| [148](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L148) | simulation mutation | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgFraco) then Empurra(Jog, -6);` |
| [149](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L149) | simulation mutation | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgForte) then Empurra(Jog, 11);` |
| [150](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L150) | simulation mutation | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgForte) then Empurra(Jog, -11);` |
| [151](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L151) | simulation mutation | `P.EhChute := True;` |
| [153](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L153) | timer/branch, frame | `if (P.Acao = acEspecial5) and (P.QuadroAcao = 0) then P.Quadro := Desloc + 38;` |
| [154](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L154) | timer/branch | `if (P.Acao = acEspecial5) and (P.QuadroAcao > 2) then` |
| [156](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L156) | frame | `P.Quadro := Desloc + 38;` |
| [157](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L157) | simulation mutation | `if P.Lado = ldDireita then Empurra(Jog, 25);` |
| [158](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L158) | simulation mutation | `if P.Lado = ldEsquerda then Empurra(Jog, -25);` |
| [159](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L159) | effect, RNG | `for Cont := 1 to 3 do CriaEfeito(2, '', 0, Random(40) + (P.X - 20), Random(10) + P.Y);` |
| [163](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L163) | simulation mutation | `P.Recuo := 0;` |
| [164](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L164) | frame, RNG | `P.Quadro := Random(46) + 1 + Desloc;` |
| [165](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L165) | RNG, simulation mutation | `if Flash and (Random(5) = 0) then FlashVermelho := 3;` |
| [166](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L166) | RNG, simulation mutation | `if Random(3) = 0 then PausaImpacto := Random(3) + 3;` |
| [167](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L167) | simulation mutation | `if P.Lado = ldDireita then Empurra(Jog, 3);` |
| [168](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L168) | simulation mutation | `if P.Lado = ldEsquerda then Empurra(Jog, -3);` |
| [169](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L169) | RNG | `O.Quadro := Random(2) + 26 + (50 - Desloc);` |
| [172](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L172) | simulation mutation | `O.VelY := 0;` |
| [173](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L173) | simulation mutation | `O.Y := O.Y + 10;` |
| [174](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L174) | simulation mutation | `if ChaoY < O.Y then O.Y := ChaoY;` |
| [175](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L175) | RNG, simulation mutation | `if P.Lado = ldDireita then O.X := Random(2) shl 3 + (P.X + 40);` |
| [176](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L176) | RNG, simulation mutation | `if P.Lado = ldEsquerda then O.X := P.X - 40 - Random(2) shl 3;` |
| [177](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L177) | simulation mutation | `P.Acertos := P.Acertos + 1;` |
| [178](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L178) | timer/branch, simulation mutation | `P.TempoAcertos := 43;` |
| [179](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L179) | sound, RNG | `if Random(4) = 0 then SomPendente := sndAcertoFraco;` |
| [180](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L180) | sound, RNG | `if Random(4) = 0 then SomPendente := sndAcertoForte;` |
| [181](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L181) | sound, RNG | `if Random(4) = 0 then SomPendente := sndAcertoEspecial;` |
| [182](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L182) | effect, RNG | `CriaEfeito(1, '', 0, Random(40) + (O.X - 20), Random(100) + (O.Y - 150));` |
| [183](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L183) | timer/branch | `if P.QuadroAcao > P.DuracaoAcao - 2 then` |
| [185](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L185) | simulation mutation | `P.DanoGolpe := 36;` |
| [186](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L186) | simulation mutation | `O.Vida := O.Vida - P.DanoGolpe * NivelDano;` |
| [188](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L188) | frame | `P.Quadro := Desloc + 12;` |
| [189](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L189) | simulation mutation | `if O.Vida > 0 then PausaImpacto := 5;` |
| [190](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L190) | timer/branch | `P.QuadroAcao := 0;` |
| [191](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L191) | simulation mutation | `P.VelY := -48;` |
| [192](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L192) | simulation mutation | `P.NoAr := True;` |
| [193](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L193) | simulation mutation | `AcaoJogador(Jog, cmEsp3Forte);` |
| [194](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L194) | simulation mutation | `AcaoJogador(3 - Jog, cmDerrubar);` |
| [195](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L195) | simulation mutation | `O.Recuo := 30;` |
| [196](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C07.rot#L196) | simulation mutation | `O.VelY := -35;` |

## 🌈 8. Bruxa do 71

- Source: [`C08.rot` lines 78–194](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L78)
- SHA/blob: `38c419da7364fcc5a79654c5facace2b9da38900`
- `[Quadro]` source lines: 117
- Indexed event statements: 78
- Category counts: `frame` 21; `timer/branch` 36; `simulation mutation` 34; `effect` 12; `sound` 1

| Source line | Category | Ordered source statement |
|---:|---|---|
| [81](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L81) | frame | `P.Quadro := Desloc + 33;` |
| [82](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L82) | timer/branch, simulation mutation | `if P.QuadroAcao mod 2 = 0 then P.Y := P.Y - 4;` |
| [83](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L83) | timer/branch, simulation mutation | `if P.QuadroAcao mod 2 <> 0 then P.Y := P.Y + 4;` |
| [84](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L84) | simulation mutation | `if ChaoY < P.Y then P.Y := ChaoY;` |
| [86](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L86) | timer/branch, frame | `if (P.Acao = acPose) and (P.QuadroAcao = 0) then P.Quadro := Desloc + 12;` |
| [87](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L87) | timer/branch, frame | `if (P.Acao = acPose) and (P.QuadroAcao = 2) then P.Quadro := Desloc + 30;` |
| [90](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L90) | timer/branch, frame | `if P.QuadroAcao = 0 then P.Quadro := Desloc + 34;` |
| [91](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L91) | timer/branch, frame | `if P.QuadroAcao = 5 then P.Quadro := Desloc + 35;` |
| [92](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L92) | timer/branch | `if P.QuadroAcao = 10 then` |
| [94](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L94) | frame | `P.Quadro := Desloc + 36;` |
| [95](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L95) | effect | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgFraco) then CriaEfeito(18, 'A', Jog, P.X + 40, P.Y - 90);` |
| [96](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L96) | effect | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgFraco) then CriaEfeito(18, 'A', Jog, P.X - 40, P.Y - 90);` |
| [97](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L97) | effect | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgForte) then CriaEfeito(18, 'B', Jog, P.X + 40, P.Y - 90);` |
| [98](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L98) | effect | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgForte) then CriaEfeito(18, 'B', Jog, P.X - 60, P.Y - 90);` |
| [103](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L103) | timer/branch | `if P.QuadroAcao = 0 then` |
| [105](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L105) | simulation mutation | `P.VelY := 0;` |
| [106](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L106) | simulation mutation | `P.NoAr := False;` |
| [108](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L108) | timer/branch, frame | `if P.QuadroAcao = 0 then P.Quadro := Desloc + 34;` |
| [109](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L109) | timer/branch, frame | `if P.QuadroAcao = 5 then P.Quadro := Desloc + 37;` |
| [110](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L110) | timer/branch | `if P.QuadroAcao = 10 then` |
| [112](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L112) | frame | `P.Quadro := Desloc + 38;` |
| [113](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L113) | effect | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgFraco) then CriaEfeito(19, 'A', Jog, P.X + 35, P.Y - 48);` |
| [114](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L114) | effect | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgFraco) then CriaEfeito(19, 'A', Jog, P.X - 35, P.Y - 48);` |
| [115](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L115) | effect | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgForte) then CriaEfeito(19, 'B', Jog, P.X + 35, P.Y - 48);` |
| [116](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L116) | effect | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgForte) then CriaEfeito(19, 'B', Jog, P.X - 35, P.Y - 48);` |
| [118](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L118) | timer/branch | `if P.QuadroAcao = 8 then` |
| [120](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L120) | simulation mutation | `P.VelY := 0;` |
| [121](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L121) | simulation mutation | `P.NoAr := False;` |
| [126](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L126) | timer/branch, frame | `if P.QuadroAcao = 0 then P.Quadro := Desloc + 39;` |
| [127](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L127) | timer/branch, frame | `if P.QuadroAcao = 2 then P.Quadro := Desloc + 40;` |
| [128](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L128) | timer/branch, frame | `if P.QuadroAcao = 4 then P.Quadro := Desloc + 41;` |
| [129](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L129) | timer/branch, frame | `if P.QuadroAcao = 6 then P.Quadro := Desloc + 40;` |
| [130](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L130) | timer/branch, frame | `if P.QuadroAcao = 8 then P.Quadro := Desloc + 41;` |
| [131](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L131) | timer/branch, frame | `if P.QuadroAcao = 10 then P.Quadro := Desloc + 40;` |
| [132](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L132) | timer/branch, simulation mutation | `if (P.QuadroAcao = 2) and (P.ForcaGolpe = fgFraco) then P.VelY := -25;` |
| [133](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L133) | timer/branch, simulation mutation | `if (P.QuadroAcao = 2) and (P.ForcaGolpe = fgForte) then P.VelY := -32;` |
| [134](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L134) | timer/branch | `if P.QuadroAcao = 2 then` |
| [136](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L136) | simulation mutation | `P.DirPulo := puloNenhum;` |
| [137](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L137) | simulation mutation | `P.NoAr := True;` |
| [139](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L139) | timer/branch | `if (P.QuadroAcao > 2) and (P.QuadroAcao < 8) then` |
| [141](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L141) | simulation mutation | `if (P.ForcaGolpe = fgFraco) and (P.Lado = ldDireita) then Empurra(Jog, 5);` |
| [142](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L142) | simulation mutation | `if (P.ForcaGolpe = fgFraco) and (P.Lado = ldEsquerda) then Empurra(Jog, -5);` |
| [143](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L143) | simulation mutation | `if (P.ForcaGolpe = fgForte) and (P.Lado = ldDireita) then Empurra(Jog, 12);` |
| [144](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L144) | simulation mutation | `if (P.ForcaGolpe = fgForte) and (P.Lado = ldEsquerda) then Empurra(Jog, -12);` |
| [146](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L146) | simulation mutation | `P.EhChute := True;` |
| [150](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L150) | timer/branch, effect | `if P.QuadroAcao = 0 then CriaEfeito(17, '', Jog, P.Y - 90, P.X);` |
| [151](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L151) | timer/branch | `if P.QuadroAcao = 0 then` |
| [153](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L153) | simulation mutation | `P.VelY := 0;` |
| [154](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L154) | simulation mutation | `P.NoAr := False;` |
| [156](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L156) | timer/branch, simulation mutation | `if P.QuadroAcao = 1 then P.Invulneravel := True;` |
| [157](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L157) | timer/branch, simulation mutation | `if P.QuadroAcao = 9 then P.Invulneravel := False;` |
| [158](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L158) | timer/branch | `if P.QuadroAcao = 8 then` |
| [160](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L160) | simulation mutation | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgFraco) then P.X := P.X + 80;` |
| [161](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L161) | simulation mutation | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgForte) then P.X := P.X + 150;` |
| [162](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L162) | simulation mutation | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgFraco) then P.X := P.X - 80;` |
| [163](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L163) | simulation mutation | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgForte) then P.X := P.X - 150;` |
| [164](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L164) | simulation mutation | `P.Y := 50;` |
| [165](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L165) | simulation mutation | `P.VelY := 0;` |
| [166](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L166) | simulation mutation | `P.DirPulo := puloNenhum;` |
| [167](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L167) | simulation mutation | `P.NoAr := True;` |
| [168](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L168) | effect | `CriaEfeito(24, '', Jog, P.X - 37, P.Y - 80);` |
| [170](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L170) | timer/branch, frame | `if P.QuadroAcao = 9 then P.Quadro := Desloc + 42;` |
| [171](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L171) | timer/branch, frame | `if P.QuadroAcao = 11 then P.Quadro := Desloc + 43;` |
| [172](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L172) | timer/branch, frame | `if P.QuadroAcao = 13 then P.Quadro := Desloc + 42;` |
| [173](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L173) | timer/branch, frame | `if P.QuadroAcao = 15 then P.Quadro := Desloc + 43;` |
| [174](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L174) | timer/branch, frame | `if P.QuadroAcao = 17 then P.Quadro := Desloc + 42;` |
| [175](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L175) | timer/branch | `if P.QuadroAcao >= 8 then` |
| [177](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L177) | simulation mutation | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgFraco) then Empurra(Jog, 5);` |
| [178](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L178) | simulation mutation | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgForte) then Empurra(Jog, 8);` |
| [179](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L179) | simulation mutation | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgFraco) then Empurra(Jog, -5);` |
| [180](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L180) | simulation mutation | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgForte) then Empurra(Jog, -8);` |
| [182](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L182) | simulation mutation | `P.EhChute := True;` |
| [186](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L186) | timer/branch, sound | `if P.QuadroAcao = 0 then SomPendente := 42;` |
| [187](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L187) | timer/branch, frame | `if P.QuadroAcao = 0 then P.Quadro := Desloc + 33;` |
| [188](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L188) | timer/branch | `if (P.QuadroAcao mod 2 = 0) and (P.QuadroAcao < 15) then` |
| [190](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L190) | effect | `if P.Lado = ldDireita then CriaEfeito(20, 'A', Jog, P.X + 20, P.Y - 90);` |
| [191](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L191) | effect | `if P.Lado = ldEsquerda then CriaEfeito(20, 'A', Jog, P.X - 20, P.Y - 90);` |
| [193](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C08.rot#L193) | simulation mutation | `O.Invulneravel := False;` |

## 🌈 9. Paty

- Source: [`C09.rot` lines 83–203](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L83)
- SHA/blob: `aaa2677dbc75f82a97631752aca9f62b54b99472`
- `[Quadro]` source lines: 121
- Indexed event statements: 81
- Category counts: `timer/branch` 42; `frame` 31; `simulation mutation` 38; `sound` 1

| Source line | Category | Ordered source statement |
|---:|---|---|
| [84](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L84) | timer/branch | `if (P.Acao = acVitoria) and (P.QuadroAcao mod 3 = 0) then` |
| [86](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L86) | frame | `if P.Quadro = Desloc + 33 then P.Quadro := Desloc + 34` |
| [87](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L87) | frame | `else P.Quadro := Desloc + 33;` |
| [89](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L89) | timer/branch, frame | `if (P.Acao = acPose) and (P.QuadroAcao = 0) then P.Quadro := Desloc + 12;` |
| [90](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L90) | timer/branch, frame | `if (P.Acao = acPose) and (P.QuadroAcao = 2) then P.Quadro := Desloc + 30;` |
| [93](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L93) | timer/branch, frame | `if P.QuadroAcao mod 2 = 0 then P.Quadro := Desloc + 35` |
| [94](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L94) | frame | `else P.Quadro := Desloc + 36;` |
| [95](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L95) | timer/branch, frame | `if P.QuadroAcao < 2 then P.Quadro := Desloc + 19;` |
| [96](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L96) | timer/branch, frame | `if (P.QuadroAcao >= 10) and (P.ForcaGolpe = fgFraco) then P.Quadro := Desloc + 12;` |
| [97](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L97) | timer/branch, frame | `if (P.QuadroAcao >= 18) and (P.ForcaGolpe = fgForte) then P.Quadro := Desloc + 12;` |
| [98](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L98) | timer/branch | `if (P.QuadroAcao < 10) and (P.ForcaGolpe = fgFraco) then` |
| [100](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L100) | simulation mutation | `if P.Lado = ldDireita then Empurra(Jog, 14);` |
| [101](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L101) | simulation mutation | `if P.Lado = ldEsquerda then Empurra(Jog, -14);` |
| [103](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L103) | timer/branch | `if (P.QuadroAcao < 18) and (P.ForcaGolpe = fgForte) then` |
| [105](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L105) | simulation mutation | `if P.Lado = ldDireita then Empurra(Jog, 13);` |
| [106](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L106) | simulation mutation | `if P.Lado = ldEsquerda then Empurra(Jog, -13);` |
| [108](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L108) | simulation mutation | `P.EhChute := True;` |
| [112](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L112) | simulation mutation | `P.DirPulo := puloNenhum;` |
| [113](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L113) | timer/branch, frame | `if P.QuadroAcao = 0 then P.Quadro := Desloc + 37;` |
| [114](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L114) | timer/branch, frame | `if P.QuadroAcao = 4 then P.Quadro := Desloc + 38;` |
| [115](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L115) | timer/branch, frame | `if P.QuadroAcao = 8 then P.Quadro := Desloc + 39;` |
| [116](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L116) | timer/branch | `if P.QuadroAcao = 8 then` |
| [118](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L118) | simulation mutation | `P.VelY := -10;` |
| [119](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L119) | simulation mutation | `P.NoAr := True;` |
| [121](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L121) | timer/branch | `if P.QuadroAcao < 8 then` |
| [123](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L123) | simulation mutation | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgFraco) then Empurra(Jog, 6);` |
| [124](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L124) | simulation mutation | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgFraco) then Empurra(Jog, -6);` |
| [125](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L125) | simulation mutation | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgForte) then Empurra(Jog, 12);` |
| [126](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L126) | simulation mutation | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgForte) then Empurra(Jog, -12);` |
| [127](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L127) | simulation mutation | `if P.ForcaGolpe = fgFraco then P.Y := P.Y - 9;` |
| [128](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L128) | simulation mutation | `if P.ForcaGolpe = fgForte then P.Y := P.Y - 15;` |
| [130](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L130) | simulation mutation | `P.EhChute := True;` |
| [134](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L134) | timer/branch, frame | `if P.QuadroAcao = 0 then P.Quadro := Desloc + 40;` |
| [135](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L135) | timer/branch, frame | `if P.QuadroAcao = 3 then P.Quadro := Desloc + 41;` |
| [136](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L136) | timer/branch, frame | `if P.QuadroAcao = 5 then P.Quadro := Desloc + 42;` |
| [137](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L137) | timer/branch, frame | `if P.QuadroAcao = 7 then P.Quadro := Desloc + 43;` |
| [138](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L138) | timer/branch | `if P.QuadroAcao < 3 then` |
| [140](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L140) | simulation mutation | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgFraco) then Empurra(Jog, 20);` |
| [141](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L141) | simulation mutation | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgForte) then Empurra(Jog, 25);` |
| [142](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L142) | simulation mutation | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgFraco) then Empurra(Jog, -20);` |
| [143](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L143) | simulation mutation | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgForte) then Empurra(Jog, -25);` |
| [145](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L145) | timer/branch, simulation mutation | `if (P.QuadroAcao >= 6) and (P.ForcaGolpe = fgForte) then P.EhChute := True;` |
| [149](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L149) | timer/branch, frame | `if P.QuadroAcao = 0 then P.Quadro := Desloc + 24;` |
| [150](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L150) | timer/branch, simulation mutation | `if P.QuadroAcao = 0 then P.VelY := 0;` |
| [151](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L151) | timer/branch, frame | `if P.QuadroAcao = 3 then P.Quadro := Desloc + 44;` |
| [152](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L152) | timer/branch | `if P.QuadroAcao >= 3 then` |
| [154](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L154) | simulation mutation | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgFraco) then Empurra(Jog, 4);` |
| [155](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L155) | simulation mutation | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgFraco) then Empurra(Jog, -4);` |
| [156](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L156) | simulation mutation | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgForte) then Empurra(Jog, 12);` |
| [157](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L157) | simulation mutation | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgForte) then Empurra(Jog, -12);` |
| [158](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L158) | simulation mutation | `P.Y := P.Y + 15;` |
| [162](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L162) | timer/branch | `P.QuadroAcao := 0;` |
| [163](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L163) | simulation mutation | `P.GolpeAtivo := False;` |
| [164](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L164) | simulation mutation | `P.Y := ChaoY;` |
| [167](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L167) | simulation mutation | `P.EhChute := True;` |
| [171](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L171) | timer/branch, frame | `if P.QuadroAcao = 0 then P.Quadro := Desloc + 35;` |
| [172](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L172) | timer/branch, sound | `if P.QuadroAcao = 0 then SomPendente := 46;` |
| [173](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L173) | timer/branch, frame | `if P.QuadroAcao = 1 then P.Quadro := Desloc + 36;` |
| [174](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L174) | timer/branch, frame | `if P.QuadroAcao = 2 then P.Quadro := Desloc + 35;` |
| [175](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L175) | timer/branch, frame | `if P.QuadroAcao = 3 then P.Quadro := Desloc + 36;` |
| [176](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L176) | timer/branch, frame | `if P.QuadroAcao = 4 then P.Quadro := Desloc + 35;` |
| [177](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L177) | timer/branch, frame | `if P.QuadroAcao = 5 then P.Quadro := Desloc + 36;` |
| [178](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L178) | timer/branch, frame | `if P.QuadroAcao = 6 then P.Quadro := Desloc + 35;` |
| [179](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L179) | timer/branch, frame | `if P.QuadroAcao = 7 then P.Quadro := Desloc + 36;` |
| [180](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L180) | timer/branch, frame | `if P.QuadroAcao = 8 then P.Quadro := Desloc + 35;` |
| [181](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L181) | timer/branch, frame | `if P.QuadroAcao = 9 then P.Quadro := Desloc + 36;` |
| [182](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L182) | timer/branch | `if (P.QuadroAcao >= 0) and (P.QuadroAcao <= 10) then` |
| [184](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L184) | simulation mutation | `if P.Lado = ldDireita then Empurra(Jog, 12);` |
| [185](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L185) | simulation mutation | `if P.Lado = ldEsquerda then Empurra(Jog, -12);` |
| [187](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L187) | timer/branch, frame | `if P.QuadroAcao = 11 then P.Quadro := Desloc + 37;` |
| [188](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L188) | timer/branch, frame | `if P.QuadroAcao = 12 then P.Quadro := Desloc + 38;` |
| [189](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L189) | timer/branch | `if (P.QuadroAcao >= 12) and (P.QuadroAcao <= 18) then` |
| [191](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L191) | simulation mutation | `if P.Lado = ldDireita then Empurra(Jog, 15);` |
| [192](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L192) | simulation mutation | `if P.Lado = ldEsquerda then Empurra(Jog, -15);` |
| [193](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L193) | simulation mutation | `P.Y := P.Y - 18;` |
| [194](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L194) | simulation mutation | `P.EhChute := True;` |
| [195](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L195) | simulation mutation | `O.Invulneravel := False;` |
| [197](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L197) | timer/branch, frame | `if P.QuadroAcao = 22 then P.Quadro := Desloc + 39;` |
| [198](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L198) | timer/branch | `if P.QuadroAcao = 22 then` |
| [200](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L200) | simulation mutation | `P.VelY := -14;` |
| [201](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C09.rot#L201) | simulation mutation | `P.NoAr := True;` |

## 🌈 10. Nhonho

- Source: [`C10.rot` lines 86–183](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L86)
- SHA/blob: `e4b229975d0e7bb1f4044f2030a8aa38211218f6`
- `[Quadro]` source lines: 98
- Indexed event statements: 61
- Category counts: `timer/branch` 29; `frame` 19; `RNG` 4; `simulation mutation` 29; `sound` 1; `effect` 2

| Source line | Category | Ordered source statement |
|---:|---|---|
| [90](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L90) | timer/branch, frame | `if (P.Acao = acVitoria) and (P.QuadroAcao < 12) then P.Quadro := Desloc + 33;` |
| [91](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L91) | timer/branch, frame | `if (P.Acao = acVitoria) and (P.QuadroAcao = 12) then P.Quadro := Desloc + 34;` |
| [92](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L92) | timer/branch, frame | `if (P.Acao = acPose) and (P.QuadroAcao = 0) then P.Quadro := Desloc + 12;` |
| [93](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L93) | timer/branch, frame | `if (P.Acao = acPose) and (P.QuadroAcao = 2) then P.Quadro := Desloc + 30;` |
| [97](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L97) | RNG | `S_0 := Random(3) + (Desloc + 35);` |
| [99](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L99) | frame | `P.Quadro := S_0;` |
| [100](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L100) | simulation mutation | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgFraco) then Empurra(Jog, 2);` |
| [101](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L101) | simulation mutation | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgFraco) then Empurra(Jog, -2);` |
| [102](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L102) | simulation mutation | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgForte) then Empurra(Jog, 3);` |
| [103](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L103) | simulation mutation | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgForte) then Empurra(Jog, -3);` |
| [104](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L104) | sound, RNG | `if Random(5) = 0 then SomPendente := 47;` |
| [108](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L108) | timer/branch, frame | `if P.QuadroAcao = 0 then P.Quadro := Desloc + 38;` |
| [109](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L109) | timer/branch, frame | `if P.QuadroAcao = 2 then P.Quadro := Desloc + 39;` |
| [110](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L110) | timer/branch, frame | `if P.QuadroAcao = 4 then P.Quadro := Desloc + 40;` |
| [111](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L111) | timer/branch, frame | `if (P.QuadroAcao = 10) and (P.ForcaGolpe = fgFraco) then P.Quadro := Desloc + 12;` |
| [112](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L112) | timer/branch, frame | `if (P.QuadroAcao = 16) and (P.ForcaGolpe = fgForte) then P.Quadro := Desloc + 12;` |
| [113](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L113) | timer/branch | `if (P.QuadroAcao >= 4) and (P.ForcaGolpe = fgFraco) then` |
| [115](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L115) | simulation mutation | `if P.Lado = ldDireita then Empurra(Jog, 19);` |
| [116](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L116) | simulation mutation | `if P.Lado = ldEsquerda then Empurra(Jog, -19);` |
| [118](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L118) | timer/branch | `if (P.QuadroAcao >= 4) and (P.QuadroAcao < 16) and (P.ForcaGolpe = fgForte) then` |
| [120](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L120) | simulation mutation | `if P.Lado = ldDireita then Empurra(Jog, 17);` |
| [121](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L121) | simulation mutation | `if P.Lado = ldEsquerda then Empurra(Jog, -17);` |
| [123](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L123) | timer/branch | `if P.QuadroAcao = 2 then` |
| [124](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L124) | effect, RNG | `for Cont := 1 to 3 do CriaEfeito(2, '', 0, Random(40) + (P.X - 20), Random(10) + P.Y);` |
| [125](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L125) | simulation mutation | `P.EhChute := True;` |
| [129](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L129) | simulation mutation | `P.DirPulo := puloNenhum;` |
| [130](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L130) | timer/branch, frame | `if P.QuadroAcao = 0 then P.Quadro := Desloc + 41;` |
| [131](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L131) | timer/branch, frame | `if P.QuadroAcao = 4 then P.Quadro := Desloc + 42;` |
| [132](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L132) | timer/branch | `if P.QuadroAcao <= 4 then` |
| [134](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L134) | simulation mutation | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgFraco) then Empurra(Jog, 16);` |
| [135](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L135) | simulation mutation | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgFraco) then Empurra(Jog, -16);` |
| [136](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L136) | simulation mutation | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgForte) then Empurra(Jog, 20);` |
| [137](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L137) | simulation mutation | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgForte) then Empurra(Jog, -20);` |
| [139](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L139) | timer/branch, simulation mutation | `if P.QuadroAcao >= 5 then P.EhChute := True;` |
| [143](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L143) | timer/branch, frame | `if P.QuadroAcao = 0 then P.Quadro := Desloc + 43;` |
| [144](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L144) | timer/branch, frame | `if P.QuadroAcao = 6 then P.Quadro := Desloc + 44;` |
| [145](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L145) | timer/branch | `if P.QuadroAcao >= 6 then` |
| [147](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L147) | simulation mutation | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgFraco) then Empurra(Jog, 16);` |
| [148](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L148) | simulation mutation | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgFraco) then Empurra(Jog, -16);` |
| [149](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L149) | simulation mutation | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgForte) then Empurra(Jog, 20);` |
| [150](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L150) | simulation mutation | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgForte) then Empurra(Jog, -20);` |
| [151](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L151) | effect, RNG | `for Cont := 1 to 3 do CriaEfeito(2, '', 0, Random(40) + (P.X - 20), Random(10) + P.Y);` |
| [153](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L153) | simulation mutation | `P.EhChute := True;` |
| [157](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L157) | timer/branch, frame | `if P.QuadroAcao = 0 then P.Quadro := Desloc + 38;` |
| [158](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L158) | timer/branch, frame | `if P.QuadroAcao = 2 then P.Quadro := Desloc + 39;` |
| [159](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L159) | timer/branch, frame | `if P.QuadroAcao = 4 then P.Quadro := Desloc + 40;` |
| [160](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L160) | timer/branch, frame | `if P.QuadroAcao = 15 then P.Quadro := Desloc + 41;` |
| [161](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L161) | timer/branch | `if P.QuadroAcao = 15 then` |
| [163](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L163) | simulation mutation | `P.VelY := -35;` |
| [164](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L164) | simulation mutation | `P.NoAr := True;` |
| [166](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L166) | timer/branch | `if P.QuadroAcao = 20 then` |
| [168](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L168) | frame | `P.Quadro := Desloc + 42;` |
| [169](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L169) | simulation mutation | `P.VelY := 10;` |
| [171](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L171) | timer/branch | `if (P.QuadroAcao >= 3) and (P.QuadroAcao < 15) then` |
| [173](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L173) | simulation mutation | `if P.Lado = ldDireita then Empurra(Jog, 13);` |
| [174](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L174) | simulation mutation | `if P.Lado = ldEsquerda then Empurra(Jog, -13);` |
| [176](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L176) | timer/branch | `if (P.QuadroAcao >= 15) and (P.QuadroAcao < 20) then` |
| [178](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L178) | simulation mutation | `if P.Lado = ldDireita then Empurra(Jog, 23);` |
| [179](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L179) | simulation mutation | `if P.Lado = ldEsquerda then Empurra(Jog, -23);` |
| [181](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L181) | timer/branch, simulation mutation | `if P.QuadroAcao = 14 then P.EhChute := True;` |
| [182](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C10.rot#L182) | timer/branch, simulation mutation | `if P.QuadroAcao > 14 then O.Invulneravel := False;` |

## 🌈 11. Godines

- Source: [`C11.rot` lines 77–180](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L77)
- SHA/blob: `862192ac6dbbafbe4d71e3100d99a515f408c894`
- `[Quadro]` source lines: 104
- Indexed event statements: 73
- Category counts: `frame` 34; `timer/branch` 42; `effect` 9; `sound` 3; `simulation mutation` 22; `RNG` 1

| Source line | Category | Ordered source statement |
|---:|---|---|
| [80](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L80) | frame | `if P.Acao = acVitoria then P.Quadro := Desloc + 33;` |
| [81](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L81) | timer/branch, frame | `if (P.Acao = acPose) and (P.QuadroAcao = 0) then P.Quadro := Desloc + 12;` |
| [82](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L82) | timer/branch, frame | `if (P.Acao = acPose) and (P.QuadroAcao = 2) then P.Quadro := Desloc + 30;` |
| [85](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L85) | timer/branch, frame | `if P.QuadroAcao = 0 then P.Quadro := Desloc + 34;` |
| [86](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L86) | timer/branch, frame | `if P.QuadroAcao = 5 then P.Quadro := Desloc + 35;` |
| [87](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L87) | timer/branch | `if P.QuadroAcao = 10 then` |
| [89](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L89) | frame | `P.Quadro := Desloc + 36;` |
| [90](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L90) | effect | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgFraco) then CriaEfeito(22, 'A', Jog, P.X + 30, P.Y - 100);` |
| [91](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L91) | effect | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgFraco) then CriaEfeito(22, 'A', Jog, P.X - 30, P.Y - 100);` |
| [92](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L92) | effect | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgForte) then CriaEfeito(22, 'B', Jog, P.X + 30, P.Y - 100);` |
| [93](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L93) | effect | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgForte) then CriaEfeito(22, 'B', Jog, P.X - 30, P.Y - 100);` |
| [94](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L94) | sound | `SomPendente := 50;` |
| [99](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L99) | timer/branch, frame | `if P.QuadroAcao = 0 then P.Quadro := Desloc + 37;` |
| [100](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L100) | timer/branch | `if P.QuadroAcao = 4 then` |
| [102](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L102) | frame | `P.Quadro := Desloc + 38;` |
| [103](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L103) | effect | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgFraco) then CriaEfeito(23, 'A', Jog, P.X + 50, P.Y);` |
| [104](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L104) | effect | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgFraco) then CriaEfeito(23, 'A', Jog, P.X - 50, P.Y);` |
| [105](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L105) | effect | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgForte) then CriaEfeito(23, 'B', Jog, P.X + 50, P.Y);` |
| [106](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L106) | effect | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgForte) then CriaEfeito(23, 'B', Jog, P.X - 50, P.Y);` |
| [107](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L107) | sound | `SomPendente := 51;` |
| [112](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L112) | timer/branch, frame | `if (P.ForcaGolpe = fgFraco) and (P.QuadroAcao < 1) then P.Quadro := Desloc + 15;` |
| [113](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L113) | timer/branch, frame | `if (P.ForcaGolpe = fgForte) and (P.QuadroAcao < 1) then P.Quadro := Desloc + 6;` |
| [114](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L114) | timer/branch, frame | `if (P.ForcaGolpe = fgFraco) and (P.QuadroAcao >= 1) then P.Quadro := Desloc + 39;` |
| [115](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L115) | timer/branch, frame | `if (P.ForcaGolpe = fgForte) and (P.QuadroAcao >= 1) then P.Quadro := Desloc + 40;` |
| [116](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L116) | timer/branch, sound | `if P.QuadroAcao = 1 then SomPendente := 52;` |
| [117](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L117) | timer/branch | `if P.QuadroAcao >= 1 then` |
| [119](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L119) | simulation mutation | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgFraco) then Empurra(Jog, 19);` |
| [120](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L120) | simulation mutation | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgFraco) then Empurra(Jog, -19);` |
| [121](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L121) | simulation mutation | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgForte) then Empurra(Jog, 16);` |
| [122](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L122) | simulation mutation | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgForte) then Empurra(Jog, -16);` |
| [123](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L123) | effect, RNG | `for Cont := 1 to 8 do CriaEfeito(2, '', 0, Random(40) + (P.X - 20), Random(10) + P.Y);` |
| [125](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L125) | simulation mutation | `if P.ForcaGolpe = fgFraco then P.EhChute := False;` |
| [126](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L126) | simulation mutation | `if P.ForcaGolpe = fgForte then P.EhChute := True;` |
| [130](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L130) | timer/branch, frame | `if P.QuadroAcao = 0 then P.Quadro := Desloc + 41;` |
| [131](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L131) | timer/branch, frame | `if P.QuadroAcao = 2 then P.Quadro := Desloc + 42;` |
| [132](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L132) | timer/branch, frame | `if P.QuadroAcao = 4 then P.Quadro := Desloc + 41;` |
| [133](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L133) | timer/branch, frame | `if P.QuadroAcao = 6 then P.Quadro := Desloc + 42;` |
| [134](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L134) | timer/branch, frame | `if P.QuadroAcao = 8 then P.Quadro := Desloc + 41;` |
| [135](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L135) | timer/branch, frame | `if P.QuadroAcao = 10 then P.Quadro := Desloc + 42;` |
| [136](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L136) | timer/branch, frame | `if P.QuadroAcao = 12 then P.Quadro := Desloc + 41;` |
| [137](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L137) | simulation mutation | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgFraco) then Empurra(Jog, 7);` |
| [138](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L138) | simulation mutation | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgFraco) then Empurra(Jog, -7);` |
| [139](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L139) | simulation mutation | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgForte) then Empurra(Jog, 12);` |
| [140](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L140) | simulation mutation | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgForte) then Empurra(Jog, -12);` |
| [141](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L141) | simulation mutation | `if P.ForcaGolpe = fgFraco then P.Y := P.Y - 6;` |
| [142](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L142) | simulation mutation | `if P.ForcaGolpe = fgForte then P.Y := P.Y - 8;` |
| [143](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L143) | simulation mutation | `P.EhChute := True;` |
| [146](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L146) | timer/branch, simulation mutation | `if P.TempoAcertos > 19 then O.Invulneravel := False;` |
| [147](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L147) | timer/branch, simulation mutation | `if P.TempoAcertos < 17 then O.Invulneravel := True;` |
| [149](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L149) | timer/branch, simulation mutation | `if P.QuadroAcao = 11 then P.VelY := -10;` |
| [153](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L153) | timer/branch, frame | `if P.QuadroAcao = 0 then P.Quadro := Desloc + 7;` |
| [154](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L154) | timer/branch, frame | `if P.QuadroAcao = 2 then P.Quadro := Desloc + 9;` |
| [155](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L155) | timer/branch, frame | `if P.QuadroAcao = 4 then P.Quadro := Desloc + 7;` |
| [156](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L156) | timer/branch, frame | `if P.QuadroAcao = 6 then P.Quadro := Desloc + 5;` |
| [157](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L157) | timer/branch, frame | `if P.QuadroAcao = 8 then P.Quadro := Desloc + 9;` |
| [158](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L158) | timer/branch, frame | `if P.QuadroAcao = 10 then P.Quadro := Desloc + 5;` |
| [159](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L159) | timer/branch, frame | `if P.QuadroAcao = 12 then P.Quadro := Desloc + 9;` |
| [160](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L160) | timer/branch, frame | `if P.QuadroAcao = 14 then P.Quadro := Desloc + 5;` |
| [161](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L161) | timer/branch, frame | `if P.QuadroAcao = 16 then P.Quadro := Desloc + 9;` |
| [162](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L162) | timer/branch, frame | `if P.QuadroAcao = 18 then P.Quadro := Desloc + 7;` |
| [163](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L163) | timer/branch, frame | `if P.QuadroAcao = 20 then P.Quadro := Desloc + 9;` |
| [164](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L164) | timer/branch, frame | `if P.QuadroAcao = 22 then P.Quadro := Desloc + 7;` |
| [165](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L165) | timer/branch, frame | `if P.QuadroAcao = 24 then P.Quadro := Desloc + 5;` |
| [166](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L166) | timer/branch, frame | `if P.QuadroAcao = 26 then P.Quadro := Desloc + 9;` |
| [167](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L167) | timer/branch, frame | `if P.QuadroAcao = 28 then P.Quadro := Desloc + 11;` |
| [168](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L168) | timer/branch | `if P.QuadroAcao < 27 then` |
| [170](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L170) | simulation mutation | `if P.Lado = ldDireita then Empurra(Jog, 7);` |
| [171](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L171) | simulation mutation | `if P.Lado = ldEsquerda then Empurra(Jog, -7);` |
| [173](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L173) | timer/branch | `if P.QuadroAcao >= 27 then` |
| [175](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L175) | simulation mutation | `if P.Lado = ldDireita then Empurra(Jog, 12);` |
| [176](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L176) | simulation mutation | `if P.Lado = ldEsquerda then Empurra(Jog, -12);` |
| [178](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L178) | timer/branch, simulation mutation | `if (P.Acertos >= 3) and (P.TempoAcertos >= 18) then O.Invulneravel := False;` |
| [179](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C11.rot#L179) | timer/branch, simulation mutation | `if P.QuadroAcao >= 28 then P.EhChute := True;` |

## 🌈 12. Sr. Furtado

- Source: [`C12.rot` lines 72–192](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L72)
- SHA/blob: `d34fdcd41d050ae2d754d4552d58f429a218fa8b`
- `[Quadro]` source lines: 121
- Indexed event statements: 74
- Category counts: `timer/branch` 35; `frame` 25; `simulation mutation` 31; `RNG` 2; `effect` 5; `sound` 5

| Source line | Category | Ordered source statement |
|---:|---|---|
| [77](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L77) | timer/branch, frame | `if P.QuadroAcao mod 2 = 0 then P.Quadro := Desloc + 33` |
| [78](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L78) | frame | `else P.Quadro := Desloc + 34;` |
| [79](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L79) | simulation mutation | `if P.Lado = ldDireita then P.X := P.X + 12;` |
| [80](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L80) | simulation mutation | `if P.Lado = ldEsquerda then P.X := P.X - 12;` |
| [83](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L83) | timer/branch, RNG | `if (P.QuadroAcao mod 8 = 0) and (Random(2) = 0) then` |
| [88](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L88) | effect, RNG | `for Cont := 1 to 5 do CriaEfeito(2, '', 0, Random(40) + (P.X - 30), Random(10) + P.Y);` |
| [90](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L90) | timer/branch, frame | `if (P.Acao = acPose) and (P.QuadroAcao = 0) then P.Quadro := Desloc + 12;` |
| [91](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L91) | timer/branch, frame | `if (P.Acao = acPose) and (P.QuadroAcao = 2) then P.Quadro := Desloc + 30;` |
| [94](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L94) | timer/branch, frame | `if P.QuadroAcao = 0 then P.Quadro := Desloc + 35;` |
| [95](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L95) | timer/branch, frame | `if P.QuadroAcao = 5 then P.Quadro := Desloc + 36;` |
| [96](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L96) | timer/branch | `if P.QuadroAcao = 10 then` |
| [98](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L98) | frame | `P.Quadro := Desloc + 37;` |
| [99](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L99) | effect | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgFraco) then CriaEfeito(9, 'A', Jog, P.X + 35, P.Y - 105);` |
| [100](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L100) | effect | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgFraco) then CriaEfeito(9, 'A', Jog, P.X - 35, P.Y - 105);` |
| [101](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L101) | effect | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgForte) then CriaEfeito(9, 'B', Jog, P.X + 35, P.Y - 105);` |
| [102](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L102) | effect | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgForte) then CriaEfeito(9, 'B', Jog, P.X - 35, P.Y - 105);` |
| [103](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L103) | sound | `SomPendente := 53;` |
| [108](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L108) | timer/branch | `if P.QuadroAcao = 0 then` |
| [110](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L110) | frame | `P.Quadro := Desloc + 38;` |
| [111](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L111) | sound | `SomPendente := 54;` |
| [113](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L113) | timer/branch, frame | `if P.QuadroAcao = 3 then P.Quadro := Desloc + 39;` |
| [114](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L114) | timer/branch | `if ((P.QuadroAcao = 8) and (P.ForcaGolpe = fgFraco)) or ((P.QuadroAcao = 10) and (P.ForcaGolpe = fgForte)) then` |
| [116](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L116) | frame | `P.Quadro := Desloc + 40;` |
| [117](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L117) | simulation mutation | `P.NoAr := True;` |
| [118](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L118) | simulation mutation | `P.VelY := -6;` |
| [124](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L124) | simulation mutation | `P.Y := P.Y - 15;` |
| [125](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L125) | simulation mutation | `if P.Lado = ldDireita then Empurra(Jog, 4);` |
| [126](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L126) | simulation mutation | `if P.Lado = ldEsquerda then Empurra(Jog, -4);` |
| [130](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L130) | simulation mutation | `P.Y := P.Y - 17;` |
| [131](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L131) | simulation mutation | `if P.Lado = ldDireita then Empurra(Jog, 8);` |
| [132](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L132) | simulation mutation | `if P.Lado = ldEsquerda then Empurra(Jog, -8);` |
| [135](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L135) | simulation mutation | `if P.Quadro = Desloc + 40 then P.NoAr := True;` |
| [136](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L136) | simulation mutation | `P.EhChute := True;` |
| [140](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L140) | timer/branch, frame | `if P.QuadroAcao = 0 then P.Quadro := Desloc + 41;` |
| [141](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L141) | timer/branch, frame | `if P.QuadroAcao = 2 then P.Quadro := Desloc + 42;` |
| [142](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L142) | timer/branch, frame | `if P.QuadroAcao = 4 then P.Quadro := Desloc + 41;` |
| [143](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L143) | timer/branch, frame | `if P.QuadroAcao = 6 then P.Quadro := Desloc + 43;` |
| [144](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L144) | timer/branch, frame | `if P.QuadroAcao = 8 then P.Quadro := Desloc + 44;` |
| [145](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L145) | timer/branch, frame | `if P.QuadroAcao = 10 then P.Quadro := Desloc + 43;` |
| [146](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L146) | timer/branch, frame | `if P.QuadroAcao = 12 then P.Quadro := Desloc + 41;` |
| [147](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L147) | timer/branch, frame | `if P.QuadroAcao = 14 then P.Quadro := Desloc + 42;` |
| [148](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L148) | timer/branch, frame | `if P.QuadroAcao = 16 then P.Quadro := Desloc + 41;` |
| [149](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L149) | simulation mutation | `P.Y := P.Y - 11;` |
| [150](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L150) | timer/branch, simulation mutation | `P.Y := P.QuadroAcao + P.Y;` |
| [151](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L151) | simulation mutation | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgFraco) then Empurra(Jog, 10);` |
| [152](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L152) | simulation mutation | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgFraco) then Empurra(Jog, -10);` |
| [153](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L153) | simulation mutation | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgForte) then Empurra(Jog, 15);` |
| [154](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L154) | simulation mutation | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgForte) then Empurra(Jog, -15);` |
| [155](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L155) | timer/branch, sound | `if P.QuadroAcao = 2 then SomPendente := 55;` |
| [156](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L156) | timer/branch, sound | `if P.QuadroAcao = 8 then SomPendente := 55;` |
| [157](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L157) | timer/branch, sound | `if P.QuadroAcao = 14 then SomPendente := 56;` |
| [158](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L158) | timer/branch, simulation mutation | `if (P.QuadroAcao >= 6) and (P.QuadroAcao <= 16) then O.Invulneravel := False;` |
| [159](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L159) | timer/branch, simulation mutation | `if P.QuadroAcao >= 10 then P.EhChute := True;` |
| [163](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L163) | timer/branch, frame | `if P.QuadroAcao = 0 then P.Quadro := Desloc + 38;` |
| [164](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L164) | timer/branch, frame | `if P.QuadroAcao = 4 then P.Quadro := Desloc + 39;` |
| [165](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L165) | timer/branch, frame | `if P.QuadroAcao = 8 then P.Quadro := Desloc + 40;` |
| [166](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L166) | timer/branch, frame | `if P.QuadroAcao = 10 then P.Quadro := Desloc + 38;` |
| [167](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L167) | timer/branch, frame | `if P.QuadroAcao = 13 then P.Quadro := Desloc + 39;` |
| [168](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L168) | timer/branch, frame | `if P.QuadroAcao = 25 then P.Quadro := Desloc + 40;` |
| [169](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L169) | timer/branch | `if P.QuadroAcao <= 8 then` |
| [171](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L171) | simulation mutation | `if P.Lado = ldDireita then Empurra(Jog, 15);` |
| [172](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L172) | simulation mutation | `if P.Lado = ldEsquerda then Empurra(Jog, -15);` |
| [174](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L174) | timer/branch | `if (P.QuadroAcao >= 13) and (P.QuadroAcao < 25) then` |
| [176](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L176) | simulation mutation | `P.Y := P.Y - 12;` |
| [177](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L177) | simulation mutation | `if P.Lado = ldDireita then Empurra(Jog, 4);` |
| [178](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L178) | simulation mutation | `if P.Lado = ldEsquerda then Empurra(Jog, -4);` |
| [180](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L180) | timer/branch | `if P.QuadroAcao = 25 then` |
| [182](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L182) | simulation mutation | `P.NoAr := True;` |
| [183](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L183) | simulation mutation | `P.DirPulo := puloNenhum;` |
| [184](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L184) | simulation mutation | `P.VelY := -8;` |
| [186](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L186) | simulation mutation | `O.Recuo := 0;` |
| [187](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L187) | timer/branch | `if P.QuadroAcao >= 13 then` |
| [189](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L189) | simulation mutation | `P.EhChute := True;` |
| [190](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C12.rot#L190) | simulation mutation | `O.Invulneravel := False;` |

## 🌈 13. Jaiminho

- Source: [`C13.rot` lines 77–152](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C13.rot#L77)
- SHA/blob: `08a0664d1098b59ecfcfb8ca5a3f6cae2126e3a5`
- `[Quadro]` source lines: 76
- Indexed event statements: 51
- Category counts: `frame` 14; `timer/branch` 18; `simulation mutation` 17; `effect` 13; `sound` 4

| Source line | Category | Ordered source statement |
|---:|---|---|
| [80](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C13.rot#L80) | frame | `P.Quadro := Desloc + 33;` |
| [81](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C13.rot#L81) | timer/branch, simulation mutation | `P.Y := P.Y - Round(Cos(P.QuadroAcao / 3) * 6);` |
| [82](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C13.rot#L82) | simulation mutation | `if ChaoY - 10 < P.Y then P.Y := ChaoY - 10;` |
| [86](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C13.rot#L86) | timer/branch, frame | `if P.QuadroAcao = 0 then P.Quadro := Desloc + 34;` |
| [87](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C13.rot#L87) | timer/branch, frame | `if P.QuadroAcao = 5 then P.Quadro := Desloc + 35;` |
| [88](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C13.rot#L88) | timer/branch | `if P.QuadroAcao = 10 then` |
| [90](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C13.rot#L90) | frame | `P.Quadro := Desloc + 36;` |
| [91](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C13.rot#L91) | effect | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgFraco) then CriaEfeito(26, 'A', Jog, P.X + 30, P.Y - 100);` |
| [92](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C13.rot#L92) | effect | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgFraco) then CriaEfeito(26, 'A', Jog, P.X - 30, P.Y - 100);` |
| [93](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C13.rot#L93) | effect | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgForte) then CriaEfeito(26, 'B', Jog, P.X + 30, P.Y - 100);` |
| [94](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C13.rot#L94) | effect | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgForte) then CriaEfeito(26, 'B', Jog, P.X - 30, P.Y - 100);` |
| [96](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C13.rot#L96) | timer/branch, sound | `if P.QuadroAcao = 3 then SomPendente := 57;` |
| [100](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C13.rot#L100) | timer/branch, frame | `if P.QuadroAcao = 0 then P.Quadro := Desloc + 37;` |
| [101](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C13.rot#L101) | timer/branch, frame | `if P.QuadroAcao = 2 then P.Quadro := Desloc + 38;` |
| [102](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C13.rot#L102) | timer/branch, frame | `if P.QuadroAcao = 5 then P.Quadro := Desloc + 39;` |
| [103](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C13.rot#L103) | timer/branch, frame | `if P.QuadroAcao = 8 then P.Quadro := Desloc + 40;` |
| [104](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C13.rot#L104) | timer/branch, frame | `if P.QuadroAcao = 11 then P.Quadro := Desloc + 41;` |
| [105](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C13.rot#L105) | simulation mutation | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgFraco) then Empurra(Jog, 8);` |
| [106](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C13.rot#L106) | simulation mutation | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgFraco) then Empurra(Jog, -8);` |
| [107](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C13.rot#L107) | simulation mutation | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgForte) then Empurra(Jog, 13);` |
| [108](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C13.rot#L108) | simulation mutation | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgForte) then Empurra(Jog, -13);` |
| [112](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C13.rot#L112) | timer/branch, frame | `if P.QuadroAcao mod 2 = 0 then P.Quadro := Desloc + 42` |
| [113](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C13.rot#L113) | frame | `else P.Quadro := Desloc + 43;` |
| [114](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C13.rot#L114) | timer/branch | `if P.QuadroAcao = 3 then` |
| [116](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C13.rot#L116) | effect | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgFraco) then CriaEfeito(27, 'A', Jog, P.X + 45, P.Y - 78);` |
| [117](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C13.rot#L117) | effect | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgFraco) then CriaEfeito(27, 'A', Jog, P.X - 45, P.Y - 78);` |
| [118](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C13.rot#L118) | effect | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgForte) then CriaEfeito(27, 'B', Jog, P.X + 45, P.Y - 78);` |
| [119](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C13.rot#L119) | effect | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgForte) then CriaEfeito(27, 'B', Jog, P.X - 45, P.Y - 78);` |
| [121](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C13.rot#L121) | timer/branch, sound | `if P.QuadroAcao = 3 then SomPendente := 58;` |
| [125](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C13.rot#L125) | timer/branch, frame | `if P.QuadroAcao = 1 then P.Quadro := Desloc + 33;` |
| [126](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C13.rot#L126) | timer/branch | `if P.QuadroAcao = 10 then` |
| [128](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C13.rot#L128) | effect | `CriaEfeito(17, '', Jog, P.Y - 90, P.X);` |
| [129](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C13.rot#L129) | effect | `CriaEfeito(17, '', Jog, CameraX + 80, CameraY + 110);` |
| [130](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C13.rot#L130) | effect | `CriaEfeito(17, '', Jog, CameraX + 160, CameraY + 110);` |
| [131](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C13.rot#L131) | effect | `CriaEfeito(17, '', Jog, CameraX + 240, CameraY + 110);` |
| [132](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C13.rot#L132) | effect | `CriaEfeito(17, '', Jog, CameraX + $140, CameraY + 110);` |
| [133](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C13.rot#L133) | simulation mutation | `if P.ForcaGolpe = fgFraco then P.X := CameraX + 80;` |
| [134](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C13.rot#L134) | simulation mutation | `if P.ForcaGolpe = fgForte then P.X := CameraX + 160;` |
| [135](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C13.rot#L135) | simulation mutation | `if P.ForcaGolpe = fgEspecial then P.X := CameraX + 240;` |
| [136](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C13.rot#L136) | simulation mutation | `if P.ForcaGolpe = 4 then P.X := CameraX + $140;` |
| [137](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C13.rot#L137) | simulation mutation | `P.Y := CameraY + 150;` |
| [138](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C13.rot#L138) | simulation mutation | `P.NoAr := True;` |
| [139](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C13.rot#L139) | simulation mutation | `P.VelY := 0;` |
| [140](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C13.rot#L140) | sound | `SomPendente := 41;` |
| [145](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C13.rot#L145) | timer/branch, frame | `if P.QuadroAcao mod 2 = 0 then P.Quadro := Desloc + 44` |
| [146](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C13.rot#L146) | frame | `else P.Quadro := Desloc + 45;` |
| [147](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C13.rot#L147) | simulation mutation | `if P.Lado = ldDireita then Empurra(Jog, 14);` |
| [148](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C13.rot#L148) | simulation mutation | `if P.Lado = ldEsquerda then Empurra(Jog, -14);` |
| [149](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C13.rot#L149) | simulation mutation | `if P.Acertos > 1 then O.Invulneravel := False;` |
| [150](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C13.rot#L150) | timer/branch, simulation mutation | `if P.QuadroAcao >= 15 then P.EhChute := True;` |
| [151](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C13.rot#L151) | timer/branch, sound | `if P.QuadroAcao = 2 then SomPendente := 59;` |

## 🌈 14. Dona Neves

- Source: [`C14.rot` lines 68–151](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C14.rot#L68)
- SHA/blob: `58157df7c2455e41c201c36217ee51ab9c626728`
- `[Quadro]` source lines: 84
- Indexed event statements: 54
- Category counts: `timer/branch` 24; `frame` 19; `effect` 8; `sound` 4; `simulation mutation` 15

| Source line | Category | Ordered source statement |
|---:|---|---|
| [69](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C14.rot#L69) | timer/branch | `if (P.Acao = acVitoria) and (P.QuadroAcao mod 3 = 0) then` |
| [71](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C14.rot#L71) | frame | `if P.Quadro = Desloc + 33 then P.Quadro := Desloc + 34` |
| [72](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C14.rot#L72) | frame | `else P.Quadro := Desloc + 33;` |
| [76](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C14.rot#L76) | timer/branch, frame | `if P.QuadroAcao = 0 then P.Quadro := Desloc + 35;` |
| [77](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C14.rot#L77) | timer/branch, frame | `if P.QuadroAcao = 7 then P.Quadro := Desloc + 36;` |
| [78](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C14.rot#L78) | timer/branch | `if P.QuadroAcao = 10 then` |
| [80](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C14.rot#L80) | frame | `P.Quadro := Desloc + 37;` |
| [81](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C14.rot#L81) | effect | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgFraco) then CriaEfeito(29, 'A', Jog, P.X + 30, P.Y - 85);` |
| [82](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C14.rot#L82) | effect | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgFraco) then CriaEfeito(29, 'A', Jog, P.X - 30, P.Y - 85);` |
| [83](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C14.rot#L83) | effect | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgForte) then CriaEfeito(29, 'B', Jog, P.X + 30, P.Y - 85);` |
| [84](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C14.rot#L84) | effect | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgForte) then CriaEfeito(29, 'B', Jog, P.X - 30, P.Y - 85);` |
| [86](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C14.rot#L86) | timer/branch, sound | `if P.QuadroAcao = 3 then SomPendente := 60;` |
| [90](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C14.rot#L90) | timer/branch, frame | `if P.QuadroAcao = 0 then P.Quadro := Desloc + 38;` |
| [91](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C14.rot#L91) | timer/branch | `if P.QuadroAcao = 5 then` |
| [93](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C14.rot#L93) | frame | `P.Quadro := Desloc + 39;` |
| [94](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C14.rot#L94) | effect | `if P.Lado = ldDireita then CriaEfeito(30, 'A', Jog, P.X + 42, P.Y - 93);` |
| [95](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C14.rot#L95) | effect | `if P.Lado = ldEsquerda then CriaEfeito(30, 'A', Jog, P.X - 42, P.Y - 93);` |
| [97](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C14.rot#L97) | timer/branch, frame | `if P.QuadroAcao = 32 then P.Quadro := Desloc + 38;` |
| [98](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C14.rot#L98) | timer/branch, sound | `if P.QuadroAcao = 3 then SomPendente := 61;` |
| [102](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C14.rot#L102) | timer/branch, frame | `if P.QuadroAcao = 0 then P.Quadro := Desloc + 40;` |
| [103](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C14.rot#L103) | timer/branch, frame | `if P.QuadroAcao = 1 then P.Quadro := Desloc + 40;` |
| [104](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C14.rot#L104) | timer/branch, frame | `if P.QuadroAcao = 3 then P.Quadro := Desloc + 41;` |
| [105](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C14.rot#L105) | timer/branch, frame | `if P.QuadroAcao = 5 then P.Quadro := Desloc + 42;` |
| [106](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C14.rot#L106) | timer/branch, frame | `if P.QuadroAcao = 7 then P.Quadro := Desloc + 43;` |
| [107](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C14.rot#L107) | timer/branch | `if P.QuadroAcao = 9 then P.QuadroAcao := 1;` |
| [108](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C14.rot#L108) | timer/branch, sound | `if P.QuadroAcao = 0 then SomPendente := 62;` |
| [109](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C14.rot#L109) | simulation mutation | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgFraco) then Empurra(Jog, 6);` |
| [110](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C14.rot#L110) | simulation mutation | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgFraco) then Empurra(Jog, -6);` |
| [111](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C14.rot#L111) | simulation mutation | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgForte) then Empurra(Jog, 9);` |
| [112](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C14.rot#L112) | simulation mutation | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgForte) then Empurra(Jog, -9);` |
| [113](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C14.rot#L113) | simulation mutation | `P.Y := P.Y + 8;` |
| [117](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C14.rot#L117) | timer/branch, frame | `if P.QuadroAcao = 0 then P.Quadro := Desloc + 44;` |
| [118](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C14.rot#L118) | timer/branch, frame | `if P.QuadroAcao = 3 then P.Quadro := Desloc + 45;` |
| [119](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C14.rot#L119) | timer/branch, frame | `if P.QuadroAcao = 5 then P.Quadro := Desloc + 46;` |
| [120](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C14.rot#L120) | timer/branch | `if (P.QuadroAcao >= 4) and (P.QuadroAcao <= 15) then` |
| [122](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C14.rot#L122) | simulation mutation | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgFraco) then Empurra(Jog, 4);` |
| [123](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C14.rot#L123) | simulation mutation | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgFraco) then Empurra(Jog, -4);` |
| [124](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C14.rot#L124) | simulation mutation | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgForte) then Empurra(Jog, 8);` |
| [125](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C14.rot#L125) | simulation mutation | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgForte) then Empurra(Jog, -8);` |
| [126](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C14.rot#L126) | simulation mutation | `if P.ForcaGolpe = fgFraco then P.Y := P.Y - 6;` |
| [127](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C14.rot#L127) | simulation mutation | `if P.ForcaGolpe = fgForte then P.Y := P.Y - 9;` |
| [129](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C14.rot#L129) | timer/branch | `if P.QuadroAcao = 17 then` |
| [131](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C14.rot#L131) | simulation mutation | `P.NoAr := True;` |
| [132](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C14.rot#L132) | simulation mutation | `P.DirPulo := puloNenhum;` |
| [133](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C14.rot#L133) | simulation mutation | `P.VelY := -2;` |
| [134](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C14.rot#L134) | frame | `P.Quadro := Desloc + 21;` |
| [136](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C14.rot#L136) | timer/branch, sound | `if P.QuadroAcao = 2 then SomPendente := 60;` |
| [137](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C14.rot#L137) | simulation mutation | `P.EhChute := True;` |
| [141](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C14.rot#L141) | timer/branch | `if P.QuadroAcao mod 2 = 0 then` |
| [143](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C14.rot#L143) | frame | `if P.Quadro = Desloc + 47 then P.Quadro := Desloc + 48` |
| [144](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C14.rot#L144) | frame | `else P.Quadro := Desloc + 47;` |
| [146](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C14.rot#L146) | timer/branch | `if P.QuadroAcao = 1 then` |
| [148](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C14.rot#L148) | effect | `CriaEfeito(31, 'A', Jog, P.Y - 80, P.X);` |
| [149](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C14.rot#L149) | effect | `CriaEfeito(32, 'A', Jog, P.Y - 80, P.X);` |

## 🌈 15. Glória

- Source: [`C15.rot` lines 71–184](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L71)
- SHA/blob: `08e71835d1e7ae004063c57947c411dfa98d55f8`
- `[Quadro]` source lines: 114
- Indexed event statements: 71
- Category counts: `timer/branch` 38; `frame` 19; `effect` 10; `sound` 5; `simulation mutation` 24; `RNG` 2

| Source line | Category | Ordered source statement |
|---:|---|---|
| [76](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L76) | timer/branch, frame | `if P.QuadroAcao <= 10 then P.Quadro := Desloc + 33;` |
| [77](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L77) | timer/branch, frame | `if P.QuadroAcao = 10 then P.Quadro := Desloc + 34;` |
| [78](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L78) | timer/branch, frame | `if P.QuadroAcao = 14 then P.Quadro := Desloc + 35;` |
| [82](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L82) | timer/branch, frame | `if P.QuadroAcao = 0 then P.Quadro := Desloc + 36;` |
| [83](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L83) | timer/branch, frame | `if P.QuadroAcao = 7 then P.Quadro := Desloc + 37;` |
| [84](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L84) | timer/branch | `if P.QuadroAcao = 10 then` |
| [86](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L86) | frame | `P.Quadro := Desloc + 38;` |
| [87](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L87) | effect | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgFraco) then CriaEfeito(33, 'A', Jog, P.X + 60, P.Y - 92);` |
| [88](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L88) | effect | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgFraco) then CriaEfeito(33, 'A', Jog, P.X - 60, P.Y - 92);` |
| [89](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L89) | effect | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgForte) then CriaEfeito(33, 'B', Jog, P.X + 60, P.Y - 92);` |
| [90](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L90) | effect | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgForte) then CriaEfeito(33, 'B', Jog, P.X - 60, P.Y - 92);` |
| [92](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L92) | timer/branch, sound | `if P.QuadroAcao = 3 then SomPendente := 63;` |
| [96](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L96) | timer/branch, frame | `if P.QuadroAcao = 0 then P.Quadro := Desloc + 39;` |
| [97](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L97) | timer/branch, frame | `if P.QuadroAcao = 4 then P.Quadro := Desloc + 40;` |
| [98](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L98) | timer/branch, frame | `if P.QuadroAcao = 8 then P.Quadro := Desloc + 41;` |
| [99](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L99) | timer/branch, frame | `if P.QuadroAcao = 12 then P.Quadro := Desloc + 42;` |
| [100](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L100) | timer/branch | `if (P.QuadroAcao >= 2) and (P.QuadroAcao <= 4) and (P.ForcaGolpe = fgForte) then` |
| [102](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L102) | simulation mutation | `if P.Lado = ldDireita then Empurra(Jog, 15);` |
| [103](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L103) | simulation mutation | `if P.Lado = ldEsquerda then Empurra(Jog, -15);` |
| [105](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L105) | timer/branch | `if P.QuadroAcao = 3 then` |
| [107](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L107) | effect | `if P.Lado = ldDireita then CriaEfeito(34, 'A', Jog, P.X + 65, P.Y - 30);` |
| [108](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L108) | effect | `if P.Lado = ldEsquerda then CriaEfeito(34, 'A', Jog, P.X - 65, P.Y - 30);` |
| [110](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L110) | timer/branch, sound | `if P.QuadroAcao = 1 then SomPendente := 64;` |
| [114](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L114) | timer/branch, frame | `if P.QuadroAcao = 0 then P.Quadro := Desloc + 43;` |
| [115](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L115) | timer/branch, frame | `if P.QuadroAcao = 2 then P.Quadro := Desloc + 44;` |
| [116](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L116) | timer/branch, frame | `if P.QuadroAcao = 4 then P.Quadro := Desloc + 45;` |
| [117](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L117) | timer/branch, frame | `if P.QuadroAcao = 7 then P.Quadro := Desloc + 46;` |
| [118](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L118) | timer/branch, frame | `if P.QuadroAcao = 10 then P.Quadro := Desloc + 47;` |
| [119](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L119) | timer/branch, frame | `if P.QuadroAcao = 13 then P.Quadro := Desloc + 48;` |
| [120](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L120) | timer/branch, frame | `if P.QuadroAcao = 16 then P.Quadro := Desloc + 12;` |
| [121](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L121) | timer/branch | `if P.QuadroAcao = 5 then` |
| [123](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L123) | effect | `if P.Lado = ldDireita then CriaEfeito(35, 'A', Jog, P.X + 40, P.Y - 160);` |
| [124](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L124) | effect | `if P.Lado = ldEsquerda then CriaEfeito(35, 'A', Jog, P.X - 40, P.Y - 160);` |
| [126](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L126) | timer/branch | `if (P.QuadroAcao >= 0) and (P.QuadroAcao <= 8) then` |
| [128](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L128) | simulation mutation | `if P.ForcaGolpe = fgFraco then P.Y := P.Y - 6;` |
| [131](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L131) | simulation mutation | `P.Y := P.Y - 10;` |
| [132](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L132) | timer/branch | `if P.QuadroAcao <= 2 then` |
| [134](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L134) | simulation mutation | `if P.Lado = ldDireita then Empurra(Jog, 10);` |
| [135](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L135) | simulation mutation | `if P.Lado = ldEsquerda then Empurra(Jog, -10);` |
| [137](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L137) | timer/branch | `if P.QuadroAcao >= 3 then` |
| [139](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L139) | simulation mutation | `if P.Lado = ldDireita then Empurra(Jog, 3);` |
| [140](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L140) | simulation mutation | `if P.Lado = ldEsquerda then Empurra(Jog, -3);` |
| [144](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L144) | timer/branch | `if P.QuadroAcao = 9 then` |
| [146](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L146) | simulation mutation | `P.NoAr := True;` |
| [147](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L147) | simulation mutation | `P.VelY := -5;` |
| [149](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L149) | timer/branch, sound | `if P.QuadroAcao = 1 then SomPendente := 65;` |
| [150](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L150) | simulation mutation | `P.EhChute := True;` |
| [151](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L151) | timer/branch, simulation mutation | `if P.QuadroAcao <= 3 then O.Invulneravel := False;` |
| [155](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L155) | frame | `P.Quadro := Desloc + 49;` |
| [156](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L156) | timer/branch | `S_2 := 18 - P.QuadroAcao;` |
| [157](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L157) | simulation mutation | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgFraco) then Empurra(Jog, S_2);` |
| [158](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L158) | simulation mutation | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgFraco) then Empurra(Jog, -S_2);` |
| [159](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L159) | simulation mutation | `if (P.Lado = ldDireita) and (P.ForcaGolpe = fgForte) then Empurra(Jog, S_2 * 1.3);` |
| [160](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L160) | simulation mutation | `if (P.Lado = ldEsquerda) and (P.ForcaGolpe = fgForte) then Empurra(Jog, -(S_2 * 1.3));` |
| [161](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L161) | timer/branch | `S_2 := ShortInt(P.QuadroAcao + 248);` |
| [162](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L162) | simulation mutation | `P.Y := S_2 + P.Y;` |
| [163](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L163) | timer/branch, sound | `if P.QuadroAcao = 1 then SomPendente := 66;` |
| [164](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L164) | simulation mutation | `P.EhChute := True;` |
| [165](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L165) | timer/branch, simulation mutation | `if P.QuadroAcao <= 3 then O.Invulneravel := False;` |
| [169](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L169) | frame | `P.Quadro := Desloc + 50;` |
| [170](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L170) | timer/branch | `S_2 := 22 - P.QuadroAcao;` |
| [172](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L172) | simulation mutation | `if P.Lado = ldDireita then Empurra(Jog, S_2);` |
| [173](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L173) | simulation mutation | `if P.Lado = ldEsquerda then Empurra(Jog, -S_2);` |
| [174](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L174) | timer/branch | `if P.QuadroAcao < 20 then` |
| [176](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L176) | effect, RNG | `if P.Lado = ldDireita then CriaEfeito(36, 'A', Jog, P.X - 30, Random(20) + (P.Y - 110));` |
| [177](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L177) | effect, RNG | `if P.Lado = ldEsquerda then CriaEfeito(36, 'A', Jog, P.X + 30, Random(20) + (P.Y - 110));` |
| [179](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L179) | timer/branch | `S_2 := ShortInt(Round(P.QuadroAcao * 0.4) + 251);` |
| [180](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L180) | simulation mutation | `P.Y := S_2 + P.Y;` |
| [181](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L181) | timer/branch, sound | `if P.QuadroAcao = 1 then SomPendente := 66;` |
| [182](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L182) | timer/branch, simulation mutation | `if P.QuadroAcao >= 15 then P.EhChute := True;` |
| [183](https://github.com/danimesq/OpenStreetChaves/blob/e1b025c35ecbc2b14d8635564aafd23c2107d3d1/Dados/Lutadores/C15.rot#L183) | timer/branch, simulation mutation | `if P.QuadroAcao <= 15 then O.Invulneravel := False;` |

## 🌈 Reading rules and unresolved work

1. **Do not sort by timer value.** Source order is execution order within the script. Several independent `if` statements may match on the same tick, and later assignments can overwrite earlier ones.
2. **Do not collapse repeated RNG calls.** A random check that fails still consumes RNG state. Loops such as `for Cont := 1 to N` consume multiple values and create multiple effects.
3. **Do not treat frame selection as presentation-only.** The scripts can move fighters, change airborne/invulnerability flags, mutate opponent state, queue sounds, spawn effects, and directly apply damage.
4. **Common script order matters.** Each fighter's `[Quadro]` runs after `Comum.rot`'s `[Quadro]`; the Pascal project documentation describes that interpreter order. This ledger covers only the character-specific section.
5. **This is not a complete tick trace.** Exact execution requires expanding nested `begin/end` guards and loops and combining the common section, fighter section, box application, and outer fight-loop order.

### 🌈 Next normalization pass

- [ ] Convert each source-ordered ledger into action-specific timelines with fully expanded guard paths.
- [ ] Record every timer tick that can change frame, state, motion, effect, sound, or hit qualification.
- [ ] Preserve sequential RNG call indices per branch and loop iteration.
- [ ] Join frame/action timelines to hitbox and hurtbox activation rows.
- [ ] Add deterministic fixture candidates for chained-hit, multi-hit, camera-relative, and RNG-heavy actions.
- [ ] Compare these Pascal timelines with the C frame-selector tables before assigning original-game semantics.

The rows are source-derived evidence, not a substitute for executable observation or tests.
