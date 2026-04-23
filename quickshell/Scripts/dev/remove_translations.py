#!/usr/bin/env python3
"""
remove_translations.py

Scan the repository for uses of I18n.tr and I18n.trp and replace them
with English literal strings from `Assets/Translations/en.json`.

Usage:
  ./remove_translations.py [--dry-run] [--apply] [--remove-files]

Options:
  --dry-run       Print changes that would be made (default)
  --apply         Apply changes in-place (creates backups)
  --backup-dir    Directory to store backups (default: .remove_translations_backup)
  --root          Repo root (default: project root / cwd)
  --remove-files  Also remove `Common/I18n.qml` and non-English translation JSON files

This script is conservative and aims to handle common call patterns like:
  I18n.tr("common.save")
  I18n.tr("actions.close-app", {app: appName})
  I18n.trp("items.count", count, {name: username})

It will create readable JS/QML replacements using string concatenation
and ternary expressions for plural calls.
"""

from __future__ import annotations

import argparse
import json
import os
import re
import shutil
from pathlib import Path
from typing import Dict, Tuple, Optional


def load_en_translations(en_path: Path) -> Dict[str, str]:
    with en_path.open("r", encoding="utf-8") as f:
        data = json.load(f)

    # Flatten nested dict into dot-separated keys
    def walk(obj, prefix=""):
        out = {}
        for k, v in obj.items():
            key = f"{prefix}.{k}" if prefix else k
            if isinstance(v, dict):
                out.update(walk(v, key))
            else:
                out[key] = str(v)
        return out

    return walk(data)


def escape_js_string(s: str) -> str:
    # Escape backslashes and double quotes and newlines
    s = s.replace("\\", "\\\\")
    s = s.replace('"', '\\"')
    s = s.replace("\n", "\\n")
    return s


def build_replacement_from_value(value: str, interp_map: Dict[str, str]) -> str:
    # value e.g. "Hello {name}, welcome"
    # interp_map maps placeholder -> expression (JS)
    parts = re.split(r"(\{[^}]+\})", value)
    expr_parts = []
    for part in parts:
        if not part:
            continue
        m = re.fullmatch(r"\{([^}]+)\}", part)
        if m:
            name = m.group(1)
            if name in interp_map:
                expr_parts.append(f"({interp_map[name]})")
            else:
                # leave placeholder text if not provided
                expr_parts.append(f'"{{{name}}}"')
        else:
            expr_parts.append(f'"{escape_js_string(part)}"')

    # Join with +, but collapse consecutive string literals
    merged: list[str] = []
    for p in expr_parts:
        if not merged:
            merged.append(p)
            continue
        # If both last and current are string literals, merge
        if merged[-1].startswith('"') and p.startswith('"'):
            a = merged.pop()
            # strip surrounding quotes
            a_val = a[1:-1]
            b_val = p[1:-1]
            merged.append(f'"{a_val + b_val}"')
        else:
            merged.append(p)

    if not merged:
        return '""'
    if len(merged) == 1:
        return merged[0]
    return " + ".join(merged)


def parse_interp_object(text: str) -> Dict[str, str]:
    """Very small parser for a JS object literal with simple values.
    Accepts forms like: {app: appName, name: "x"}
    Returns dict of key -> expression string.
    If parsing fails, returns empty dict.
    """
    out = {}
    try:
        inner = text.strip()
        if inner.startswith("{") and inner.endswith("}"):
            inner = inner[1:-1].strip()
        else:
            return out
        if not inner:
            return out
        # Split top-level commas (this is simplistic but sufficient for common cases)
        parts = re.split(r",(?![^{}]*\})", inner)
        for p in parts:
            if ':' not in p:
                continue
            k, v = p.split(':', 1)
            key = k.strip()
            val = v.strip()
            out[key] = val
    except Exception:
        return {}
    return out


TR_REGEX = re.compile(
    r"I18n\.tr\(\s*([\"'])(?P<key>.+?)\1\s*(,\s*(?P<interp>\{.*?\}|[^)]*?))?\)",
    re.DOTALL,
)

TRP_REGEX = re.compile(
    r"I18n\.trp\(\s*([\"'])(?P<key>.+?)\1\s*,\s*(?P<count>[^,\)]+)\s*(,\s*(?P<interp>\{.*?\}))?\)",
    re.DOTALL,
)


