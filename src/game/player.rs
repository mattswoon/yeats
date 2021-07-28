use serenity::model::user::User;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Player {
    pub name: String,
    pub user: User
}

impl Player {
}

impl From<&User> for Player {
    fn from(u: &User) -> Player {
        Player {
            name: u.name.clone(),
            user: u.clone()
        }
    }
}
