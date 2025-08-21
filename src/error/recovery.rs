//! Error recovery strategies

#[derive(Debug, Clone, Copy)]
pub enum RecoveryStrategy {
    FailFast,
    Continue,
    AutoFix,
}

pub struct ErrorRecovery {}