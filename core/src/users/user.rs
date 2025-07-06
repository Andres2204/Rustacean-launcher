pub trait User {
    fn username(&self) -> String;
    fn token(&self) -> String;
}

#[derive(Default)]
pub enum UserType {
    PREMIUM { token: String }, // TODO change
    #[default]
    OFFLINE
}

/*
    Types implementations for the trait User
*/
pub struct OfflineUser {
    name: String,
}

impl User for OfflineUser {
    fn username(&self) -> String {
        self.name.to_owned()
    }
    fn token(&self) -> String {
        String::new()
    }
}

// pub struct PremiumUser {} ; is not implemented yet

/*
    Builder implementation
*/

pub struct UserBuilder {
    name: Option<String>,
    token: Option<String>,
    user_type: UserType
}

impl UserBuilder {
    pub fn new() -> Self {
        UserBuilder {
            name: None,
            token: None,
            user_type: UserType::OFFLINE
        }
    }
    pub fn default() -> impl User {
        OfflineUser { name: "TheRustierOne".to_owned(), }
    }
    pub fn default_boxed() -> Box<dyn User> {
        Box::new(Self::default())
    }
    
    // pub fn premium() -> impl User {}
    
    // TODO: pub cached_or_default() -> impl User {}
}

impl UserBuilder {
    pub fn user_type(mut self, user_type: UserType) -> Self {
        self.user_type = user_type;
        self
    }
    
    pub fn name(mut self, name: String) -> Self {
        self.name = Some(name);
        self
    }
    
    pub fn token(mut self, token: String) -> Self {
        self.token = Some(token);
        self
    }
    
    pub fn build(self) -> Result<impl User, String> {
        match self.user_type {
            UserType::PREMIUM { token } => {
                //if let Some(name) = self.name {
                //    Ok()
                //}
                Err("Premiun is not implemented yet".to_string())
            }
            UserType::OFFLINE => {
                if let Some(name) = self.name {
                    Ok(OfflineUser { name })
                } else {
                    Err("Name is required for offline user".to_string())
                }
            }
        }
    }
}