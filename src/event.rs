use crate::gameplay::health::HitDetails;

#[derive(Debug, Copy, Clone)]
pub enum GameEvent {
    Delete(hecs::Entity),
    Hit(hecs::Entity, HitDetails),
    GameOver,
    TextUpdated,
    EnemyDied(hecs::Entity),
}
