"""Verify final source and packaged executable identities before committing F06."""
from pathlib import Path
import hashlib,json
out=Path(__file__).parent
manifest=json.loads((out/'source-manifest.json').read_text())
for name,expected in manifest['files'].items():
    assert hashlib.sha256(Path(name).read_bytes()).hexdigest()==expected,name
context=json.loads((out/'desktop-context.json').read_text())
source=Path('target/release/bundle/macos/Agent Workspace.app/Contents/MacOS/workspace-desktop')
assert hashlib.sha256(source.read_bytes()).hexdigest()==context['desktop_sha256']
assert hashlib.sha256((Path(context['bundle'])/'Contents/MacOS/workspace-desktop').read_bytes()).hexdigest()==context['desktop_sha256']
provenance=json.loads((out/'sidecar-provenance.json').read_text())
assert provenance['sourceRevision']==manifest['current_predecessor']
assert hashlib.sha256(Path('src-tauri/gen/sidecar/workspace-supervisor').read_bytes()).hexdigest()==provenance['sha256']
print(json.dumps({'source_files_verified':len(manifest['files']),'final_fixture_matches_package':True,'sidecar_matches_provenance':True}))
