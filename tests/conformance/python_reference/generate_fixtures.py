#!/usr/bin/env python3
"""Generate Python Rich conformance fixtures for rich_rust.

This script prefers the bundled legacy Rich snapshot (legacy_rich/) if present.
Otherwise it falls back to the installed `rich` package.
"""

from __future__ import annotations

import json
import re
import sys
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Dict

ROOT = Path(__file__).resolve().parents[3]
LEGACY_RICH = ROOT / "legacy_rich"
if LEGACY_RICH.exists():
    sys.path.insert(0, str(LEGACY_RICH))

try:
    import rich  # type: ignore
    from rich.console import Console  # type: ignore
    from rich.rule import Rule  # type: ignore
    from rich.panel import Panel  # type: ignore
    from rich.table import Table  # type: ignore
    from rich.tree import Tree  # type: ignore
    from rich.columns import Columns  # type: ignore
    from rich.padding import Padding  # type: ignore
    from rich.align import Align  # type: ignore
    from rich.progress_bar import ProgressBar  # type: ignore
    from rich.markdown import Markdown  # type: ignore
    from rich.syntax import Syntax  # type: ignore
    from rich.json import JSON  # type: ignore
    from rich import box  # type: ignore
    from rich.theme import Theme  # type: ignore
    from rich.traceback import Frame, Stack, Trace, Traceback  # type: ignore
except Exception as exc:  # pragma: no cover - import error path
    raise SystemExit(f"Failed to import rich: {exc}")


DEFAULTS = {
    "width": 40,
    "color_system": "truecolor",
    "force_terminal": True,
}

DEFAULT_ENV: Dict[str, str] = {}


