#!/usr/bin/env python3
"""Normalize Street Chaves .rot scripts into source-preserving symbolic ASTs.

Parses the dialect implemented by OpenStreetChaves/source/game/ChavesRoteiro.pas.
It never executes scripts or guesses game state: guards, loops, case dispatch,
short-circuit rules, call sites and source order remain explicit.
"""
from __future__ import annotations

import argparse
import hashlib
import json
import re
import sys
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Iterable

PARSER_VERSION = "0.1.0"
UPSTREAM_COMMIT = "e1b025c35ecbc2b14d8635564aafd23c2107d3d1"


@dataclass(frozen=True)
class Token:
    kind: str
    text: str
    value: Any
    line: int
    column: int
    start: int
    end: int


class RotError(Exception):
    def __init__(self, source: str, line: int, column: int, message: str):
        super().__init__(f"{source}:{line}:{column}: {message}")


def lex(source_text: str, source_name: str, first_line: int = 1) -> list[Token]:
    tokens: list[Token] = []
    i, line, col, n = 0, first_line, 1, len(source_text)

    def advance() -> str:
        nonlocal i, line, col
        ch = source_text[i]
        i += 1
        if ch == "\n":
            line, col = line + 1, 1
        else:
            col += 1
        return ch

    def add(kind: str, start: int, ln: int, cl: int, value: Any = None):
        raw = source_text[start:i]
        tokens.append(Token(kind, raw, raw if value is None else value, ln, cl, start, i))

    while i < n:
        ch = source_text[i]
        if ch.isspace():
            advance()
            continue
        if source_text.startswith("//", i):
            while i < n and source_text[i] != "\n":
                advance()
            continue
        if ch == "{":
            ln, cl = line, col
            advance()
            while i < n and source_text[i] != "}":
                advance()
            if i >= n:
                raise RotError(source_name, ln, cl, "unterminated { comment")
            advance()
            continue
        if source_text.startswith("(*", i):
            ln, cl = line, col
            advance(); advance()
            while i < n and not source_text.startswith("*)", i):
                advance()
            if i >= n:
                raise RotError(source_name, ln, cl, "unterminated (* comment *)")
            advance(); advance()
            continue

        start, ln, cl = i, line, col
        if ch == "'":
            advance()
            value = ""
            while i < n:
                if source_text[i] == "'":
                    advance()
                    if i < n and source_text[i] == "'":
                        advance()
                        value += "'"
                        continue
                    break
                value += advance()
            else:
                raise RotError(source_name, ln, cl, "unterminated string literal")
            add("string", start, ln, cl, value)
            continue

        if ch == "$":
            advance()
            digits = i
            while i < n and source_text[i] in "0123456789abcdefABCDEF":
                advance()
            if i == digits:
                raise RotError(source_name, ln, cl, "hex literal needs digits")
            add("integer", start, ln, cl, int(source_text[digits:i], 16))
            continue

        if ch.isdigit():
            while i < n and source_text[i].isdigit():
                advance()
            real = False
            if i + 1 < n and source_text[i] == "." and source_text[i + 1].isdigit():
                real = True
                advance()
                while i < n and source_text[i].isdigit():
                    advance()
                if i < n and source_text[i] in "eE":
                    advance()
                    if i < n and source_text[i] in "+-":
                        advance()
                    exp = i
                    while i < n and source_text[i].isdigit():
                        advance()
                    if exp == i:
                        raise RotError(source_name, ln, cl, "malformed real exponent")
            raw = source_text[start:i]
            add("real" if real else "integer", start, ln, cl, float(raw) if real else int(raw))
            continue

        if ch.isalpha() or ch == "_":
            advance()
            while i < n and (source_text[i].isalnum() or source_text[i] == "_"):
                advance()
            add("identifier", start, ln, cl, source_text[start:i])
            continue

        pair = source_text[i:i + 2]
        if pair in {":=", "<=", ">=", "<>", ".."}:
            advance(); advance()
            add("symbol", start, ln, cl)
            continue
        if ch in "+-*/=<>()[].,:;":
            advance()
            add("symbol", start, ln, cl)
            continue
        raise RotError(source_name, line, col, f"unexpected character {ch!r}")

    tokens.append(Token("eof", "", None, line, col, n, n))
    return tokens


