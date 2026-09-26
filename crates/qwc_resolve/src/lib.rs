mod scope;
mod local;
mod export;
mod resolve;
mod collect_scp;
mod collect_exp;
pub mod dupm_scp;
pub mod dump_exp;
pub mod imod;

pub mod import;

pub use scope::{Scope, ScopeMap, ScopeKind, ScopeKindAst, ScopeKindHir, ImportDef, ImportSegment, ImportStatus, InsertResult};
pub use export::{Export, ExportMap, ExportKind};
pub use resolve::{Resolver, LookupResult};
pub use local::{LocalVarInfo, LocalScopeManager};
pub use dump_exp as dupm_exp;
pub use import::resolve_imports;
pub use imod::Imod;
pub use collect_scp::ScopeCollector;
