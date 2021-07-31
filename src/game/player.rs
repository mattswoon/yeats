use serenity::model::user::User;
use std::fmt::{Display, Formatter, self};

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

impl Display for Player {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.name, self.user)
    }
}