@dataclass
class Section:
    name: str
    text: str
    first_line: int
    source: str


def load_sections(path: Path) -> dict[str, Section]:
    lines = path.read_text(encoding="utf-8-sig").splitlines(keepends=True)
    sections: dict[str, Section] = {}
    name: str | None = None
    body: list[str] = []
    first_line = 1

    def finish():
        if name is not None:
            sections[name.lower()] = Section(name, "".join(body), first_line, str(path))

    for line_no, line in enumerate(lines, 1):
        m = re.fullmatch(r"\[([^\]]+)\]", line.strip())
        if m:
            finish()
            name = m.group(1).strip()
            body = []
            first_line = line_no + 1
        elif name is not None:
            # ChavesRoteiro.CarregarRoteiro treats semicolon-prefixed whole lines as comments.
            body.append("\n" if line.lstrip().startswith(";") and line.endswith("\n")
                        else ("" if line.lstrip().startswith(";") else line))
    finish()
    return sections


class Parser:
    def __init__(self, text: str, source: str, first_line: int = 1):
        self.text, self.source = text, source
        self.tokens = lex(text, source, first_line)
        self.pos = 0
        self.locals: list[dict[str, str]] = []

    @property
    def tok(self) -> Token:
        return self.tokens[self.pos]

    def at(self, value: str) -> bool:
        return self.tok.text.lower() == value.lower()

    def take(self) -> Token:
        t = self.tok
        if t.kind != "eof":
            self.pos += 1
        return t

    def expect(self, value: str) -> Token:
        if not self.at(value):
            raise RotError(self.source, self.tok.line, self.tok.column,
                           f"expected {value!r}, found {self.tok.text!r}")
        return self.take()

    def expect_kind(self, kind: str) -> Token:
        if self.tok.kind != kind:
            raise RotError(self.source, self.tok.line, self.tok.column,
                           f"expected {kind}, found {self.tok.text!r}")
        return self.take()

    def node(self, kind: str, start: Token, end: Token | None = None, **fields: Any):
        end = end or self.tokens[max(0, self.pos - 1)]
        return {"kind": kind,
                "span": {"line": start.line, "column": start.column,
                         "end_line": end.line, "end_column": end.column,
                         "start_offset": start.start, "end_offset": end.end},
                **fields}

    def parse(self):
        start = self.tok
        declarations = []
        while self.at("var"):
            declarations.extend(self.parse_var())
        body = self.parse_list(set())
        if self.tok.kind != "eof":
            raise RotError(self.source, self.tok.line, self.tok.column,
                           f"unexpected trailing token {self.tok.text!r}")
        return {"kind": "section_ast", "span": {"line": start.line, "column": start.column},
                "declarations": declarations, "body": body}

    def parse_var(self):
        self.expect("var")
        names = [self.expect_kind("identifier")]
        while self.at(","):
            self.take()
            names.append(self.expect_kind("identifier"))
        self.expect(":")
        typ = self.expect_kind("identifier")
        end = self.expect(";")
        out = []
        for name in names:
            self.locals.append({"name": name.text, "type": typ.text})
            out.append({"name": name.text, "type": typ.text,
                        "span": {"line": name.line, "column": name.column,
                                 "end_line": end.line, "end_column": end.column}})
        return out

    def parse_list(self, stop: set[str]):
        start, items = self.tok, []
        while self.tok.kind != "eof" and self.tok.text.lower() not in stop:
            if self.at(";"):
                self.take()
                continue
            before = self.pos
            items.append(self.statement())
            if self.at(";"):
                self.take()
            elif self.tok.kind != "eof" and self.tok.text.lower() not in stop:
                raise RotError(self.source, self.tok.line, self.tok.column,
                               f"expected ';', found {self.tok.text!r}")
            if before == self.pos:
                raise RotError(self.source, self.tok.line, self.tok.column, "parser made no progress")
        return self.node("block", start, statements=items)

    def statement_or_empty(self):
        if self.at("else") or self.at(";") or self.at("end") or self.at("until"):
            return self.node("block", self.tok, statements=[])
        return self.statement()

    def statement(self):
        start = self.tok
        if self.at("begin"):
            self.take()
            body = self.parse_list({"end"})
            end = self.expect("end")
            return self.node("block", start, end, statements=body["statements"])
        if self.at("if"):
            self.take()
            cond = self.expr()
            self.expect("then")
            yes = self.statement_or_empty()
            no = None
            if self.at("else"):
                self.take()
                no = self.statement_or_empty()
            return self.node("if", start, condition=cond, then=yes, otherwise=no)
        if self.at("for"):
            self.take()
            var = self.expect_kind("identifier").text
            self.expect(":=")
            first = self.expr()
            direction = "to" if self.at("to") else "downto"
            self.expect(direction)
            last = self.expr()
            self.expect("do")
            body = self.statement_or_empty()
            return self.node("for", start, variable=var, direction=direction,
                             first=first, last=last, body=body)
        if self.at("repeat"):
            self.take()
            body = self.parse_list({"until"})
            self.expect("until")
            return self.node("repeat_until", start, body=body, condition=self.expr())
        if self.at("while"):
            self.take()
            cond = self.expr()
            self.expect("do")
            return self.node("while", start, condition=cond, body=self.statement_or_empty())
        if self.at("case"):
            self.take()
            selector = self.expr()
            self.expect("of")
            arms, default = [], None
            while not self.at("end") and self.tok.kind != "eof":
                if self.at("else"):
                    self.take()
                    default = self.parse_list({"end"})
                    break
                arm_start, labels = self.tok, []
                while True:
                    low = self.expr()
                    if self.at(".."):
                        self.take()
                        labels.append(self.node("range", arm_start, low=low, high=self.expr()))
                    else:
                        labels.append(low)
                    if not self.at(","):
                        break
                    self.take()
                self.expect(":")
                arms.append(self.node("case_arm", arm_start, labels=labels,
                                      body=self.statement_or_empty()))
                if self.at(";"):
                    self.take()
            end = self.expect("end")
            return self.node("case", start, end, selector=selector, arms=arms, default=default)

        target = self.factor()
        if self.at(":="):
            if target["kind"] not in {"identifier", "field"}:
                raise RotError(self.source, start.line, start.column, "invalid assignment target")
            self.take()
            return self.node("assign", start, target=target, value=self.expr())
        if target["kind"] != "call":
            raise RotError(self.source, start.line, start.column,
                           "statement must be an assignment or procedure call")
        return self.node("call_statement", start, call=target)

    def expr(self):
        left = self.simple()
        while self.tok.text.lower() in {"=", "<>", "<", ">", "<=", ">=", "in"}:
            op = self.take()
            right = self.simple()
            left = self.binary(op, left, right)
        return left

    def simple(self):
        left = self.term()
        while self.tok.text.lower() in {"+", "-", "or", "xor"}:
            left = self.binary(self.take(), left, self.term())
        return left

    def term(self):
        left = self.factor()
        while self.tok.text.lower() in {"*", "/", "div", "mod", "and", "shl", "shr"}:
            left = self.binary(self.take(), left, self.factor())
        return left

    def binary(self, op, left, right):
        n = self.node("binary", op, op=op.text.lower(), left=left, right=right)
        n["span"].update(line=left["span"]["line"], column=left["span"]["column"],
                         start_offset=left["span"]["start_offset"],
                         end_line=right["span"]["end_line"], end_column=right["span"]["end_column"],
                         end_offset=right["span"]["end_offset"])
        return n

    def factor(self):
        start = self.tok
        if self.at("not") or self.at("+") or self.at("-"):
            op = self.take()
            return self.node("unary", start, op=op.text.lower(), value=self.factor())
        if self.at("("):
            self.take()
            value = self.expr()
            return self.node("group", start, self.expect(")"), value=value)
        if self.at("["):
            self.take()
            items = []
            while not self.at("]"):
                low = self.expr()
                if self.at(".."):
                    self.take()
                    items.append(self.node("range", start, low=low, high=self.expr()))
                else:
                    items.append(low)
                if not self.at(","):
                    break
                self.take()
            return self.node("set", start, self.expect("]"), items=items)
        if self.tok.kind in {"integer", "real", "string"}:
            t = self.take()
            return self.node("literal", t, value=t.value, literal_type=t.kind, raw=t.text)
        if self.tok.kind != "identifier":
            raise RotError(self.source, self.tok.line, self.tok.column,
                           f"unexpected expression token {self.tok.text!r}")
        name = self.take()
        if name.text.lower() in {"true", "false"}:
            return self.node("literal", name, value=name.text.lower() == "true",
                             literal_type="boolean", raw=name.text)
        if name.text.lower() in {"p", "o"} and self.at("."):
            self.take()
            field = self.expect_kind("identifier")
            return self.node("field", name, field, base=name.text.upper(), name=field.text)
        if self.at("("):
            self.take()
            args = []
            if not self.at(")"):
                while True:
                    args.append(self.expr())
                    if not self.at(","):
                        break
                    self.take()
            return self.node("call", name, self.expect(")"), name=name.text, args=args)
        return self.node("identifier", name, name=name.text)


