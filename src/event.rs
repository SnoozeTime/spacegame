#[derive(Debug, Copy, Clone)]
pub enum GameEvent {
    Delete(hecs::Entity),
    Hit(hecs::Entity),
    GameOver,
    TextUpdated,
    EnemyDied(hecs::Entity),
}
