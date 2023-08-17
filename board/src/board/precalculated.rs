mod magic;
mod nonmagic;

pub(crate) use magic::BISHOP_MAGICS;
pub(crate) use magic::ROOK_MAGICS;
pub(crate) use magic::Magics;
pub(crate) use magic::UnsafeMagicsExt;

pub(crate) use nonmagic::KING_NONMAGICS;
pub(crate) use nonmagic::KNIGHT_NONMAGICS;
pub(crate) use nonmagic::WHITE_PAWN_NONMAGICS;
pub(crate) use nonmagic::BLACK_PAWN_NONMAGICS;
pub(crate) use nonmagic::Nonmagics;
pub(crate) use nonmagic::UnsafeNonmagicsExt;