CASES = [
    {
        "id": "text/plain",
        "kind": "text",
        "input": {"markup": "Hello, World!"},
    },
    {
        "id": "text/emoji_code",
        "kind": "text",
        "input": {"markup": "hi :smile:"},
    },
    {
        "id": "text/emoji_variant_text",
        "kind": "text",
        "input": {"markup": "hi :smile-text:"},
    },
    {
        "id": "text/theme_named_style",
        "kind": "text",
        "theme": {"styles": {"warning": "bold red"}, "inherit": True},
        "input": {"markup": "[warning]Danger[/]"},
    },
    {
        "id": "text/markup_bold",
        "kind": "text",
        "input": {"markup": "[bold]Bold[/]"},
    },
    {
        "id": "text/colors",
        "kind": "text",
        "input": {"markup": "[red]Red[/] and [green]Green[/]"},
    },
    {
        "id": "text/hyperlink",
        "kind": "text",
        "input": {"markup": "[link=https://example.com]Example[/]"},
    },
    {
        "id": "text/unicode",
        "kind": "text",
        "render_options": {"width": 20},
        "input": {"markup": "Hello 世界 🌍"},
    },
    {
        "id": "rule/basic",
        "kind": "rule",
        "input": {"title": "", "align": "center", "character": "─"},
    },
    {
        "id": "rule/title_left",
        "kind": "rule",
        "render_options": {"width": 30},
        "input": {"title": "Section", "align": "left", "character": "─"},
    },
    {
        "id": "panel/basic",
        "kind": "panel",
        "input": {
            "text": "Hello, World!",
            "title": "Greeting",
            "subtitle": None,
            "width": 30,
            "box": "ROUNDED",
        },
    },
    {
        "id": "panel/subtitle",
        "kind": "panel",
        "input": {
            "text": "Content",
            "title": "Title",
            "subtitle": "v1",
            "width": 30,
            "box": "SQUARE",
        },
    },
    {
        "id": "table/basic",
        "kind": "table",
        "render_options": {"width": 40},
        "input": {
            "columns": ["Name", "Age"],
            "rows": [["Alice", "30"], ["Bob", "25"]],
            "show_header": True,
            "show_lines": False,
            "title": "Users",
            "caption": None,
            "column_justifies": ["left", "right"],
        },
    },
    {
        "id": "table/lines",
        "kind": "table",
        "render_options": {"width": 40},
        "input": {
            "columns": ["A", "B"],
            "rows": [["1", "2"], ["3", "4"]],
            "show_header": True,
            "show_lines": True,
            "title": None,
            "caption": None,
            "column_justifies": ["left", "left"],
        },
    },
    {
        "id": "tree/basic",
        "kind": "tree",
        "input": {
            "label": "Root",
            "children": [
                {"label": "Child 1", "children": []},
                {
                    "label": "Child 2",
                    "children": [
                        {"label": "Leaf", "children": []},
                    ],
                },
            ],
        },
    },
    {
        "id": "progress/basic",
        "kind": "progress",
        "input": {"total": 100, "completed": 50, "width": 20},
    },
    {
        "id": "columns/basic",
        "kind": "columns",
        "input": {"items": ["One", "Two", "Three", "Four"]},
    },
    {
        "id": "padding/basic",
        "kind": "padding",
        "render_options": {"width": 12},
        "input": {"text": "Padded", "pad": [1, 2, 1, 2]},
    },
    {
        "id": "align/center",
        "kind": "align",
        "input": {"text": "Centered", "width": 20, "align": "center"},
    },
    {
        "id": "markdown/plain",
        "kind": "markdown",
        "compare_ansi": True,
        "input": {"text": "Just text"},
        "notes": "No styling; ANSI output is identical to plain.",
    },
    {
        "id": "markdown/emphasis_no_terminal",
        "kind": "markdown",
        "compare_ansi": True,
        "render_options": {"color_system": "auto", "force_terminal": False},
        "input": {"text": "This is **bold** and *italic*."},
        "notes": "force_terminal=false disables ANSI; compare_ansi=true ensures no SGR leakage.",
    },
    {
        "id": "json/basic",
        "kind": "json",
        "compare_ansi": True,
        "input": {"json": "{\"age\": 30, \"name\": \"Alice\"}"},
        "notes": "Default JSON styling is intended to match Python Rich defaults.",
    },
    {
        "id": "json/nested",
        "kind": "json",
        "compare_ansi": True,
        "input": {
            "json": "{\"items\": [{\"id\": 1, \"name\": \"A\"}, {\"id\": 2, \"name\": \"B\"}]}"
        },
        "notes": "Nested structures with default JSON styling.",
    },
    {
        "id": "syntax/basic",
        "kind": "syntax",
        "compare_ansi": False,
        "input": {"code": "fn main() { println!(\"hi\"); }", "language": "rust"},
        "notes": "Rust uses syntect; Python Rich uses Pygments, so ANSI differs. Plain comparison still guards layout.",
    },
    {
        "id": "syntax/no_terminal",
        "kind": "syntax",
        "compare_ansi": True,
        "render_options": {"color_system": "auto", "force_terminal": False},
        "input": {"code": "fn main() { println!(\"hi\"); }", "language": "rust"},
        "notes": "force_terminal=false disables ANSI; compare_ansi=true ensures no SGR leakage.",
    },
    {
        "id": "traceback/basic",
        "kind": "traceback",
        "compare_ansi": False,
        "render_options": {"width": 60},
        "input": {
            "frames": [
                {"name": "<module>", "line": 14},
                {"name": "level1", "line": 11},
                {"name": "level2", "line": 8},
                {"name": "level3", "line": 5},
            ],
            "exception_type": "ZeroDivisionError",
            "exception_message": "division by zero",
            "extra_lines": 0,
            "word_wrap": False,
            "show_locals": False,
            "indent_guides": False,
        },
        "notes": "Deterministic traceback generated from explicit frames.",
    },
    {
        "id": "terminal/no_color",
        "kind": "text",
        "render_options": {"color_system": "auto", "force_terminal": None},
        "env": {"NO_COLOR": "1", "FORCE_COLOR": "1", "TERM": "xterm-256color"},
        "input": {"markup": "[#ff8800]No Color[/]"},
        "notes": "NO_COLOR disables colors even when terminal supports them.",
    },
    {
        "id": "terminal/colorterm_truecolor",
        "kind": "text",
        "render_options": {"color_system": "auto", "force_terminal": None},
        "env": {"FORCE_COLOR": "1", "COLORTERM": "truecolor", "TERM": "xterm-256color"},
        "input": {"markup": "[#ff0000]TrueColor[/]"},
        "notes": "COLORTERM truecolor should yield 24-bit ANSI.",
    },
    {
        "id": "terminal/term_256color",
        "kind": "text",
        "render_options": {"color_system": "auto", "force_terminal": None},
        "env": {"FORCE_COLOR": "1", "TERM": "xterm-256color"},
        "input": {"markup": "[#00ff00]EightBit[/]"},
        "notes": "TERM -256color should yield 256-color ANSI.",
    },
    {
        "id": "terminal/term_16color",
        "kind": "text",
        "render_options": {"color_system": "auto", "force_terminal": None},
        "env": {"FORCE_COLOR": "1", "TERM": "xterm-16color"},
        "input": {"markup": "[#0000ff]Standard[/]"},
        "notes": "TERM -16color should yield standard ANSI colors.",
    },
    {
        "id": "terminal/term_dumb",
        "kind": "text",
        "render_options": {"color_system": "auto", "force_terminal": None},
        "env": {"FORCE_COLOR": "1", "TERM": "dumb"},
        "input": {"markup": "[#ff00ff]Dumb[/]"},
        "notes": "TERM dumb should disable color output.",
    },
]


