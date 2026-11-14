"""Freeze F08 environment and enumerate normative source paragraphs for audit."""
from pathlib import Path
import hashlib,json,platform,re,subprocess
out=Path(__file__).parent

def command(args):
    return subprocess.check_output(args,text=True).strip()

def write(name,value):
    (out/name).write_text(json.dumps(value,indent=2)+'\n')

write('environment.json',{
    'starting_revision':command(['git','rev-parse','HEAD']),
    'branch':command(['git','branch','--show-current']),
    'platform':platform.platform(),
    'versions':{key:command(args) for key,args in {
        'node':['node','--version'],'pnpm':['corepack','pnpm','--version'],
        'rustc':['rustc','--version'],'cargo':['cargo','--version'],
        'macos':['sw_vers']}.items()},
    'hardware':{key:command(['sysctl','-n',arg]) for key,arg in {
        'model':'hw.model','memory_bytes':'hw.memsize','logical_cpus':'hw.logicalcpu',
        'cpu':'machdep.cpu.brand_string'}.items()},
    'target':str(Path('target').resolve()),
    'cache':'warm worktree target; isolated private targets for baseline reconstruction',
    'unit_profile':'test + codegen, Rust --test-threads=1; Node --test-concurrency=1; Vitest --maxWorkers=1',
    'packaging_profile':'release app; any inspector-enabled build separately identified',
    'initial_tree_and_index':'clean before F08 evidence creation',
    'preexisting_fixture_owners':'F08-labelled /private/tmp/aw-f08-desktop-vxqzwtcr and unrelated owners predate this execution; not reused or signalled',
    'dependencies':json.loads(Path('package.json').read_text())['dependencies']})
requirements=[]
# Every normative paragraph/bullet/table row, retaining exact source locations.
sources=[Path('.scratch/frontend-navigation/spec.md'),*sorted(Path('.scratch/frontend-navigation/issues').glob('*.md'))]
for source in sources:
    if source.name.startswith('09'): continue
    lines=source.read_text().splitlines(); section=''; subsection=''; paragraph=[]; start=0; footholds=False
    def flush():
        global paragraph
        if paragraph:
            text=' '.join(paragraph)
            requirements.append({'id':f'R{len(requirements)+1:03}', 'source':str(source),
                'line':start,'section':section,'requirement':text,'status':'not-run','evidence':[]})
            paragraph=[]
    for number,line in enumerate(lines,1):
        if line.startswith('## '):
            flush();section=line[3:];footholds=False
        if line.startswith('### '):flush();subsection=line[4:]
        if 'source footholds:' in line or line.startswith('Source footholds:'):flush();footholds=True
        if section=='Comments':break
        relevant=(source.name=='spec.md' and section in ['2. Ownership and module structure','3. Push observations, Query, Zustand and XState','4. Routes, Settings, and development tooling','5. Delivery sequence and qualification']) or (source.name!='spec.md' and section in ['Acceptance','Verification and acceptance','Required scenarios','Verification','Deliverables','Context and implementation','Module and caller interface','Implementation and removal'])
        relevant = relevant or (source.name.startswith('01-') and line.startswith('- [x]'))
        if source.name=='spec.md' and section=='5. Delivery sequence and qualification':relevant=subsection=='Verification and performance'
        if footholds or not relevant or line.startswith('#') or not line.strip():flush();continue
        if line.startswith('|') and ('---' in line or '| Responsibility' in line or '| Maximum active' in line or '| Module ' in line or '| Lane ' in line):continue
        if re.match(r'^(?:- |\d+\. |\|)',line):flush()
        if not paragraph:start=number
        paragraph.append(line.strip())
    flush()
write('acceptance-matrix.json',requirements)
print(json.dumps({'requirements':len(requirements),'environment':str(out/'environment.json')}))
