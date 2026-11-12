// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Concat contributors

//! Factory Timeline Assembler.
//!
//! Takes all processed factory node outputs (voice audio, B-roll clips, word
//! boundaries, SFX cues, caption bursts, ducking keyframes) and constructs a
//! fully-timed Concat project `Document` ready for GPU rendering.

use std::path::PathBuf;
use std::sync::Arc;
use serde::{Deserialize, Serialize};

use crate::auto_ducking::{DuckingParams, GainKeyframe, build_ducking_keyframes};
use crate::model::{
    Clip, ClipKind, MediaItem, MediaKind, MediaOrigin, Project, TextStyle, VideoSettings,
};

/// A single visual scene unit ready for timeline placement.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssembledScene {
    /// Scene sequence number.
    pub index: usize,
    /// Start timecode in seconds.
    pub start_seconds: f32,
    /// Duration in seconds.
    pub duration_seconds: f32,
    /// B-roll clip path.
    pub video_path: PathBuf,
    /// Narration audio for this scene.
    pub audio_path: Option<PathBuf>,
    /// Whether the clip is horizontally mirrored.
    pub mirror_horizontal: bool,
    /// Ken Burns motion applied (string tag, resolved at render time).
    pub ken_burns_motion: String,
}

/// Caption burst: a group of 1-3 words displayed simultaneously.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptionBurst {
    /// Words in this burst.
    pub words: Vec<String>,
    /// Burst start in seconds.
    pub start_seconds: f32,
    /// Burst end in seconds.
    pub end_seconds: f32,
    /// Index of the word currently being spoken (highlighted).
    pub active_word_index: usize,
}

/// Complete assembled factory project, ready for export.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssembledProject {
    /// Total video duration in seconds.
    pub total_duration_seconds: f32,
    /// Target resolution width.
    pub width: u32,
    /// Target resolution height.
    pub height: u32,
    /// Assembled video scenes in order.
    pub scenes: Vec<AssembledScene>,
    /// Caption bursts across the full video.
    pub caption_bursts: Vec<CaptionBurst>,
    /// BGM gain automation curve.
    pub ducking_keyframes: Vec<GainKeyframe>,
    /// Path to the BGM audio track.
    pub bgm_path: Option<PathBuf>,
    /// Path to thumbnail image.
    pub thumbnail_path: Option<PathBuf>,
}

impl AssembledProject {
    /// Assembles a complete project from factory node outputs.
    pub fn assemble(
        scenes: Vec<AssembledScene>,
        bgm_path: Option<PathBuf>,
        caption_bursts: Vec<CaptionBurst>,
        width: u32,
        height: u32,
        ducking_params: &DuckingParams,
    ) -> Self {
        let total_duration = scenes
            .last()
            .map(|s| s.start_seconds + s.duration_seconds)
            .unwrap_or(0.0);

        // Collect voice activity windows from scenes that have audio
        let voice_windows: Vec<(f32, f32)> = scenes
            .iter()
            .filter(|s| s.audio_path.is_some())
            .map(|s| (s.start_seconds, s.start_seconds + s.duration_seconds))
            .collect();

        let ducking_keyframes =
            build_ducking_keyframes(&voice_windows, total_duration, ducking_params);

        Self {
            total_duration_seconds: total_duration,
            width,
            height,
            scenes,
            caption_bursts,
            ducking_keyframes,
            bgm_path,
            thumbnail_path: None,
        }
    }

