"""Opt-in native Git/SQLite/process qualification, outside the unit gate.
Only creates disposable repositories/state and terminates its own supervisor handles.
No provider invocation. Invocation: python3 qualify_names.py <built supervisor> <output.json>
"""
import hashlib, json, os, pathlib, shutil, socket, sqlite3, subprocess, sys, tempfile, time, uuid
P = pathlib.Path
binary = P(sys.argv[1]).resolve()
output = P(sys.argv[2])
root = P(tempfile.mkdtemp(prefix="rag-names-", dir="/private/tmp"))
state = root / "state"
state.mkdir(mode=0o700)
players = (P(__file__).parents[2] / "research/footballer-catalog.md").read_text().split("```text\n")[1].split("```")[0].splitlines()
assert len(players) == 500
owned = []
result = {"root": str(root), "binary": str(binary), "sha256": hashlib.sha256(binary.read_bytes()).hexdigest()}
def git(repo, *args, input=None):
    return subprocess.check_output(["/usr/bin/git", "-C", str(repo), *args], input=input, stderr=subprocess.STDOUT, text=True).strip()
def exchange(field=None, body=None, directory=state):
    payload = {"protocol":4,"application_version":"0.1.0","after":None}
    if field: payload[field] = body
    with socket.socket(socket.AF_UNIX) as s:
        s.settimeout(3);s.connect(str(directory / "supervisor.sock"));s.sendall((json.dumps(payload)+"\n").encode())
        return json.loads(s.makefile("rb").readline())
def request(field, body, directory=state):
    response = exchange(field,body,directory)["result"]
    assert "Ok" in response, response
    return response["Ok"]
def wait(read, done, seconds=8):
    deadline=time.monotonic()+seconds
    while time.monotonic()<deadline:
        value=read()
        if done(value): return value
        time.sleep(.01)
    raise AssertionError(value)
def launch(directory=state):
    log=(root/f"owner-{len(owned)}.log").open("w")
    proc=subprocess.Popen([str(binary),str(directory)],stdout=log,stderr=log);owned.append(proc)
    wait(lambda: read_snapshot(directory),lambda x:x is not None)
    return proc
def read_snapshot(directory):
    try: return exchange(directory=directory)
    except (OSError,ValueError): return None
def make_repo(parent):
    repo=root/parent/"Readable Repo";repo.mkdir(parents=True)
    git(repo,"init","--initial-branch=main");(repo/"file.txt").write_text("base\n");git(repo,"add",".")
    git(repo,"-c","user.name=Qualification","-c","user.email=q@example.invalid","commit","-m","base")
    return repo
def register(repo, initial=True):
    aid=str(uuid.uuid4());request("repository",{"operation":"register","request":{"action_id":aid,"path":str(repo),"create_initial_workspace":initial}})
    return wait(lambda:request("repository",{"operation":"action","action_id":aid})["registration"],lambda a:a["state"]!="pending")
def get(wid,directory=state):return request("workspace",{"operation":"get","id":wid},directory)["workspace"]
def completed(wid):return wait(lambda:get(wid),lambda w:w["status"]["state"] not in ["queued","running"])
def create(rid):
    action={"operation":"create","action_id":str(uuid.uuid4()),"repository_id":rid}
    w=request("workspace",action)["workspace"]
    return completed(w["id"]),action
