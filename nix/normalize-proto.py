#!/usr/bin/env python3

from pathlib import Path
import re
import sys


def matching_brace(source: str, opening: int) -> int:
    depth = 0
    for index in range(opening, len(source)):
        character = source[index]
        if character == "{":
            depth += 1
        elif character == "}":
            depth -= 1
            if depth == 0:
                return index
    raise ValueError("unclosed generated function")


def replace_words(source: str, start: int, end: int, replacements: dict[str, str]) -> str:
    body = source[start:end]
    for old, new in replacements.items():
        body = re.sub(rf"\b{old}\b", new, body)
    return source[:start] + body + source[end:]


def normalize_functions(source: str) -> str:
    functions = list(re.finditer(r"\bfn (from_reader|write_message|from)\b[^{]*\{", source))
    for function in reversed(functions):
        opening = source.find("{", function.start(), function.end())
        closing = matching_brace(source, opening)
        name = function.group(1)
        if name == "from_reader":
            replacements = {"t": "tag", "e": "error"}
        elif name == "write_message":
            replacements = {"w": "writer", "s": "value"}
        elif "i32" in function.group(0):
            replacements = {"i": "value"}
        else:
            replacements = {"s": "value"}
        source = replace_words(source, function.start(), closing + 1, replacements)

    lines = source.splitlines(keepends=True)
    for index, line in enumerate(lines):
        if "|m|" in line or "|s|" in line:
            line = re.sub(r"\bm\b", "value", line)
            line = re.sub(r"\bs\b", "value", line)
            lines[index] = line
    source = "".join(lines)
    source = re.sub(
        r"msg\.(\w+) = (r\.read_(?:bytes|string)\(bytes\)\?)\.to_owned\(\)([,;])",
        r"\2.clone_into(&mut msg.\1)\3",
        source,
    )
    source = re.sub(r"pub struct (\w+) \{ \}", r"pub struct \1;", source)
    for field in ("ota_installed", "stats_ok", "settings_diff", "market_ok", "timeout", "upload_stat", "adaptive_heartbeat", "use_rmq2", "from_trusted_server", "immediate_ack", "number_discarded_events"):
        source = re.sub(
            rf"self\.{field}\.as_ref\(\)\.map_or\(0, \|value\| (\d+) \+ sizeof_varint\(\*\(value\) as u64\)\)",
            rf"self.{field}.as_ref().map_or(0, |value| \1 + sizeof_varint(u64::from(*value)))",
            source,
        )
    for field in ("stats_ok", "timeout"):
        source = source.replace(
            f"sizeof_varint(*(&self.{field}) as u64)",
            f"sizeof_varint(u64::from(self.{field}))",
        )
    impls = list(re.finditer(r"impl(?:<[^>]+>)? (?:Default|From<[^>]+>) for (\w+) \{", source))
    for impl in reversed(impls):
        opening = source.find("{", impl.start(), impl.end())
        closing = matching_brace(source, opening)
        body = source[opening + 1 : closing]
        body = re.sub(rf"\b{impl.group(1)}::", "Self::", body)
        source = source[: opening + 1] + body + source[closing:]
    return source


def add_message_write_methods(source: str) -> str:
    methods = """
    fn write_file<P>(&self, p: P) -> Result<()>
    where
        P: AsRef<Path>,
    {
        let file = BufWriter::new(File::create(p)?);
        let mut writer = Writer::new(file);
        self.write_message(&mut writer)
    }
"""
    matches = list(re.finditer(r"impl MessageWrite for \w+ \{", source))
    for match in reversed(matches):
        opening = source.find("{", match.start(), match.end())
        closing = matching_brace(source, opening)
        body = source[opening + 1 : closing].strip()
        if "fn write_file" in body:
            continue
        if body:
            insertion = methods
        else:
            insertion = """
    fn write_message<W>(&self, _: &mut Writer<W>) -> Result<()>
    where
        W: WriterBackend,
    {
        Ok(())
    }

    fn get_size(&self) -> usize {
        0
    }

""" + methods
        source = source[:closing] + insertion + source[closing:]
    return source


def normalize(path: Path) -> None:
    source = path.read_text()
    source = normalize_functions(source)
    source = add_message_write_methods(source)
    source = source.replace(
        "use quick_protobuf::sizeofs::*;",
        "use quick_protobuf::sizeofs::{sizeof_len, sizeof_varint};",
    )
    for message in ("Close", "StreamAck"):
        source = source.replace(
            f"impl<'a> MessageRead<'a> for {message}",
            f"impl MessageRead<'_> for {message}",
        )
    source = re.sub(
        r"fn write_message<W: WriterBackend>([^\n]*) -> Result<\(\)> \{",
        r"fn write_message<W>\1 -> Result<()>\n    where\n        W: WriterBackend,\n    {",
        source,
    )
    if "use std::{fs::File, io::BufWriter, path::Path};" not in source:
        source = source.replace(
            "#![cfg_attr(rustfmt, rustfmt_skip)]\n",
            "#![cfg_attr(rustfmt, rustfmt_skip)]\n\nuse std::{fs::File, io::BufWriter, path::Path};\n",
            1,
        )
    source = source.replace(
        "fn write_message<W>(&self, writer: &mut Writer<W>) -> Result<()> where W: WriterBackend",
        "fn write_message<W>(&self, writer: &mut Writer<W>) -> Result<()>\n    where\n        W: WriterBackend",
    )
    path.write_text(source)


for argument in sys.argv[1:]:
    normalize(Path(argument))
