#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    EventLoopError(#[from] winit::error::EventLoopError),
    RenderError(String),
}
