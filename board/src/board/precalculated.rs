mod magic;
mod nonmagic;

pub use magic::BISHOP_MAGICS;
pub use magic::ROOK_MAGICS;
pub use magic::Magics;
pub use magic::UnsafeMagicsExt;

pub use nonmagic::KING_NONMAGICS;
pub use nonmagic::KNIGHT_NONMAGICS;
pub use nonmagic::WHITE_PAWN_NONMAGICS;
pub use nonmagic::BLACK_PAWN_NONMAGICS;
pub use nonmagic::Nonmagics;
pub use nonmagic::UnsafeNonmagicsExt;
