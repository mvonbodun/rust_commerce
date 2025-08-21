pub mod common {
    include!(concat!(env!("OUT_DIR"), "/common.rs"));
}

// Note: Do not re-export at top level to avoid confusion with common::Status
// Users should use shared_proto::common::Status directly

// Helper functions for working with status codes
impl common::Status {
    pub fn ok() -> Self {
        Self {
            code: common::Code::Ok as i32,
            message: String::new(),
            details: vec![],
        }
    }

    pub fn error(code: common::Code, message: impl Into<String>) -> Self {
        Self {
            code: code as i32,
            message: message.into(),
            details: vec![],
        }
    }

    pub fn not_found(message: impl Into<String>) -> Self {
        Self::error(common::Code::NotFound, message)
    }

    pub fn invalid_argument(message: impl Into<String>) -> Self {
        Self::error(common::Code::InvalidArgument, message)
    }

    pub fn internal(message: impl Into<String>) -> Self {
        Self::error(common::Code::Internal, message)
    }

    pub fn already_exists(message: impl Into<String>) -> Self {
        Self::error(common::Code::AlreadyExists, message)
    }
}
