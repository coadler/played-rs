use foundationdb::options::StreamingMode;
use foundationdb::tuple;
use foundationdb::RangeOption;

const SUBSPACE_PREFIX: &[u8] = b"played";

enum Subspace {
    FirstSeen = 1,
    LastUpdated = 2,
    Current = 3,
    UserGame = 4,
    Whitelist = 5,
}

pub(crate) fn fmt_first_seen_key(user: &[u8]) -> Vec<u8> {
    tuple::Subspace::all()
        .subspace(&SUBSPACE_PREFIX)
        .subspace(&(Subspace::FirstSeen as u16))
        .pack(&user)
}

pub(crate) fn fmt_first_seen_range() -> RangeOption<'static> {
    let mut rg = RangeOption::from(
        tuple::Subspace::all()
            .subspace(&SUBSPACE_PREFIX)
            .subspace(&(Subspace::FirstSeen as u16))
            .range(),
    );
    rg.mode = StreamingMode::WantAll;

    rg
}

pub(crate) fn fmt_last_updated_key(user: &[u8]) -> Vec<u8> {
    tuple::Subspace::all()
        .subspace(&SUBSPACE_PREFIX)
        .subspace(&(Subspace::LastUpdated as u16))
        .pack(&user)
}

pub(crate) fn fmt_last_updated_range() -> RangeOption<'static> {
    RangeOption::from(
        tuple::Subspace::all()
            .subspace(&SUBSPACE_PREFIX)
            .subspace(&(Subspace::LastUpdated as u16))
            .range(),
    )
}

pub(crate) fn fmt_current_game_key(user: &[u8]) -> Vec<u8> {
    tuple::Subspace::all()
        .subspace(&SUBSPACE_PREFIX)
        .subspace(&(Subspace::Current as u16))
        .pack(&user)
}

pub(crate) fn fmt_current_game_range() -> RangeOption<'static> {
    let mut rg = RangeOption::from(
        tuple::Subspace::all()
            .subspace(&SUBSPACE_PREFIX)
            .subspace(&(Subspace::Current as u16))
            .range(),
    );
    rg.mode = StreamingMode::WantAll;

    rg
}

pub(crate) type UserGameKey = (Vec<u8>, u16, Vec<u8>, Vec<u8>);

pub(crate) fn fmt_user_game(user: &[u8], game: &[u8]) -> Vec<u8> {
    tuple::Subspace::all()
        .subspace(&SUBSPACE_PREFIX)
        .subspace(&(Subspace::UserGame as u16))
        .pack(&(user, game))
}

#[allow(dead_code)]
pub(crate) fn fmt_user_range<'a>(user: &[u8]) -> RangeOption<'static> {
    let mut rg = RangeOption::from(
        tuple::Subspace::all()
            .subspace(&SUBSPACE_PREFIX)
            .subspace(&(Subspace::UserGame as u16))
            .subspace(&user)
            .range(),
    );
    rg.mode = StreamingMode::WantAll;

    rg
}

pub(crate) fn fmt_whitelist_user(user: &[u8]) -> Vec<u8> {
    tuple::Subspace::all()
        .subspace(&SUBSPACE_PREFIX)
        .subspace(&(Subspace::Whitelist as u16))
        .pack(&user)
}