def replace_in_text(text: str, en_map: Dict[str, str]) -> Tuple[str, int]:
    changes = 0

    def tr_repl(m: re.Match) -> str:
        nonlocal changes
        key = m.group('key')
        interp_raw = m.group('interp')
        interp_map = parse_interp_object(interp_raw) if interp_raw else {}
        if key not in en_map:
            # no mapping -> leave unchanged
            return m.group(0)
        value = en_map[key]
        replacement = build_replacement_from_value(value, interp_map)
        changes += 1
        return replacement

    def trp_repl(m: re.Match) -> str:
        nonlocal changes
        key = m.group('key')
        count_expr = m.group('count').strip()
        interp_raw = m.group('interp')
        interp_map = parse_interp_object(interp_raw) if interp_raw else {}

        singular = en_map.get(key)
        plural = en_map.get(f"{key}-plural")
        if singular is None and plural is None:
            return m.group(0)
        if plural is None:
            # fallback to singular for both cases
            singular_expr = build_replacement_from_value(singular or "", interp_map)
            changes += 1
            return singular_expr
        singular_expr = build_replacement_from_value(singular or "", interp_map)
        plural_expr = build_replacement_from_value(plural or "", interp_map)
        changes += 1
        return f"(({count_expr}) == 1 ? {singular_expr} : {plural_expr})"

    # First replace trp (plural) then tr
    new_text = TRP_REGEX.sub(trp_repl, text)
    new_text = TR_REGEX.sub(tr_repl, new_text)
    return new_text, changes


def find_target_files(root: Path) -> list[Path]:
    exts = {'.qml', '.js', '.jsx', '.ts', '.tsx'}
    matches: list[Path] = []
    for p in root.rglob("*"):
        if p.is_file() and p.suffix in exts:
            # quick check for I18n usage
            try:
                txt = p.read_text(encoding='utf-8')
            except Exception:
                continue
            if 'I18n.tr(' in txt or 'I18n.trp(' in txt:
                matches.append(p)
    return matches


def backup_file(path: Path, backup_root: Path) -> None:
    dest = backup_root / path.relative_to(Path.cwd())
    dest.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(path, dest)


def main() -> None:
    ap = argparse.ArgumentParser(description="Replace I18n.tr/trp calls with English literals")
    ap.add_argument('--dry-run', action='store_true', default=True, dest='dry_run', help='Show changes (default)')
    ap.add_argument('--apply', action='store_true', help='Apply changes in-place')
    ap.add_argument('--backup-dir', default='.remove_translations_backup', help='Backup directory')
    ap.add_argument('--root', default='.', help='Repository root')
    ap.add_argument('--remove-files', action='store_true', help='Remove I18n.qml and non-en translation files')
    args = ap.parse_args()

    root = Path(args.root).resolve()
    en_path = root / 'Assets' / 'Translations' / 'en.json'
    if not en_path.exists():
        print(f"Error: English translation file not found at: {en_path}")
        return

    en_map = load_en_translations(en_path)
    print(f"Loaded {len(en_map)} English translation keys from {en_path}")

    targets = find_target_files(root)
    print(f"Found {len(targets)} files containing I18n.tr/trp candidates")

    total_changes = 0
    backup_root = Path(args.backup_dir).resolve()
    if args.apply:
        backup_root.mkdir(parents=True, exist_ok=True)

    for p in targets:
        try:
            txt = p.read_text(encoding='utf-8')
        except Exception:
            print(f"Skipping unreadable file: {p}")
            continue
        new_txt, changes = replace_in_text(txt, en_map)
        if changes > 0:
            print(f"{p}: {changes} replacement(s)")
            total_changes += changes
            if args.apply:
                backup_file(p, backup_root)
                p.write_text(new_txt, encoding='utf-8')
            else:
                # show diff-ish context
                print('--- Preview snippet ---')
                # show up to first 3 matches with context
                for i, m in enumerate(re.finditer(r"I18n\.tr\(|I18n\.trp\(", txt)):
                    start = max(0, m.start()-80)
                    end = min(len(txt), m.end()+80)
                    print(txt[start:end].replace('\n','\\n'))
                    if i >= 2:
                        break
                print('-----------------------')

    print(f"Total replacements: {total_changes}")

    if args.remove_files:
        i18n_qml = root / 'Common' / 'I18n.qml'
        translations_dir = root / 'Assets' / 'Translations'
        removed = []
        if i18n_qml.exists():
            removed.append(str(i18n_qml))
            if args.apply:
                backup_file(i18n_qml, backup_root)
                i18n_qml.unlink()
        # remove non-en json translation files
        if translations_dir.exists():
            for jf in translations_dir.glob('*.json'):
                if jf.name == 'en.json':
                    continue
                removed.append(str(jf))
                if args.apply:
                    backup_file(jf, backup_root)
                    jf.unlink()
        if removed:
            print('Removed files:')
            for r in removed:
                print(' -', r)
        else:
            print('No translation files to remove')

    if args.apply:
        print(f"Applied changes. Backups saved to: {backup_root}")
    else:
        print("Dry-run (no files modified). Re-run with --apply to make changes.")


if __name__ == '__main__':
    main()
