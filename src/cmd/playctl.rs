use crate::player::start_player;

pub fn play_pause() -> bool {
    let (player, _player_handle) = start_player(std::time::Duration::ZERO);
    player.play_pause()
}

pub fn next() -> bool {
    let (player, _player_handle) = start_player(std::time::Duration::ZERO);
    player.next()
}

pub fn previous() -> bool {
    let (player, _player_handle) = start_player(std::time::Duration::ZERO);
    player.previous()
}