def expr_text(node):
    k = node["kind"]
    if k == "literal":
        if node["literal_type"] == "string": return "'" + str(node["value"]).replace("'", "''") + "'"
        if node["literal_type"] == "boolean": return "True" if node["value"] else "False"
        return node["raw"]
    if k == "identifier": return node["name"]
    if k == "field": return f"{node['base']}.{node['name']}"
    if k == "group": return f"({expr_text(node['value'])})"
    if k == "unary": return f"{node['op']} {expr_text(node['value'])}"
    if k == "binary": return f"{expr_text(node['left'])} {node['op']} {expr_text(node['right'])}"
    if k == "call": return f"{node['name']}({', '.join(expr_text(x) for x in node['args'])})"
    if k == "set": return "[" + ", ".join(expr_text(x) for x in node["items"]) + "]"
    if k == "range": return f"{expr_text(node['low'])}..{expr_text(node['high'])}"
    return f"<{k}>"


def walk_expr(node):
    yield node
    k = node["kind"]
    if k == "binary":
        yield from walk_expr(node["left"]); yield from walk_expr(node["right"])
    elif k in {"unary", "group"}:
        yield from walk_expr(node["value"])
    elif k == "call":
        for arg in node["args"]: yield from walk_expr(arg)
    elif k == "set":
        for item in node["items"]: yield from walk_expr(item)
    elif k == "range":
        yield from walk_expr(node["low"]); yield from walk_expr(node["high"])


