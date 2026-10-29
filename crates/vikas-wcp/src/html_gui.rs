//! VIKAS STUDIO — compatibility shim: the old `vikas gui` entry point now
//! generates the full professional Studio workbench. The legacy function
//! signature is preserved so all existing call sites keep working; callers
//! that have extra analysis context should call
//! `studio::generate_studio(dump, &ctx, title, out)` directly instead.

use vikas_wave::WaveDump;

pub fn generate_html_gui(dump: &WaveDump, title: &str, out_path: &std::path::Path) -> anyhow::Result<()> {
    let ctx = super::studio::StudioCtx::default();
    super::studio::generate_studio(dump, &ctx, title, out_path)
}
