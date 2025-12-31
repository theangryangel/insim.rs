#[derive(Debug, Clone, Copy)]
pub(super) struct ObjectFlags(pub(super) u8);
impl ObjectFlags {
    /// Check if the floating flag is set
    pub(super) fn floating(&self) -> bool {
        self.0 & 0x80 != 0
    }

    /// Extract colour from flags (bits 0-2)
    pub(super) fn colour(&self) -> u8 {
        self.0 & 0x07
    }

    /// Extract mapping from flags (bits 3-6)
    pub(super) fn mapping(&self) -> u8 {
        (self.0 >> 3) & 0x0f
    }
}
