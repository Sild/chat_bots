use std::borrow::Borrow;

pub struct From {
    pub first_name: String,
    pub last_name: String,
    pub username: String,
}

impl From {
    pub fn from(src_user: &telegram_bot::types::User) -> From {
        return From {
            first_name: src_user.first_name.clone(),
            last_name: src_user.last_name.clone().unwrap_or_default(),
            username: src_user.username.clone().unwrap_or_default(),
        };
    }
}

pub struct Message {
    pub data: String
}

impl Message {
    pub fn from(src_msg: &telegram_bot::types::Message) -> Message {
        let mut message_data = String::new();
        if let telegram_bot::MessageKind::Text { ref data, .. } = src_msg.kind {
            message_data = data.to_string()
        }

        return Message {
            data: message_data,
        }
    }
}