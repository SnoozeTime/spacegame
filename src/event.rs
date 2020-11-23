use crate::gameplay::health::HitDetails;

#[derive(Debug, Clone)]
pub enum GameEvent {
    Delete(hecs::Entity),
    Hit(hecs::Entity, HitDetails),
    GameOver,
    TextUpdated,

    /// Enemy that dies, its position, the amount of scrap to gain, the % of chance to drop a pickup.
    EnemyDied(hecs::Entity, glam::Vec2, (u32, u32), u8),

    /// Some text to display for the player. E.g. Pickup.
    InfoText(String),

    /// Play the background music.
    PlayBackgroundMusic(String),

    /// Play some sound
    PlaySound(String),

    /// Start the next stage.
    NextStage(String),

    /// No more stages, you are the boss !
    YouWin,
}
