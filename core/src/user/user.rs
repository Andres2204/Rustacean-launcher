pub trait User {
    fn username(&self) -> String;
    fn token(&self) -> String;
}