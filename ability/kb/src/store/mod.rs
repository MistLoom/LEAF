//! 知识库存储：本地实现与多库 Hub。不依赖工具运行时。

mod hub;
mod io;
mod local;
mod store;

pub use hub::KbHub;
pub use local::LocalKb;
pub use store::{KbCapabilities, KbStore, DimBucket, DimQuery, DimStat, DupEstimate, SearchHit};