def infer_random_width(arg):
    # ChavesRoteiro marks Round/Trunc as Int64; it dispatches Random to Sortear64.
    for sub in walk_expr(arg):
        if sub["kind"] == "call" and sub["name"].lower() in {"round", "trunc"}:
            return "int64"
        if sub["kind"] == "literal" and sub.get("literal_type") == "integer":
            if not -(2**31) <= int(sub["value"]) <= 2**31 - 1:
                return "int64"
    return "int32"


def guard_hints(guards):
    return ([g for g in guards if re.search(r"\bP\.Acao\b", g, re.I)],
            [g for g in guards if re.search(r"\bP\.QuadroAcao\b", g, re.I)])


class EventExtractor:
    def __init__(self, character, section, argument_order):
        self.character, self.section = character, section
        self.argument_order, self.events = argument_order, []

    def emit(self, kind, node, guards, loops, **extra):
        action, tick = guard_hints(guards)
        event = {"sequence": len(self.events), "character": self.character, "section": self.section,
                 "kind": kind, "source_line": node["span"]["line"],
                 "source_column": node["span"]["column"], "guards": list(guards),
                 "loop_context": list(loops), "symbolic_action_context": action,
                 "symbolic_tick_context": tick, "runtime_tick": None,
                 "runtime_rng_draw_index": None, **extra}
        self.events.append(event)
        return event

    def expr(self, node, guards, loops, arg_context=None):
        k = node["kind"]
        if k == "binary":
            op = node["op"]
            self.expr(node["left"], guards, loops, arg_context)
            if op in {"and", "or"}:
                left = expr_text(node["left"])
                predicate = f"({left}) is True" if op == "and" else f"({left}) is False"
                rhs_guard = f"RHS skipped only if left is Boolean and {predicate}; integer bitwise {op} evaluates RHS"
                self.expr(node["right"], guards + [rhs_guard], loops, arg_context)
            else:
                self.expr(node["right"], guards, loops, arg_context)
        elif k in {"unary", "group"}:
            self.expr(node["value"], guards, loops, arg_context)
        elif k == "call":
            args = node["args"]
            order = ("single-argument" if len(args) <= 1 else
                     ("compiler-profile-dependent" if self.argument_order == "unknown" else self.argument_order))
            indices = list(range(len(args)))
            if self.argument_order == "right-to-left":
                indices.reverse()
            for i in indices:
                self.expr(args[i], guards, loops, f"{node['name']} arg[{i}] ({order})")
            name = node["name"].lower()
            if name == "random":
                self.emit("rng_draw", node, guards, loops,
                          rng_site=f"{self.character}.{self.section}.L{node['span']['line']}.C{node['span']['column']}",
                          function="Random", range_expression=expr_text(args[0]),
                          random_width=infer_random_width(args[0]), argument_evaluation=order,
                          argument_context=arg_context,
                          draw_index_semantics="assigned only during concrete execution")
            elif name in {"criaefeito", "empurra", "acaojogador", "aplicarcaixas"}:
                self.emit("simulation_call", node, guards, loops, function=node["name"],
                          arguments=[expr_text(x) for x in args], argument_evaluation=order,
                          argument_context=arg_context)
        elif k == "set":
            for x in node["items"]: self.expr(x, guards, loops, arg_context)
        elif k == "range":
            self.expr(node["low"], guards, loops, arg_context)
            self.expr(node["high"], guards, loops, arg_context)

    def statement(self, node, guards=None, loops=None):
        guards, loops = list(guards or []), list(loops or [])
        k = node["kind"]
        if k == "block":
            for child in node["statements"]: self.statement(child, guards, loops)
        elif k == "if":
            self.expr(node["condition"], guards, loops)
            cond = expr_text(node["condition"])
            self.statement(node["then"], guards + [cond], loops)
            if node["otherwise"] is not None:
                self.statement(node["otherwise"], guards + [f"not ({cond})"], loops)
        elif k == "for":
            self.expr(node["first"], guards, loops)
            self.expr(node["last"], guards, loops)
            region = f"for {node['variable']} := {expr_text(node['first'])} {node['direction']} {expr_text(node['last'])}"
            marker = self.emit("loop_region", node, guards, loops, loop_type="for",
                               variable=node["variable"], direction=node["direction"],
                               first=expr_text(node["first"]), last=expr_text(node["last"]),
                               body_event_count=None)
            before = len(self.events)
            self.statement(node["body"], guards + [region], loops + [region])
            marker["body_event_count"] = len(self.events) - before
        elif k == "while":
            region = f"while {expr_text(node['condition'])}"
            marker = self.emit("loop_region", node, guards, loops, loop_type="while",
                               condition=expr_text(node["condition"]), iteration_event_count=None)
            before = len(self.events)
            self.expr(node["condition"], guards + [region + " (tested each iteration)"], loops + [region])
            self.statement(node["body"], guards + [region], loops + [region])
            marker["iteration_event_count"] = len(self.events) - before
        elif k == "repeat_until":
            region = f"repeat until {expr_text(node['condition'])}"
            marker = self.emit("loop_region", node, guards, loops, loop_type="repeat_until",
                               condition=expr_text(node["condition"]), iteration_event_count=None)
            before = len(self.events)
            self.statement(node["body"], guards + [region], loops + [region])
            self.expr(node["condition"], guards + [region + " (tested after body)"], loops + [region])
            marker["iteration_event_count"] = len(self.events) - before
        elif k == "case":
            self.expr(node["selector"], guards, loops)
            selector = expr_text(node["selector"])
            for ai, arm in enumerate(node["arms"]):
                reached = f"case dispatch reaches arm {ai + 1}"
                for label in arm["labels"]:
                    if label["kind"] == "range":
                        self.expr(label["low"], guards + [reached], loops)
                        self.expr(label["high"], guards + [reached], loops)
                    else:
                        self.expr(label, guards + [reached], loops)
                labels = [expr_text(x) for x in arm["labels"]]
                self.statement(arm["body"], guards + [f"case {selector} matches {' or '.join(labels)}"], loops)
            if node["default"] is not None:
                self.statement(node["default"], guards + [f"case {selector} matches no listed arm"], loops)
        elif k == "assign":
            self.expr(node["value"], guards, loops)
            target = expr_text(node["target"])
            globals_ = {"sompendente", "flashtela", "camerax", "cameray", "ganhocarga",
                        "pausaimpacto", "tremortela", "sangue", "flash", "gravidade",
                        "niveljogo", "fase", "contpisca", "flashvermelho"}
            kind = "state_write" if node["target"]["kind"] == "field" or target.lower() in globals_ else "local_or_global_assignment"
            self.emit(kind, node, guards, loops, target=target, value_expression=expr_text(node["value"]))
        elif k == "call_statement":
            self.expr(node["call"], guards, loops)
            name = node["call"]["name"].lower()
            if name not in {"random", "criaefeito", "empurra", "acaojogador", "aplicarcaixas"}:
                self.emit("procedure_call", node, guards, loops, function=node["call"]["name"],
                          arguments=[expr_text(x) for x in node["call"]["args"]])