def build_renderable(case: Dict[str, Any]):
    kind = case["kind"]
    inp = case["input"]

    if kind == "text":
        return inp["markup"]

    if kind == "rule":
        return Rule(inp.get("title", ""), characters=inp.get("character", "─"), align=inp.get("align", "center"))

    if kind == "panel":
        box_name = inp.get("box", "ROUNDED")
        box_value = getattr(box, box_name, box.ROUNDED)
        return Panel(
            inp.get("text", ""),
            title=inp.get("title"),
            subtitle=inp.get("subtitle"),
            width=inp.get("width"),
            box=box_value,
        )

    if kind == "table":
        table = Table(
            show_header=inp.get("show_header", True),
            show_lines=inp.get("show_lines", False),
            title=inp.get("title"),
            caption=inp.get("caption"),
        )
        columns = inp.get("columns", [])
        justifies = inp.get("column_justifies", ["left"] * len(columns))
        for idx, col in enumerate(columns):
            justify = justifies[idx] if idx < len(justifies) else "left"
            table.add_column(col, justify=justify)
        for row in inp.get("rows", []):
            table.add_row(*row)
        return table

    if kind == "tree":
        def build_node(node: Dict[str, Any]) -> Tree:
            tree = Tree(node.get("label", ""))
            for child in node.get("children", []):
                tree.add(build_node(child))
            return tree

        return build_node(inp)

    if kind == "progress":
        total = inp.get("total", 100)
        completed = inp.get("completed", 0)
        width = inp.get("width")
        bar = ProgressBar(total=total, completed=completed, width=width)
        return bar

    if kind == "columns":
        items = inp.get("items", [])
        return Columns(items)

    if kind == "padding":
        text = inp.get("text", "")
        pad = tuple(inp.get("pad", [0, 0, 0, 0]))
        return Padding(text, pad=pad)

    if kind == "align":
        text = inp.get("text", "")
        align = inp.get("align", "left")
        width = inp.get("width", None)
        return Align(text, align=align, width=width)

    if kind == "markdown":
        text = inp.get("text", "")
        return Markdown(text)

    if kind == "json":
        json_text = inp.get("json", "{}")
        return JSON(json_text)

    if kind == "syntax":
        code = inp.get("code", "")
        language = inp.get("language", "rust")
        return Syntax(code, language)

    if kind == "traceback":
        width = case.get("render_options", {}).get("width", DEFAULTS["width"])
        extra_lines = inp.get("extra_lines", 0)
        word_wrap = inp.get("word_wrap", False)
        show_locals = inp.get("show_locals", False)
        indent_guides = inp.get("indent_guides", False)
        frames = [
            Frame(
                filename="<traceback_fixture>",
                lineno=int(frame.get("line", 0)),
                name=str(frame.get("name", "")),
            )
            for frame in inp.get("frames", [])
        ]
        exception_type = inp.get("exception_type", "Error")
        exception_message = inp.get("exception_message", "")
        trace = Trace(
            stacks=[
                Stack(
                    exc_type=str(exception_type),
                    exc_value=str(exception_message),
                    frames=frames,
                )
            ]
        )
        return Traceback(
            trace=trace,
            width=width,
            extra_lines=extra_lines,
            word_wrap=word_wrap,
            show_locals=show_locals,
            indent_guides=indent_guides,
        )

    raise ValueError(f"Unknown kind: {kind}")