started=time.monotonic()
try:
    # Metadata must exit before state creation, permission changes or owner admission.
    metadata_dir=root/"metadata";metadata_dir.mkdir()
    assert subprocess.check_output([str(binary),"--protocol-version"],cwd=metadata_dir,text=True)=="4\n"
    assert list(metadata_dir.iterdir())==[]
    result["metadata_no_state_effects"]=True
    repo=make_repo("one");base=git(repo,"rev-parse","HEAD")
    (repo/"file.txt").write_text("dirty original\n");before=(repo/"file.txt").read_bytes()
    # One bounded Git transaction occupies every plain catalog name.
    git(repo,"update-ref","--stdin",input="start\n"+"".join(f"create refs/heads/workspace/{name} {base}\n" for name in players)+"prepare\ncommit\n")
    (state/"workspaces/readable-repo").mkdir(parents=True,mode=0o700)
    (state/"workspaces").chmod(0o700)
    owner=launch();registration=register(repo);rid=registration["repository_id"]
    first=completed(registration["initial_workspace_id"])
    assert first["status"]["state"]=="ready",first
    assert first["name"] in [name+"-2" for name in players]
    directory=state/"workspaces/readable-repo-2"
    assert first["path"]==str(directory/first["name"]/"checkout")
    assert first["branch"]=="workspace/"+first["name"]
    # All tier-2 folders now exist; pick tier 3 without touching any foreign folder.
    for name in players:(directory/(name+"-2")).mkdir(exist_ok=True,mode=0o700)
    second,_=create(rid)
    assert second["status"]["state"]=="ready" and second["name"] in [n+"-3" for n in players],second
    # Reserve names permanently even on ambiguous-base failure.
    git(repo,"branch","master")
    failed,action=create(rid);other,_=create(rid)
    assert failed["status"]["state"]==other["status"]["state"]=="failed"
    assert failed["name"]!=other["name"] and failed["name"]!=second["name"]
    git(repo,"branch","-D","master")
    request("workspace",{"operation":"reconcile","action_id":str(uuid.uuid4()),"workspace_id":failed["id"]})
    recovered=completed(failed["id"])
    assert recovered["status"]["state"]=="ready"
    assert all(recovered[k]==failed[k] for k in ["name","path","branch","id","creation_action_id"])
    assert request("workspace",action)["workspace"]["id"]==failed["id"]
    # Independent repositories with the same basename reserve distinct readable roots.
    clone=make_repo("two");reg2=register(clone);ws2=completed(reg2["initial_workspace_id"])
    assert ws2["status"]["state"]=="ready" and P(ws2["path"]).parents[1].name=="readable-repo-3",ws2
    # A missing worktree registration at the next repository root must fail quickly.
    ancestor_repo=make_repo("three");foreign=state/"workspaces/readable-repo-4"
    git(ancestor_repo,"worktree","add","-b","foreign-root",str(foreign));shutil.rmtree(foreign)
    ancestor=register(ancestor_repo);start=time.monotonic();blocked=completed(ancestor["initial_workspace_id"])
    assert blocked["status"]["state"]=="failed" and blocked["status"]["error"]["code"]=="conflict" and blocked["path"] is None,blocked
    assert time.monotonic()-start<2
    # SQLite itself rejects reservation collisions, independent of the owner policy.
    db=sqlite3.connect(state/"workspace.sqlite3")
    assert db.execute("PRAGMA user_version").fetchone()[0]==3
    constraint_results=[]
    for sql,params in [
        ("UPDATE workspaces SET name=? WHERE id=?",(first["name"],second["id"])),
        ("UPDATE workspaces SET branch=? WHERE id=?",(first["branch"],second["id"])),
        ("UPDATE workspaces SET path=? WHERE id=?",(first["path"],second["id"])),
        ("UPDATE repository_folders SET folder=? WHERE repository_id=?",("readable-repo-2",reg2["repository_id"]))]:
        try:db.execute(sql,params);raise AssertionError("constraint accepted collision")
        except sqlite3.IntegrityError:constraint_results.append(True)
        finally:db.rollback()
    db.close()
    snapshot=exchange();owner.terminate();owner.wait(timeout=5);owner=launch();restored=get(first["id"])
    assert all(first[k]==restored[k] for k in ["id","name","branch","path","base"])
    assert exchange()["snapshot"]["cursor"]["owner"]!=snapshot["snapshot"]["cursor"]["owner"]
    assert (repo/"file.txt").read_bytes()==before
    # Build schema-2 metadata in separate private state; actual legacy refs/folders stay put.
    migration=root/"migration";migration.mkdir(mode=0o700)
    db=sqlite3.connect(migration/"workspace.sqlite3");os.chmod(migration/"workspace.sqlite3",0o600)
    db.executescript("CREATE TABLE repositories(id TEXT PRIMARY KEY,common_dir TEXT UNIQUE,record TEXT); CREATE TABLE registration_actions(id TEXT PRIMARY KEY,pending INTEGER,record TEXT); CREATE TABLE workspaces(id TEXT PRIMARY KEY,repository_id TEXT,queued INTEGER,record TEXT); CREATE TABLE workspace_actions(id TEXT PRIMARY KEY,workspace_id TEXT,record TEXT); PRAGMA user_version=2;")
    mrid=str(uuid.uuid4());record={"id":mrid,"name":"Readable Repo","path":str(repo),"common_dir":git(repo,"rev-parse","--path-format=absolute","--git-common-dir")}
    db.execute("INSERT INTO repositories VALUES(?,?,?)",(mrid,record["common_dir"],json.dumps(record)))
    old=[]
    for n in [1,2]:
        wid=str(uuid.UUID(int=n));name=f"kyoto-{wid}";path=migration/"workspaces"/mrid/name/"checkout"
        branch="workspace/"+name
        if n==1:git(repo,"worktree","add","-b",branch,str(path))
        w={**first,"id":wid,"repository_id":mrid,"creation_action_id":str(uuid.uuid4()),"name":name,"branch":branch,"path":str(path),"status":{"state":"ready" if n==1 else "failed",**({} if n==1 else {"error":{"code":"git","message":"synthetic prior failure"}})}}
        w.pop("legacy");old.append(w);db.execute("INSERT INTO workspaces VALUES(?,?,?,?)",(wid,mrid,0,json.dumps(w)))
    db.commit();db.close();mowner=launch(migration)
    migrated=[get(w["id"],migration) for w in old]
    assert [w["name"] for w in migrated]==["kyoto","kyoto-2"]
    assert all(n["legacy"] and all(n[k]==o[k] for k in ["id","branch","path","status"]) for o,n in zip(old,migrated))
    assert P(old[0]["path"]).is_dir() and git(repo,"show-ref","--verify","refs/heads/"+old[0]["branch"])
    mowner.terminate();mowner.wait(timeout=5);launch(migration)
    assert [get(w["id"],migration)["name"] for w in old]==["kyoto","kyoto-2"]
    result.update({"result":"pass","catalog_count":len(players),"first":first,"tier3":second,"failed_reserved":failed,"other_failed_reserved":other,"recovered":recovered,"second_repository":ws2,"ancestor_conflict":blocked,"storage_constraints":constraint_results,"restored":restored,"migration":migrated,"original_checkout_unchanged":True})
finally:
    for proc in owned:
        if proc.poll() is None:proc.terminate();proc.wait(timeout=5)
    result["wall_seconds"]=time.monotonic()-started;output.write_text(json.dumps(result,indent=2)+"\n")
print(json.dumps({"result":result.get("result"),"root":str(root),"wall_seconds":result["wall_seconds"]}))
