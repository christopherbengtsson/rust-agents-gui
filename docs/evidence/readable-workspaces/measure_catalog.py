"""Explicit, standalone CPU measurement; not part of the unit gate.
Includes the production naming policy/catalog with inert record/error types.
No Git, storage, supervisor, providers, IPC or UI are exercised.
"""
import hashlib,json,pathlib,subprocess,tempfile,time
P=pathlib.Path
root=P(__file__).resolve().parents[3]
naming=root/'crates/core/src/workspaces/naming.rs'
catalog=naming.parent/'naming/catalog.rs'
temporary=P(tempfile.mkdtemp(prefix='rag-naming-cpu-',dir='/private/tmp'))
policy=temporary/'naming.rs'
policy.write_text(naming.read_text().replace('mod catalog;', '#[path = \"'+str(catalog)+'\"] mod catalog;'))
source=r'''
#![allow(dead_code)]
use std::{path::{Path,PathBuf}, collections::HashSet, time::Instant};
#[derive(Debug)] enum ErrorCode { Capacity, Conflict }
#[derive(Debug)] struct RepositoryError;
impl RepositoryError { fn new(_:ErrorCode,_:impl Into<String>)->Self {Self} fn storage(_:impl std::fmt::Display)->Self {Self} }
type Result<T> = std::result::Result<T,RepositoryError>;
#[derive(Hash,Eq,PartialEq,Ord,PartialOrd,Clone,Copy)] struct RepositoryId(u128);
#[derive(Eq,PartialEq,Ord,PartialOrd,Clone,Copy)] struct WorkspaceId(u128);
struct Workspace { name:Option<String>, id:WorkspaceId, repository_id:RepositoryId, legacy:bool }
#[path = "POLICY"] mod naming;
fn main() {
 let roster:Vec<&str>=include_str!("CATALOG").lines().filter_map(|line| line.trim().strip_prefix('"').and_then(|v|v.split('"').next())).collect();
 assert_eq!(roster.len(),500);
 print!("[");
 for (scenario,count) in [0,250,495,500].iter().enumerate() {
  let used:HashSet<String>=roster.iter().take(*count).map(|v|(*v).into()).collect();
  let occupied=naming::Occupied::default(); let mut times=vec![];
  for run in 0..6 {
   let start=Instant::now();
   for seed in 0..10_000 { std::hint::black_box(naming::workspace_name(std::hint::black_box(&used), &occupied, Path::new("/private/state/repo"),seed).unwrap()); }
   let ns=start.elapsed().as_nanos() as f64/10_000.0;
   if run>0 {times.push(ns);}
  }
  times.sort_by(f64::total_cmp);
  if scenario>0 {print!(",");}
  print!("{{\"reserved_plain_names\":{},\"median_ns_per_selection\":{},\"min_ns\":{},\"max_ns\":{}}}",count,times[2],times[0],times[4]);
 }
 println!("]");
}
'''.replace('POLICY',str(policy)).replace('CATALOG',str(catalog))
(temporary/'main.rs').write_text(source)
start=time.monotonic();subprocess.run(['rustc','--edition=2024','-Copt-level=3','-Cstrip=symbols',str(temporary/'main.rs'),'-o',str(temporary/'probe')],check=True)
compile_wall=time.monotonic()-start
measurements=json.loads(subprocess.check_output([str(temporary/'probe')],text=True))
result={'source_sha256':hashlib.sha256(naming.read_bytes()).hexdigest(),'catalog_sha256':hashlib.sha256(catalog.read_bytes()).hexdigest(),'compile_wall_seconds':compile_wall,'profile':'rustc1.99.1 opt-level3 strip symbols; actual production policy/catalog; inert unused identity/error record types','warmup_runs':1,'measured_runs':5,'selections_per_run':10000,'scenarios':measurements,'harness':str(temporary/'main.rs')}
(root/'docs/evidence/readable-workspaces/catalog-performance.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))
