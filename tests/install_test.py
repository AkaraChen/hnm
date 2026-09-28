"""Installer contracts with local release fixtures and a real native binary."""
import hashlib
import io
import os
from pathlib import Path
import shutil
import subprocess
import sys
import tarfile
import tempfile
import unittest

ROOT = Path(__file__).resolve().parents[1]
BINARY = Path(sys.argv.pop(1)).resolve()
VERSION = subprocess.check_output([BINARY, '--version'], text=True).strip().split()[1]


class InstallerTest(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory()
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.mock = self.root / 'mock'
        self.mock.mkdir()
        self.dest = self.root / 'bin with spaces'
        self.dest.mkdir()
        self.old = self.dest / 'hnm'
        self.old.write_text('old binary')
        self.fixture = self.root / 'release.tar.gz'
        with tarfile.open(self.fixture, 'w:gz') as archive:
            archive.add(BINARY, arcname='hnm')
        self.digest = hashlib.sha256(self.fixture.read_bytes()).hexdigest()
        self.env = dict(os.environ, PATH=f'{self.mock}:{os.environ["PATH"]}',
                        HNM_VERSION=f'v{VERSION}', HNM_INSTALL_DIR=str(self.dest),
                        FIXTURE=str(self.fixture), DIGEST=self.digest,
                        TEST_OS='Linux', TEST_ARCH='x86_64')
        self.script('uname', '#!/bin/sh\ncase "$1" in -s) echo "$TEST_OS";; -m) echo "$TEST_ARCH";; esac\n')
        self.script('curl', '''#!/usr/bin/env python3
import os, pathlib, shutil, sys
args=sys.argv[1:]
url=next(a for a in args if a.startswith('https://'))
mode=os.environ.get('MODE','')
if mode == 'network' or (mode == 'missing' and '/download/' in url): sys.exit(22)
if url.endswith('/latest'):
    print('https://github.com/AkaraChen/hnm/releases/tag/'+os.environ['LATEST'], end='')
    sys.exit(0)
name=url.rsplit('/',1)[1]
expected='hnm-'+os.environ.get('EXPECTED_VERSION',os.environ['HNM_VERSION'])+'-'+os.environ.get('EXPECTED_TARGET','x86_64-unknown-linux-gnu')+'.tar.gz'
assert name in (expected, expected+'.sha256'), name
out=pathlib.Path(args[args.index('-o')+1])
if name.endswith('.sha256'):
    out.write_text(('0'*64 if mode=='checksum' else os.environ['DIGEST'])+'  '+expected+'\\n')
else: shutil.copyfile(os.environ['FIXTURE'],out)
''')

    def script(self, name, content):
        p = self.mock / name
        p.write_text(content)
        p.chmod(0o755)

    def run_installer(self, success=True):
        result = subprocess.run(['sh', str(ROOT / 'install.sh')], env=self.env,
                                text=True, capture_output=True)
        if success:
            self.assertEqual(result.returncode, 0, result.stderr)
            self.assertIn('Installed hnm', result.stdout)
        else:
            self.assertNotEqual(result.returncode, 0, result.stdout)
            self.assertNotIn('Installed hnm', result.stdout)
            self.assertEqual(self.old.read_text(), 'old binary')
        return result

    def test_clean_install_repeat_and_init(self):
        self.old.unlink()
        self.run_installer()
        self.run_installer()
        self.assertEqual(subprocess.check_output([self.old, '--version'], text=True).strip(), f'hnm {VERSION}')
        project = self.root / 'project'
        subprocess.run([self.old, 'init', str(project), '--stack', 'rust'], check=True, stdout=subprocess.DEVNULL)
        self.assertTrue((project / 'CLAUDE.md').is_symlink())
        (project / 'AGENTS.md').write_text('user config')
        self.run_installer()
        self.assertEqual((project / 'AGENTS.md').read_text(), 'user config')
        subprocess.run([self.old, 'init', str(self.root / 'dry'), '--dry-run'], check=True, stdout=subprocess.DEVNULL)
        self.assertFalse((self.root / 'dry').exists())

    def test_platform_mapping(self):
        for system, machine, target in [
            ('Linux', 'x86_64', 'x86_64-unknown-linux-gnu'),
            ('Linux', 'aarch64', 'aarch64-unknown-linux-gnu'),
            ('Darwin', 'x86_64', 'x86_64-apple-darwin'),
            ('Darwin', 'arm64', 'aarch64-apple-darwin')]:
            with self.subTest(target=target):
                self.env.update(TEST_OS=system, TEST_ARCH=machine, EXPECTED_TARGET=target)
                self.run_installer()

    def test_latest_resolves_once(self):
        self.env.update(HNM_VERSION='latest', LATEST=f'v{VERSION}', EXPECTED_VERSION=f'v{VERSION}')
        self.run_installer()

    def test_failure_preserves_old_binary(self):
        for mode in ['network', 'missing', 'checksum']:
            with self.subTest(mode=mode):
                self.env['MODE'] = mode
                self.run_installer(False)

    def test_unsupported_os_and_arch(self):
        self.env['TEST_OS'] = 'Windows_NT'
        self.run_installer(False)
        self.env.update(TEST_OS='Linux', TEST_ARCH='riscv64')
        self.run_installer(False)

    def test_invalid_and_unavailable_version(self):
        self.env['HNM_VERSION'] = '../invalid'
        self.run_installer(False)
        self.env.update(HNM_VERSION='v99.0.0', MODE='missing')
        self.run_installer(False)

    def test_version_mismatch(self):
        self.env['HNM_VERSION'] = 'v99.0.0'
        self.run_installer(False)

    def test_bad_archive(self):
        self.fixture.write_bytes(b'not an archive')
        self.env['DIGEST'] = hashlib.sha256(self.fixture.read_bytes()).hexdigest()
        self.run_installer(False)

    def test_relative_destination(self):
        self.env['HNM_INSTALL_DIR'] = 'relative/bin'
        self.run_installer(False)

    def test_existing_symlink_target_untouched(self):
        target = self.root / 'original'
        self.old.rename(target)
        self.old.symlink_to(target)
        self.run_installer()
        self.assertFalse(self.old.is_symlink())
        self.assertEqual(target.read_text(), 'old binary')

    def test_default_destination(self):
        self.env.pop('HNM_INSTALL_DIR')
        self.env['HOME'] = str(self.root / 'home')
        result = self.run_installer()
        self.assertTrue((self.root / 'home/.local/bin/hnm').is_file())
        self.assertIn('PATH', result.stdout)


if __name__ == '__main__':
    unittest.main()
