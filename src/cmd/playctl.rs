use crate::player::PlayerServer;

pub fn play_pause() -> bool {
    let (player, _player_handle) = PlayerServer::start(std::time::Duration::ZERO);
    player.play_pause()
}

pub fn next() -> bool {
    let (player, _player_handle) = PlayerServer::start(std::time::Duration::ZERO);
    player.next()
}

pub fn previous() -> bool {
    let (player, _player_handle) = PlayerServer::start(std::time::Duration::ZERO);
    player.previous()
}
