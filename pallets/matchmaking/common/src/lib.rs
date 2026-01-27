//! # 婚恋模块 - 共享类型和工具
//!
//! 本模块提供婚恋系统的共享类型定义和 Trait 接口。
//!
//! ## 功能概述
//!
//! - **类型定义**：用户资料、匹配评分、互动记录等
//! - **Trait 接口**：八字数据提供者、匹配算法、推荐系统等
//!
//! ## 模块结构
//!
//! ```text
//! pallet-matchmaking-common
//! ├── types.rs    # 共享类型定义
//! └── traits.rs   # Trait 接口定义
//! ```

#![cfg_attr(not(feature = "std"), no_std)]

pub mod types;
pub mod traits;

pub use types::*;
pub use traits::*;
