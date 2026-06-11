//! AI 分析引擎核心模块
//! 7 维度分析 + PEG 估值 + 多空辩论 + 质量门控

pub mod auto_trigger;
pub mod checkpoint;
pub mod debate;
pub mod decision_memory;
pub mod dimensions;
pub mod engine;
pub mod llm;
pub mod peg;
pub mod progress;
pub mod profiles;
pub mod prompts;
pub mod quality_gate;
pub mod report;
pub mod schemas;
