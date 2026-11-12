// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Concat contributors

//! Native MP4 Exporter Bridge (`concat-factory` ➔ `concat-export`).
//!
//! Converts an [`AssembledProject`] into a [`concat_project::model::Project`],
//! flattens its timeline into [`concat_export::ExportClip`] entries, and
//! dispatches rendering via [`concat_export::render`].

use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use concat_export::flatten::flatten_timeline;
use concat_export::{ExportRequest, Reporter, render};
use concat_media::{ColorRange, RateMode, VideoCodec};
use concat_project::factory_assembler::AssembledProject;
use concat_project::model::ColorSpace;

/// Encoding options for exporting a factory video.
#[derive(Debug, Clone)]
pub struct FactoryExportOptions {
    /// Destination `.mp4` file path.
    pub output_path: PathBuf,
    /// Constant rate factor (lower = higher quality, default 20).
    pub crf: u8,
    /// x264/encoder preset (e.g. `"veryfast"`, `"medium"`).
    pub preset: String,
    /// Target video codec.
    pub codec: VideoCodec,
}

impl Default for FactoryExportOptions {
    fn default() -> Self {
        Self {
            output_path: PathBuf::from("output/factory_video.mp4"),
            crf: 20,
            preset: "medium".to_string(),
            codec: VideoCodec::H264,
        }
    }
}

/// Builds a [`concat_export::ExportRequest`] from an [`AssembledProject`].
pub fn build_export_request(
    assembled: &AssembledProject,
    opts: &FactoryExportOptions,
) -> ExportRequest {
    let project = assembled.to_concat_document();
    let video = project.active().video.clone();
    let clips = flatten_timeline(&project, None);

    ExportRequest {
        output: opts.output_path.to_string_lossy().into_owned(),
        width: video.width,
        height: video.height,
        rate_num: video.rate_num,
        rate_den: video.rate_den,
        crf: opts.crf,
        preset: opts.preset.clone(),
        codec: opts.codec,
        ten_bit: false,
        rate_mode: RateMode::Vbr,
        bitrate_kbps: 0,
        color_range: ColorRange::Limited,
        color_space: ColorSpace::Sdr,
        hdr: false,
        effect_time_offset: 0.0,
        canvas: video.canvas,
        clips,
    }
}

/// Renders an [`AssembledProject`] to an `.mp4` file using `concat-export`.
pub fn export_assembled_project<F>(
    assembled: &AssembledProject,
    opts: &FactoryExportOptions,
    cancel: &AtomicBool,
    mut on_progress: F,
) -> Result<PathBuf, String>
where
    F: FnMut(i64, i64, &'static str),
{
    if let Some(parent) = Path::new(&opts.output_path).parent()
        && !parent.as_os_str().is_empty()
    {
        std::fs::create_dir_all(parent).map_err(|e| format!("failed to create output dir: {e}"))?;
    }

    let request = build_export_request(assembled, opts);
    if request.clips.is_empty() {
        return Err("cannot export empty factory timeline (0 flattened clips)".to_string());
    }

    let mut progress_cb = |frame: i64, total: i64, stage: &'static str| {
        on_progress(frame, total, stage);
    };
    let reporter = Reporter {
        progress: &mut progress_cb,
        cancel,
    };

    render(&request, reporter)?;
    Ok(opts.output_path.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use concat_project::auto_ducking::DuckingParams;
    use concat_project::factory_assembler::AssembledScene;

    #[test]
    fn builds_valid_export_request_from_assembled_project() {
        let scenes = vec![AssembledScene {
            index: 0,
            start_seconds: 0.0,
            duration_seconds: 3.0,
            video_path: PathBuf::from("assets/vault/dark_smoke.mp4"),
            audio_path: Some(PathBuf::from("cache/voice0.wav")),
            mirror_horizontal: false,
            ken_burns_motion: "zoom_in".to_string(),
        }];
        let assembled = AssembledProject::assemble(
            scenes,
            None,
            Vec::new(),
            1080,
            1920,
            &DuckingParams::default(),
        );

        let opts = FactoryExportOptions {
            output_path: PathBuf::from("out/short_01.mp4"),
            ..Default::default()
        };
        let req = build_export_request(&assembled, &opts);
        assert_eq!(req.width, 1080);
        assert_eq!(req.height, 1920);
        assert_eq!(req.output, "out/short_01.mp4");
        assert_eq!(req.clips.len(), 2);
    }
}
