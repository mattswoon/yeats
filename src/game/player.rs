use serenity::model::user::User;

#[derive(Clone)]
pub struct Player {
    pub name: String,
    pub user: User
}

impl Player {
    pub fn from_user(u: &User) -> Player {
        Player {
            name: u.name.clone(),
            user: u.clone()
        }
    }
}