def normalize_section(section, character, argument_order):
    parser = Parser(section.text, section.source + f" [{section.name}]", section.first_line)
    ast = parser.parse()
    extractor = EventExtractor(character, section.name, argument_order)
    extractor.statement(ast["body"])
    events = extractor.events
    return {"source": section.source, "section": section.name, "first_line": section.first_line,
            "local_declarations": ast["declarations"], "ast": ast, "events": events,
            "counts": {"events": len(events),
                       "rng_draw_sites": sum(e["kind"] == "rng_draw" for e in events),
                       "loops": sum(e["kind"] == "loop_region" for e in events),
                       "state_writes": sum(e["kind"] == "state_write" for e in events),
                       "simulation_calls": sum(e["kind"] == "simulation_call" for e in events)}}


def strip_pascal_comments(text):
    """Replace Pascal comments and strings by spaces while preserving newlines."""
    out = list(text)
    i, n = 0, len(text)
    while i < n:
        if text.startswith("//", i):
            while i < n and text[i] != "\n":
                out[i] = " "; i += 1
            continue
        if text[i] == "{" or text.startswith("(*", i):
            close = "}" if text[i] == "{" else "*)"
            width = len(close)
            for j in range(width): out[i + j] = " "
            i += width
            while i < n and not text.startswith(close, i):
                if text[i] != "\n": out[i] = " "
                i += 1
            if i < n:
                for j in range(width): out[i + j] = " "
                i += width
            continue
        if text[i] == "'":
            out[i] = " "; i += 1
            while i < n:
                if text[i] == "'":
                    out[i] = " "; i += 1
                    if i < n and text[i] == "'":
                        out[i] = " "; i += 1
                        continue
                    break
                if text[i] != "\n": out[i] = " "
                i += 1
            continue
        i += 1
    return "".join(out)


