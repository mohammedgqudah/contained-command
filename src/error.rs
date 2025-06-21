#[derive(Debug)]
pub enum CuriumError {
    InvalidConfig,
    ContainerNotFound,
    ContainerIdAlreadyInUse,
    ContainerIsNotCreated,
    ContainerIsNotStopped,
}

pub type Result<T> = std::result::Result<T, CuriumError>;
