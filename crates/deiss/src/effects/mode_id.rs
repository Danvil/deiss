#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModeId(pub u32);

impl From<u32> for ModeId {
    fn from(value: u32) -> Self {
        ModeId(value)
    }
}

impl From<ModeId> for u32 {
    fn from(mode: ModeId) -> Self {
        mode.0
    }
}
