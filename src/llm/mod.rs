//! LLM abstraction trait.

use crate::llm::plan::Project;

pub mod deepseek;
pub mod plan;

/// Any LLM backend that can turn a natural-language instruction into a
/// validated `Project` (wiring guide + program).
pub trait LlmClient {
    fn complete(&self, instruction: &str) -> anyhow::Result<Project>;
}
