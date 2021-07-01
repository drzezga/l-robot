#[derive(Clone, Debug, PartialEq)]
pub enum ResolveMessageType {
    Error,
    Info,
    Output,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResolveMessage {
    pub msg_type: ResolveMessageType,
    pub content: String
}

impl ResolveMessage {
    pub fn error(content: &str) -> Self {
        Self {
            content: content.into(),
            msg_type: ResolveMessageType::Error
        }
    }

    pub fn info(content: &str) -> Self {
        Self {
            content: content.into(),
            msg_type: ResolveMessageType::Info
        }
    }

    pub fn output(content: &str) -> Self {
        Self {
            content: content.into(),
            msg_type: ResolveMessageType::Output
        }
    }
}