def inventory_pascal_rng(source_root):
    sites = []
    pattern = re.compile(r"\b(Randomize|RandSeed|Random|Sortear64|Sortear)\b", re.I)
    for path in sorted(source_root.rglob("*")):
        if not path.is_file() or path.suffix.lower() not in {".pas", ".inc"}:
            continue
        raw = path.read_text(encoding="utf-8-sig", errors="replace")
        code_lines, raw_lines = strip_pascal_comments(raw).splitlines(), raw.splitlines()
        for line_no, line in enumerate(code_lines, 1):
            for m in pattern.finditer(line):
                name, tail = m.group(1), line[m.end():].lstrip()
                if re.match(r"^\s*(function|procedure)\b", line, re.I):
                    continue
                if name.lower() in {"random", "sortear", "sortear64"} and not tail.startswith("("):
                    continue
                if name.lower() == "randomize" and not (tail.startswith(";") or tail.startswith("end") or not tail):
                    continue
                if name.lower() == "randseed":
                    kind = "seed_assignment" if tail.startswith(":=") else "seed_state_reference"
                elif name.lower() == "randomize":
                    kind = "seed_initialization"
                else:
                    kind = "random_draw_call"
                sites.append({"path": str(path.relative_to(source_root)), "line": line_no,
                              "column": m.start() + 1, "name": name, "kind": kind,
                              "source_excerpt": raw_lines[line_no - 1].strip() if line_no <= len(raw_lines) else ""})
    return sites


