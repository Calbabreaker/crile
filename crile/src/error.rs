#[derive(Debug, thiserror::Error)]
pub enum EngineError {
    #[error(transparent)]
    EventLoopError(#[from] winit::error::EventLoopError),
    #[error("Error while rendering: {0}")]
    RenderError(String),
}
