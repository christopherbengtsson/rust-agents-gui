"""Enumerate current normative scope, ownership, delivery and ticket requirements.

Writes only the inventory; run build-matrix.py next. Environment stays frozen.
"""
from pathlib import Path
import json,re
out=Path(__file__).parent
def write(name,value):
    (out/name).write_text(json.dumps(value,indent=2)+"\n")
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
        relevant=(source.name=='spec.md' and section in ['1. Purpose, scope, and publication','2. Ownership and module structure','3. Push observations, Query, Zustand and XState','4. Routes, Settings, and development tooling','5. Delivery sequence and qualification']) or (source.name!='spec.md' and section in ['Acceptance','Verification and acceptance','Required scenarios','Verification','Deliverables','Context and implementation','Module and caller interface','Implementation and removal'])
        relevant = relevant or (source.name.startswith('01-') and line.startswith('- [x]'))
        if footholds or not relevant or line.startswith('#') or not line.strip():flush();continue
        if line.startswith('|') and ('---' in line or '| Responsibility' in line or '| Maximum active' in line or '| Module ' in line or '| Lane ' in line or '| Ticket ' in line):continue
        if re.match(r'^(?:- |\d+\. |\|)',line):flush()
        if not paragraph:start=number
        paragraph.append(line.strip())
    flush()
write('acceptance-matrix.json',requirements)
print(json.dumps({'requirements':len(requirements)}))
