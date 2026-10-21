//! Breakthrough 13 — latch findings shared between static (kdb), live (vm) and CLI layers.

#[derive(Clone, Debug)]
pub struct LatchFind {
    pub signal: String,
    pub block_kind: String,
    pub line: u32,
    pub hint: String,
    pub hold_events: u64,
    pub hold_time_units: u64,
    pub first_hold_t: Option<u64>,
}

impl LatchFind {
    /// L3: copy-paste SystemVerilog patch to remove the inferred latch.
    pub fn patch(&self, always_variants: &str) -> String {
        let sig = self.signal.rsplit('.').next().unwrap_or(&self.signal);
        format!(
            "// [VIKAS LATCH PATCH] for `{}` (line {})\n{} begin\n    {} = {}'d0;  // <-- default assignment: latch eliminated\n    // ... original if/case logic unchanged ...\nend\n",
            self.signal,
            self.line,
            always_variants,
            sig,
            32
        )
    }
}
