#!/usr/bin/env python3
"""Validate the published archive without model calls or external dependencies."""
import ast
from collections import Counter
from html.parser import HTMLParser
import json
from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[1]

class Scripts(HTMLParser):
    def __init__(self):
        super().__init__()
        self.current = None
        self.items = {}
    def handle_starttag(self, tag, attrs):
        if tag == 'script':
            self.current = dict(attrs).get('id')
            if self.current:
                self.items[self.current] = ''
    def handle_data(self, data):
        if self.current:
            self.items[self.current] += data
    def handle_endtag(self, tag):
        if tag == 'script':
            self.current = None

def embedded(html, key):
    parser = Scripts()
    parser.feed(html)
    return json.loads(parser.items[key])

def main():
    data = json.loads((ROOT / 'results.json').read_text())
    bundle = embedded((ROOT / 'index.html').read_text(), 'embedded')
    assert bundle['results'] == data, 'HTML download and standalone results differ'
    chat = embedded(bundle['chat'], 'data')
    assert chat == data, 'Chat and standalone results differ'
    records = data['records']
    assert len({r['id'] for r in records}) == 81
    assert Counter(r['batch'] for r in records) == {
        'claude': 12, 'sol': 40, 'small': 2, 'grok': 6,
        'diag1': 5, 'diag2': 8, 'probe': 8,
    }
    assert len(list((ROOT / 'cases').glob('*.json'))) == 12
    assert {r['arm'] for r in records if r['batch'] == 'grok'} == {
        'base', 'v1', 'v2', 'v3', 'v4', 'v3b'
    }
    for version in ('v1', 'v2', 'v3', 'v4', 'v3b'):
        assert (ROOT / 'prompts' / f'{version}.txt').read_text().strip() == data['prompts'][version].strip()
    for path in ROOT.rglob('*'):
        if path.is_file() and '__pycache__' not in path.parts:
            text = path.read_text()
            assert not re.search(r'/Users/[a-zA-Z0-9_-]+/', text), f'Local home path: {path}'
            if path.suffix == '.json':
                json.loads(text)
            elif path.suffix == '.py':
                ast.parse(text, filename=str(path))
    print('OK: 81 records; 12 cases; all six Grok arms; matching HTML/JSON/prompts; valid JSON/Python.')

if __name__ == '__main__':
    main()
