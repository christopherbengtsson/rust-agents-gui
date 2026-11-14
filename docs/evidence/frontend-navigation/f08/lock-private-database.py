"""Hold a disposable database lock; real reads retain physical capacity until timeout."""
from pathlib import Path
import json,sqlite3,time
out=Path(__file__).parent;c=json.loads((out/'desktop-context.json').read_text());database=Path(c['state'])/'workspace.sqlite3';assert str(database).startswith('/private/tmp/aw-f08-qualified-')
connection=sqlite3.connect(database,timeout=5);connection.execute('BEGIN EXCLUSIVE');start=time.monotonic()
try:time.sleep(40)
finally:
    connection.rollback();connection.close();r={'database':str(database),'start':start,'released':time.monotonic(),'fault':'exclusive private SQLite transaction; production 100ms busy budget unchanged'};(out/'physical-database-lock.json').write_text(json.dumps(r,indent=2)+'\n');print(json.dumps(r),flush=True)
