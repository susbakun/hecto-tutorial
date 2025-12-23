use super::{GraphemeIdx, LineIdx};

#[derive(Copy, Clone, Default, Debug)]
pub struct Location {
    pub grapheme_idx: GraphemeIdx,
    pub line_idx: LineIdx,
}
