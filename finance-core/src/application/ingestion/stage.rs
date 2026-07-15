use thiserror::Error;

/// Errors that can occur at any pipeline stage.
#[derive(Debug, Error)]
pub enum PipelineError {
    #[error("Stage '{stage}' failed: {message}")]
    StageFailed { stage: String, message: String },

    #[error("Stage '{stage}' rejected input: {reason}")]
    Rejected { stage: String, reason: String },
}

/// A single processing step in the ETL pipeline.
///
/// Each stage takes an input, applies its logic, and returns an output.
/// Stages are composed sequentially: Parse → Normalize → Validate → Enrich → Persist.
pub trait PipelineStage {
    type Input;
    type Output;

    fn run(&self, input: Self::Input) -> Result<Self::Output, PipelineError>;
}
