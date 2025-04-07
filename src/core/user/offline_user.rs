use crate::core::user::user::User;

pub struct OfflineUser {
    pub(crate) name: String,
}

impl User for OfflineUser {
    fn username(&self) -> String {
        self.name.to_string().clone()
    }

    fn token(&self) -> String {
        String::from("")
    }
}