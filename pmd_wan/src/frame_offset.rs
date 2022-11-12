use binread::BinRead;
use binwrite::BinWrite;

/// The coordinate of some point in the Pokémon, in the form of X then Y
#[derive(BinWrite, BinRead, Debug, PartialEq, Eq, Clone)]
#[binwrite(little)]
#[br(little)]
pub struct FrameOffset {
    head: (i16, i16),
    hand_left: (i16, i16),
    hand_right: (i16, i16),
    center: (i16, i16),
}