    /// Converts this assembled factory project into a native Concat [`Project`]
    /// document ready for timeline inspection, editing, and GPU export.
    ///
    /// Track layout:
    /// - `T1`: B-Roll video scenes (with horizontal mirror anti-reuse flag)
    /// - `T2`: Voice narration clips (`MediaOrigin::Speech`)
    /// - `T3`: Background music track (auto-ducked base level)
    /// - `T4`: Dynamic caption bursts (`ClipKind::Text`)
    pub fn to_concat_document(&self) -> Project {
        let mut video = VideoSettings::default();
        if self.width > 0 {
            video.width = self.width;
        }
        if self.height > 0 {
            video.height = self.height;
        }

        let mut project = Project::with_video(video);
        let mut media_seq = 1usize;
        let mut clip_seq = 1usize;

        let mut media_items = Vec::new();
        let mut clips = Vec::new();

        for scene in &self.scenes {
            let duration = (scene.duration_seconds as f64).max(1.0 / 60.0);
            let start = (scene.start_seconds as f64).max(0.0);

            // T1: Video B-Roll
            let v_media_id = format!("m{media_seq}");
            media_seq += 1;
            let v_name = scene
                .video_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("scene.mp4")
                .to_string();
            media_items.push(MediaItem {
                id: v_media_id.clone(),
                path: scene.video_path.to_string_lossy().into_owned(),
                name: v_name.clone(),
                duration: Some(duration),
                kind: MediaKind::Video,
                width: Some(self.width),
                height: Some(self.height),
                ..MediaItem::default()
            });

            let mut v_clip = Clip::blank(
                format!("c{clip_seq}"),
                "T1",
                ClipKind::Video,
                v_name,
                start,
                duration,
            );
            clip_seq += 1;
            v_clip.media_id = v_media_id;
            v_clip.flip_h = scene.mirror_horizontal;
            clips.push(Arc::new(v_clip));

            // T2: Narration Audio (if present)
            if let Some(audio_path) = &scene.audio_path {
                let a_media_id = format!("m{media_seq}");
                media_seq += 1;
                let a_name = audio_path
                    .file_name()
                    .and_then(|n| n.to_str())
                    .unwrap_or("voice.wav")
                    .to_string();
                media_items.push(MediaItem {
                    id: a_media_id.clone(),
                    path: audio_path.to_string_lossy().into_owned(),
                    name: a_name.clone(),
                    duration: Some(duration),
                    kind: MediaKind::Audio,
                    has_audio: true,
                    origin: Some(MediaOrigin::Speech),
                    ..MediaItem::default()
                });

                let mut a_clip = Clip::blank(
                    format!("c{clip_seq}"),
                    "T2",
                    ClipKind::Audio,
                    a_name,
                    start,
                    duration,
                );
                clip_seq += 1;
                a_clip.media_id = a_media_id;
                clips.push(Arc::new(a_clip));
            }
        }

        // T3: Background Music (with ducking base gain)
        if let Some(bgm_path) = &self.bgm_path {
            let bgm_duration = (self.total_duration_seconds as f64).max(1.0 / 60.0);
            let bgm_media_id = format!("m{media_seq}");
            media_seq += 1;
            let bgm_name = bgm_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("bgm.wav")
                .to_string();
            media_items.push(MediaItem {
                id: bgm_media_id.clone(),
                path: bgm_path.to_string_lossy().into_owned(),
                name: bgm_name.clone(),
                duration: Some(bgm_duration),
                kind: MediaKind::Audio,
                has_audio: true,
                ..MediaItem::default()
            });

            let mut bgm_clip = Clip::blank(
                format!("c{clip_seq}"),
                "T3",
                ClipKind::Audio,
                bgm_name,
                0.0,
                bgm_duration,
            );
            clip_seq += 1;
            bgm_clip.media_id = bgm_media_id;
            bgm_clip.volume = self
                .ducking_keyframes
                .iter()
                .map(|k| k.gain as f64)
                .min_by(|a, b| a.total_cmp(b))
                .unwrap_or(0.35);
            clips.push(Arc::new(bgm_clip));
        }

        // T4: Caption Bursts (positioned using aspect-aware Safe Zone)
        let safe_zone = crate::captions_factory::CaptionSafeZone::for_dimensions(
            video.width,
            video.height,
        );
        for burst in &self.caption_bursts {
            let start = (burst.start_seconds as f64).max(0.0);
            let duration = ((burst.end_seconds - burst.start_seconds) as f64).max(1.0 / 60.0);
            let text = burst.words.join(" ");
            let mut t_clip = Clip::blank(
                format!("c{clip_seq}"),
                "T4",
                ClipKind::Text,
                text.clone(),
                start,
                duration,
            );
            clip_seq += 1;
            t_clip.offset_x = safe_zone.offset_x;
            t_clip.offset_y = safe_zone.offset_y;
            t_clip.text = Some(TextStyle {
                content: text,
                ..TextStyle::default()
            });
            clips.push(Arc::new(t_clip));
        }

        let _ = media_seq;
        project.media = media_items;
        project.active_mut().clips = clips;
        project
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converts_assembled_project_to_concat_document() {
        let scenes = vec![AssembledScene {
            index: 0,
            start_seconds: 0.0,
            duration_seconds: 4.5,
            video_path: PathBuf::from("cache/scene0.mp4"),
            audio_path: Some(PathBuf::from("cache/voice0.wav")),
            mirror_horizontal: true,
            ken_burns_motion: "zoom_in".to_string(),
        }];
        let captions = vec![CaptionBurst {
            words: vec!["OLÁ".into(), "MUNDO".into()],
            start_seconds: 0.2,
            end_seconds: 1.4,
            active_word_index: 0,
        }];
        let assembled = AssembledProject::assemble(
            scenes,
            Some(PathBuf::from("cache/bgm.wav")),
            captions,
            1080,
            1920,
            &DuckingParams::default(),
        );

        let doc = assembled.to_concat_document();
        assert_eq!(doc.active().video.width, 1080);
        assert_eq!(doc.active().video.height, 1920);
        assert_eq!(doc.media.len(), 3);
        assert_eq!(doc.active().clips.len(), 4);
        assert_eq!(doc.active().clips[0].track_id, "T1");
        assert!(doc.active().clips[0].flip_h);
        assert_eq!(doc.active().clips[1].track_id, "T2");
        assert_eq!(doc.active().clips[2].track_id, "T3");
        assert_eq!(doc.active().clips[3].track_id, "T4");
        assert_eq!(
            doc.active().clips[3].text.as_ref().unwrap().content,
            "OLÁ MUNDO"
        );
    }
}