def sha256_file(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def git_blob_sha1(path):
    data = path.read_bytes()
    return hashlib.sha1(f"blob {len(data)}\0".encode("ascii") + data).hexdigest()


def build_normalization(input_dir, output_dir, source_root, argument_order):
    output_dir.mkdir(parents=True, exist_ok=True)
    common_path = input_dir / "Comum.rot"
    if not common_path.is_file():
        raise FileNotFoundError(f"missing common script: {common_path}")
    common_sections = load_sections(common_path)
    pin_path = input_dir / "source-manifest.json"
    pin = json.loads(pin_path.read_text(encoding="utf-8")) if pin_path.exists() else None
    pinned = {x["file"]: x["blob_sha"] for x in pin["files"]} if pin else {}

    def check_pin(path):
        expected, actual = pinned.get(path.name), git_blob_sha1(path)
        if expected and actual != expected:
            raise ValueError(f"pinned source mismatch for {path.name}: expected {expected}, got {actual}")
        return expected

    common_blob = check_pin(common_path)
    fighters, inputs = [], [{"file": common_path.name, "sha256": sha256_file(common_path),
                             "upstream_blob_sha": common_blob}]
    for i in range(1, 16):
        path = input_dir / f"C{i:02d}.rot"
        if not path.is_file():
            raise FileNotFoundError(f"missing fighter script: {path}")
        fighter_sections = load_sections(path)
        blob = check_pin(path)
        fighter = {"fighter_id": i, "source_file": path.name, "source_sha256": sha256_file(path),
                   "upstream_blob_sha": blob, "sections": {}}
        inputs.append({"file": path.name, "sha256": fighter["source_sha256"], "upstream_blob_sha": blob})
        for section_name in ("acao", "quadro", "ia", "acerto"):
            parts = []
            if section_name in common_sections:
                parts.append(normalize_section(common_sections[section_name], "Common", argument_order))
            if section_name in fighter_sections:
                parts.append(normalize_section(fighter_sections[section_name], f"C{i:02d}", argument_order))
            combined = []
            for pi, part in enumerate(parts):
                for event in part["events"]:
                    item = dict(event)
                    item["component_order"] = pi
                    item["combined_sequence"] = len(combined)
                    combined.append(item)
            fighter["sections"][section_name] = {
                "execution_order": [p["source"] for p in parts],
                "components": parts,
                "symbolic_events": combined,
                "counts": {"events": len(combined),
                           "rng_draw_sites": sum(e["kind"] == "rng_draw" for e in combined),
                           "loops": sum(e["kind"] == "loop_region" for e in combined),
                           "state_writes": sum(e["kind"] == "state_write" for e in combined)}}
        fighters.append(fighter)

    result = {
        "schema": "street-chaves-rot-normalization/v1",
        "normalizer_version": PARSER_VERSION,
        "reference_policy": {
            "normative_source": "Pascal/FPC ChavesRoteiro interpreter and .rot data",
            "comparison_source": "C17/SDL3 port; divergences are annotations, not overrides",
            "conflict_resolution": "follow Pascal until original-executable evidence justifies a documented change",
            "original_executable": "reverse-engineering confirmation required for disputed behavior",
            "upstream_repository": "https://github.com/danimesq/OpenStreetChaves",
            "upstream_commit": UPSTREAM_COMMIT,
            "argument_evaluation_profile": argument_order},
        "timeline_semantics": {"kind": "symbolic_static_trace", "runtime_ticks_are_not_simulated": True,
                               "runtime_rng_draw_indices_are_not_assigned": True,
                               "source_order_is_preserved": True,
                               "short_circuit_and_loop_contexts_are_preserved": True,
                               "common_sections_run_before_fighter_sections": True},
        "inputs": inputs, "fighters": fighters}
    if pin:
        result["source_pin_manifest"] = pin
    if source_root is not None:
        sites = inventory_pascal_rng(source_root)
        result["pascal_rng_inventory"] = {
            "source_root": str(source_root), "site_count": len(sites), "sites": sites,
            "note": "Static source-call-site inventory. Dynamic order requires fight-loop phase order and branch outcomes."}
        (output_dir / "pascal-rng-call-sites.json").write_text(
            json.dumps(result["pascal_rng_inventory"], ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    (output_dir / "normalized-timelines.json").write_text(
        json.dumps(result, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
    (output_dir / "source-manifest.json").write_text(
        json.dumps({"upstream_commit": UPSTREAM_COMMIT, "files": inputs}, indent=2) + "\n", encoding="utf-8")
    (output_dir / "timeline-summary.md").write_text(render_summary(result), encoding="utf-8")
    return result


def render_summary(result):
    lines = ["# Symbolic .rot timelines (Pascal normative source)", "",
             f"Normalizer: {result['normalizer_version']}",
             f"Upstream: OpenStreetChaves@{UPSTREAM_COMMIT}",
             f"Argument evaluation profile: {result['reference_policy']['argument_evaluation_profile']}", "",
             "> Static symbolic trace, not a concrete execution. Branches and loops remain symbolic; runtime ticks and dynamic RNG indices are unassigned.", "",
             "## Roster summary", "",
             "| ID | Fighter | [Acao] | [Quadro] | [IA] | [Acerto] | RNG sites | Loops |",
             "|---:|---|---:|---:|---:|---:|---:|---:|"]
    for f in result["fighters"]:
        sec = f["sections"]
        counts = [sec[s]["counts"]["events"] for s in ("acao", "quadro", "ia", "acerto")]
        rng = sum(x["counts"]["rng_draw_sites"] for x in sec.values())
        loops = sum(x["counts"]["loops"] for x in sec.values())
        lines.append(f"| {f['fighter_id']} | {f['source_file']} | {counts[0]} | {counts[1]} | {counts[2]} | {counts[3]} | {rng} | {loops} |")
    lines += ["", "## Interpretation rules", "",
              "- Original source order and line numbers are retained.",
              "- Common section events precede fighter-specific events.",
              "- Nested if/else, case, for, while, and repeat/until structure remains explicit.",
              "- Boolean and/or can short-circuit; integer bitwise operands evaluate both sides in the Pascal interpreter.",
              "- A Random(...) occurrence is a static site, not a concrete RNG draw; draw indices appear only during execution/replay.",
              "- Multiple function arguments retain the compiler argument-order profile. Use unknown until measured.",
              "- Pascal is normative; C-port mismatches are documented separately and do not override this output.", ""]
    if "pascal_rng_inventory" in result:
        rng = result["pascal_rng_inventory"]
        lines += ["## Pascal source RNG sites", "",
                  f"Found {rng['site_count']} static references/calls in .pas and .inc files.", "",
                  "| Source file | Sites |", "|---|---:|"]
        groups = {}
        for site in rng["sites"]: groups.setdefault(site["path"], []).append(site)
        for path, sites in sorted(groups.items()):
            detail = ", ".join(f"{x['name']} L{x['line']}" for x in sites)
            lines.append(f"| {path} | {detail} |")
        lines.append("")
    return "\n".join(lines)


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--input-dir", type=Path, required=True,
                    help="directory containing Comum.rot, C01.rot..C15.rot, and optional source-manifest.json")
    ap.add_argument("--output-dir", type=Path, required=True)
    ap.add_argument("--pascal-source-root", type=Path,
                    help="optional upstream source root for Random/Sortear/RandSeed inventory")
    ap.add_argument("--argument-order", choices=("unknown", "left-to-right", "right-to-left"),
                    default="unknown", help="compiler argument order; keep unknown until measured")
    args = ap.parse_args(argv)
    try:
        result = build_normalization(args.input_dir, args.output_dir, args.pascal_source_root, args.argument_order)
    except (OSError, RotError, ValueError, json.JSONDecodeError) as exc:
        print(str(exc), file=sys.stderr)
        return 2
    total_rng = sum(s["counts"]["rng_draw_sites"] for f in result["fighters"] for s in f["sections"].values())
    total_events = sum(s["counts"]["events"] for f in result["fighters"] for s in f["sections"].values())
    print(f"normalized {len(result['fighters'])} fighters; {total_events} symbolic events; {total_rng} RNG sites")
    print(f"output: {args.output_dir}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