def merge_render_options(case: Dict[str, Any]) -> Dict[str, Any]:
    options = dict(DEFAULTS)
    overrides = case.get("render_options", {})
    options.update(overrides)
    return options


def build_env(case: Dict[str, Any]) -> Dict[str, str]:
    env = dict(DEFAULT_ENV)
    overrides = case.get("env", {})
    for key, value in overrides.items():
        if value is None:
            env.pop(key, None)
        else:
            env[key] = str(value)
    return env


def normalize_line_endings(text: str) -> str:
    return text.replace("\r\n", "\n").replace("\r", "\n")


def normalize_hyperlink_ids(text: str) -> str:
    # Python Rich may emit random OSC 8 link ids. Strip them for determinism.
    return re.sub(r"\x1b]8;id=[^;]*;", "\x1b]8;;", text)


def render_case(case: Dict[str, Any]) -> Dict[str, str]:
    options = merge_render_options(case)
    env = build_env(case)
    theme_config = case.get("theme")
    theme = None
    if theme_config:
        styles = theme_config.get("styles", {})
        inherit = theme_config.get("inherit", True)
        theme = Theme(styles, inherit=inherit)
    console = Console(
        record=True,
        width=options.get("width"),
        color_system=options.get("color_system"),
        force_terminal=options.get("force_terminal"),
        force_jupyter=False,
        theme=theme,
        legacy_windows=False,
        safe_box=True,
        emoji=True,
        markup=True,
        _environ=env,
    )
    renderable = build_renderable(case)
    console.print(renderable)
    plain = console.export_text(styles=False, clear=False)
    ansi = console._render_buffer(console._record_buffer)  # type: ignore[attr-defined]
    console._record_buffer.clear()  # type: ignore[attr-defined]
    return {
        "plain": normalize_line_endings(plain),
        "ansi": normalize_line_endings(normalize_hyperlink_ids(ansi)),
    }


def main() -> int:
    if LEGACY_RICH.exists():
        rich_version = "legacy"
    else:
        rich_version = getattr(rich, "__version__", None)
        if not rich_version:
            try:
                from importlib import metadata

                rich_version = metadata.version("rich")
            except Exception:  # pragma: no cover - metadata fallback
                rich_version = "unknown"
    output = {
        "rich_version": rich_version,
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "defaults": dict(DEFAULTS),
        "cases": [],
    }

    for case in CASES:
        rendered = render_case(case)
        output_case = {
            "id": case["id"],
            "kind": case["kind"],
            "compare_ansi": case.get("compare_ansi", True),
            "render_options": case.get("render_options"),
            "env": case.get("env"),
            "theme": case.get("theme"),
            "input": case["input"],
            "expected": rendered,
            "notes": case.get("notes"),
        }
        output["cases"].append(output_case)

    fixtures_path = ROOT / "tests" / "conformance" / "fixtures" / "python_rich.json"
    fixtures_path.write_text(
        json.dumps(output, indent=2, ensure_ascii=False) + "\n",
        encoding="utf-8",
    )
    print(f"Wrote fixtures: {fixtures_path}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
