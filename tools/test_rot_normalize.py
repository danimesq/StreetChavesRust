import importlib.util
import sys
import tempfile
import unittest
from pathlib import Path

MODULE_PATH = Path(__file__).with_name("rot-normalize.py")
spec = importlib.util.spec_from_file_location("rot_normalize", MODULE_PATH)
mod = importlib.util.module_from_spec(spec)
sys.modules[spec.name] = mod
spec.loader.exec_module(mod)


class RotNormalizeTests(unittest.TestCase):
    def parse(self, text):
        return mod.Parser(text, "fixture.rot [Quadro]").parse()

    def test_nested_if_for_repeat_case_and_short_circuit_are_preserved(self):
        source = """
var I: Integer;
if (P.Acao = acEspecial1) and (Random(2) = 0) then
begin
  for I := 1 to 2 do
  begin
    if P.QuadroAcao >= 3 then Random(Round(5.5));
  end;
end;
repeat
  Random(3);
until P.QuadroAcao = 4;
case P.Acao of
  acEspecial1: P.Quadro := 42;
else
  P.GolpeAtivo := False;
end;
"""
        ast = self.parse(source)
        extractor = mod.EventExtractor("C01", "Quadro", "unknown")
        extractor.statement(ast["body"])
        events = extractor.events
        draws = [e for e in events if e["kind"] == "rng_draw"]
        self.assertEqual(len(draws), 3)
        self.assertEqual(draws[0]["range_expression"], "2")
        self.assertEqual(draws[0]["random_width"], "int32")
        self.assertEqual(draws[1]["random_width"], "int64")
        self.assertTrue(any("RHS skipped only if left is Boolean" in g for g in draws[0]["guards"]))
        self.assertGreaterEqual(sum(e["kind"] == "loop_region" for e in events), 2)
        self.assertTrue(any(e.get("target") == "P.Quadro" for e in events))
        self.assertTrue(any(e.get("target") == "P.GolpeAtivo" for e in events))

    def test_loop_marker_precedes_body_events(self):
        ast = self.parse("var I: Integer; for I := 1 to 3 do P.Quadro := I;")
        extractor = mod.EventExtractor("C03", "Quadro", "unknown")
        extractor.statement(ast["body"])
        self.assertEqual(extractor.events[0]["kind"], "loop_region")
        self.assertEqual(extractor.events[0]["body_event_count"], 1)
        self.assertEqual(extractor.events[1]["target"], "P.Quadro")

    def test_multi_argument_rng_order_is_not_guessed(self):
        ast = self.parse("CriaEfeito(1, 'A', Jog, Random(10), Random(20));")
        extractor = mod.EventExtractor("C01", "Quadro", "unknown")
        extractor.statement(ast["body"])
        draws = [e for e in extractor.events if e["kind"] == "rng_draw"]
        self.assertEqual(len(draws), 2)
        self.assertTrue(all("compiler-profile-dependent" in e["argument_context"] for e in draws))
        self.assertEqual(draws[0]["range_expression"], "10")
        self.assertEqual(draws[1]["range_expression"], "20")

    def test_sections_ignore_semicolon_comments_and_preserve_line_numbers(self):
        with tempfile.TemporaryDirectory() as td:
            path = Path(td) / "C01.rot"
            path.write_text("; preface\n[Acao]\n; comment\nP.Acao := acParado;\n[Quadro]\nP.Quadro := 4;\n", encoding="utf-8")
            sections = mod.load_sections(path)
            self.assertEqual(set(sections), {"acao", "quadro"})
            self.assertEqual(sections["acao"].first_line, 3)
            normalized = mod.normalize_section(sections["acao"], "C01", "unknown")
            self.assertEqual(normalized["events"][0]["source_line"], 4)

    def test_pascal_comment_stripping_excludes_documentation_examples(self):
        source = """{ Random(1) in docs }\n// Random(2)\nS := 'Random(3)';\nX := Random(4);\n"""
        stripped = mod.strip_pascal_comments(source)
        self.assertNotIn("Random(1)", stripped)
        self.assertNotIn("Random(2)", stripped)
        self.assertNotIn("Random(3)", stripped)
        self.assertIn("Random(4)", stripped)


if __name__ == "__main__":
    unittest.main()
