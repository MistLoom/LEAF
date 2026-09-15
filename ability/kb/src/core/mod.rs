//! 知识库领域契约。不依赖工具运行时门面。

mod clue;
mod error;
mod id;
mod miu;
mod pipeline;
mod scenario;

pub use clue::{Clue, JsonClue, RawRecord, SourceRef, TextClue};
pub use error::KbError;
pub use id::{KbId, MiuId, ScopeId};
pub use miu::{Miu, MiuDraft, MiuStatus, SymbolForm, MiuMetadata};
pub use pipeline::{
    Build, Decompose, Gate, GateDecision, SemanticUnit, TextBoundaryDecompose, PassthroughBuild,
    ScoreGate,
};
pub use scenario::ScenarioConfig;
