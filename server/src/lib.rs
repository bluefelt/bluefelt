use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone)]
pub struct Lobby {
    pub id: Uuid,
    pub game_id: String,
}

pub type LobbyMap = dashmap::DashMap<Uuid, Lobby>;
