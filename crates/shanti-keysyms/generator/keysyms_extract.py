#!/usr/bin/env python3
"""Regenerate the IBus keysym value table and its Rust lookup array.

Reads names and riti virtual keys from data/keysyms.tsv, verifies every name
against IBus headers, then writes the resolved numeric keysym values back.
Run from anywhere inside the repository checkout:

    ./generator/keysyms_extract.py
    ./generator/keysyms_extract.py --diff
"""

import argparse
import os
import subprocess
import tempfile

CRATE_DIR = os.path.dirname(os.path.abspath(__file__)) + "/.."

HEADER = """\
# Single source of IBus keysym mapping (phase-2).
# Probed against the real IBus enums by generator/keysyms_extract.py.
# columns: keysym_value(X keysyms, decimal)\tkeysym_name\triti_virtual_key
"""


def sh(command, fail_message=None):
    result = subprocess.run(command, shell=True, capture_output=True, text=True)
    if result.returncode != 0:
        raise SystemExit(f"{fail_message or 'command failed'}: {command}\n{result.stderr}")
    return result.stdout.strip()


def read_mapping():
    rows = []
    with open(CRATE_DIR + "/data/keysyms.tsv") as source:
        for line in source:
            line = line.strip()
            if not line or line.startswith("#"):
                continue
            _value, name, virtual_key = line.split("\t")
            rows.append((name, virtual_key))
    if len({name for name, _ in rows}) != len(rows):
        raise SystemExit("duplicate keysym name in data/keysyms.tsv")
    return rows


def probe_keysym_values(pairs):
    names = [name for name, _virtual_key in pairs]
    printf_lines = "\n".join(
        f'  printf("%s %d\\n", "{name}", IBUS_KEY_{name});' for name in names
    )
    source = "#include <ibus.h>\n#include <cstdio>\nint main() {\n" + printf_lines + "\n}\n"
    cflags = sh("pkg-config --cflags ibus-1.0", "IBus dev headers not found")
    with tempfile.TemporaryDirectory(prefix="shanti-keysyms-gen-") as workdir:
        source_path = os.path.join(workdir, "probe.cpp")
        binary_path = os.path.join(workdir, "probe")
        output_path = os.path.join(workdir, "values.txt")
        with open(source_path, "w") as output:
            output.write(source)
        sh(f"g++ {source_path} {cflags} -o {binary_path}", "probe compilation failed")
        sh(f"{binary_path} > {output_path}", "probe run failed")
        values = {}
        with open(output_path) as output:
            for line in output:
                name, value = line.split()
                values[name] = int(value)
        if sorted(values) != sorted(names):
            raise SystemExit("IBus probe returned an unexpected keysym name set")
    return values


def build_rows():
    pairs = read_mapping()
    values = probe_keysym_values(pairs)
    virtual_keys = dict(pairs)
    rows = sorted((value, name, virtual_keys[name]) for name, value in values.items())
    if len({row[0] for row in rows}) != len(rows):
        raise SystemExit("duplicate keysym VALUE; lookup by value would be ambiguous")
    return rows


def write_outputs(rows):
    with open(CRATE_DIR + "/data/keysyms.tsv", "w") as output:
        output.write(HEADER)
        for value, name, virtual_key in rows:
            output.write(f"{value}\t{name}\t{virtual_key}\n")
    data_rows = "\n".join(f'    ({value}, "{virtual_key}"),' for value, _name, virtual_key in rows)
    with open(CRATE_DIR + "/src/data.rs", "w") as output:
        output.write(
            "// GENERATED from data/keysyms.tsv by generator/keysyms_extract.py - do not edit.\n"
            "// Sorted by IBus keysym value (X keysym).\n\n"
            "pub static KEYSYMS: [(u32, &str); %d] = [\n%s\n];\n"
            % (len(rows), data_rows)
        )


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--diff", action="store_true", help="report which outputs changed")
    parser.parse_args()

    paths = (CRATE_DIR + "/data/keysyms.tsv", CRATE_DIR + "/src/data.rs")
    before = {path: (open(path).read() if os.path.exists(path) else None) for path in paths}
    rows = build_rows()
    write_outputs(rows)
    print(f"OK: {len(rows)} IBus keysyms resolved.")
    for path in paths:
        if open(path).read() != before[path]:
            print(f"changed: {os.path.relpath(path, os.path.abspath(CRATE_DIR + '/../..'))}")


if __name__ == "__main__":
    main()
