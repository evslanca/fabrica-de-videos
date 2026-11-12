// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Concat contributors

//! Procedural Ken Burns, Film Grain, Vignette, Color Grading LUTs, and Visual FX
//! for Dark Videos.
//!
//! Applies dynamic camera movement (pan & slow zoom) to static footage/images,
//! overlays 35mm film grain + dark vignettes + GPU FX presets to defeat
//! "reused content" flags while establishing cinematic tension.

use serde::{Deserialize, Serialize};

/// Ken Burns camera motion trajectory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum KenBurnsMotion {
    /// Slow zoom into the center of the frame (1.0x -> 1.15x).
    #[default]
    ZoomInCenter,
    /// Slow zoom out revealing surrounding context (1.15x -> 1.0x).
    ZoomOutCenter,
    /// Pan smoothly from left to right with slight 1.08x zoom.
    PanLeftToRight,
    /// Pan smoothly from right to left with slight 1.08x zoom.
    PanRightToLeft,
}

/// GPU Visual FX overlay preset for dark channels and Shorts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum VisualFxPreset {
    #[default]
    None,
    /// RGB chromatic aberration & digital glitch on cuts.
    ChromaticGlitch,
    /// Retro VHS / CRT scanlines with subtle jitter.
    VhsScanlines,
    /// Floating dark dust & ash particles.
    DustAndAsh,
    /// Subtle zoom pulse synchronized with music beats / SFX impacts.
    BeatZoomPulse,
    /// Old projector flicker for horror/mystery narratives.
    NoirFlicker,
}

/// Dark-channel color grading LUT preset.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ColorGradeLut {
    #[default]
    NoirHighContrast,
    TealAndOrange,
    HorrorDesaturated,
    GoldenStoic,
    None,
}

/// Parameters for anti-reused content visual filtering and FX.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntiReusedFilterParams {
    /// Ken Burns camera trajectory.
    pub motion: KenBurnsMotion,
    /// Intensity of 35mm film grain (0.0 = none, 0.15 = subtle cinematic).
    pub film_grain_intensity: f32,
    /// Edge vignette darkness (0.0 = none, 0.45 = deep dramatic corners).
    pub vignette_darkness: f32,
    /// Micro speed variation factor (e.g. 1.015x to change digital audio/video hash).
    pub speed_variation: f32,
    /// Whether horizontal mirroring is permitted.
    pub allow_horizontal_flip: bool,
    /// GPU visual FX overlay preset.
    pub visual_fx: VisualFxPreset,
    /// Dark color grading LUT preset.
    pub color_lut: ColorGradeLut,
}

impl Default for AntiReusedFilterParams {
    fn default() -> Self {
        Self {
            motion: KenBurnsMotion::ZoomInCenter,
            film_grain_intensity: 0.12,
            vignette_darkness: 0.40,
            speed_variation: 1.012,
            allow_horizontal_flip: true,
            visual_fx: VisualFxPreset::DustAndAsh,
            color_lut: ColorGradeLut::NoirHighContrast,
        }
    }
}

/// Applies a Dark Channel [`ColorGradeLut`] in-place to an RGBA8 pixel buffer.
pub fn apply_color_grade_rgba(pixels: &mut [u8], lut: ColorGradeLut) {
    if lut == ColorGradeLut::None {
        return;
    }
    for px in pixels.chunks_exact_mut(4) {
        let r = px[0] as f32 / 255.0;
        let g = px[1] as f32 / 255.0;
        let b = px[2] as f32 / 255.0;
        let luma = 0.2126 * r + 0.7152 * g + 0.0722 * b;

        let (nr, ng, nb) = match lut {
            ColorGradeLut::NoirHighContrast => {
                let s = (luma - 0.5) * 1.35 + 0.5;
                (
                    (r * 0.45 + s * 0.55) * 0.92,
                    (g * 0.45 + s * 0.55) * 0.92,
                    (b * 0.45 + s * 0.55) * 0.95,
                )
            }
            ColorGradeLut::TealAndOrange => (
                (r * 1.10 + (1.0 - luma) * -0.04).min(1.0),
                (g * 0.98 + (1.0 - luma) * 0.03).min(1.0),
                (b * 0.88 + (1.0 - luma) * 0.14).min(1.0),
            ),
            ColorGradeLut::HorrorDesaturated => (
                luma * 0.72 + r * 0.20,
                luma * 0.76 + g * 0.22,
                luma * 0.82 + b * 0.26,
            ),
            ColorGradeLut::GoldenStoic => (
                (r * 1.12 + 0.04).min(1.0),
                (g * 1.02 + 0.01).min(1.0),
                b * 0.82,
            ),
            ColorGradeLut::None => (r, g, b),
        };

        px[0] = (nr.clamp(0.0, 1.0) * 255.0) as u8;
        px[1] = (ng.clamp(0.0, 1.0) * 255.0) as u8;
        px[2] = (nb.clamp(0.0, 1.0) * 255.0) as u8;
    }
}

/// Applies procedural 35mm film grain and radial edge vignette in-place on an RGBA8 frame.
pub fn apply_vignette_and_grain_rgba(
    pixels: &mut [u8],
    width: u32,
    height: u32,
    params: &AntiReusedFilterParams,
    frame_index: u32,
) {
    if width == 0 || height == 0 {
        return;
    }
    let cx = (width as f32 - 1.0) * 0.5;
    let cy = (height as f32 - 1.0) * 0.5;
    let max_dist_sq = (cx * cx + cy * cy).max(1.0);
    let mut seed = frame_index.wrapping_mul(747796405).wrapping_add(2891336453);

    for y in 0..height {
        let dy = y as f32 - cy;
        for x in 0..width {
            let idx = ((y * width + x) * 4) as usize;
            if idx + 3 >= pixels.len() {
                break;
            }
            let dx = x as f32 - cx;
            let dist_norm = (dx * dx + dy * dy) / max_dist_sq;
            let vig = (1.0 - params.vignette_darkness * dist_norm).clamp(0.15, 1.0);

            seed = seed.wrapping_mul(1664525).wrapping_add(1013904223);
            let grain = (((seed >> 24) as f32 / 255.0) - 0.5) * 2.0 * params.film_grain_intensity * 255.0;

            for c in 0..3 {
                let v = pixels[idx + c] as f32 * vig + grain;
                pixels[idx + c] = v.clamp(0.0, 255.0) as u8;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_grade_and_vignette_modify_rgba_frame_safely() {
        let mut frame = vec![128u8, 120, 110, 255, 200, 180, 160, 255, 60, 70, 80, 255, 10, 20, 30, 255];
        apply_color_grade_rgba(&mut frame, ColorGradeLut::GoldenStoic);
        assert!(frame[0] > 128);
        assert!(frame[2] < 110);

        let params = AntiReusedFilterParams::default();
        apply_vignette_and_grain_rgba(&mut frame, 2, 2, &params, 1);
        assert_eq!(frame[3], 255);
    }

    #[test]
    fn generates_valid_wgsl_shader_for_fx_and_lut() {
        let wgsl = wgsl_dark_fx_shader(VisualFxPreset::ChromaticGlitch, ColorGradeLut::TealAndOrange);
        assert!(wgsl.contains("@fragment"));
        assert!(wgsl.contains("fn fs_main"));
        assert!(wgsl.contains("ChromaticGlitch"));
        assert!(wgsl.contains("TealAndOrange"));
    }
}

/// Generates a WGSL fragment shader implementing the selected [`VisualFxPreset`]
/// and [`ColorGradeLut`] for GPU compositing in `concat-render`.
pub fn wgsl_dark_fx_shader(fx: VisualFxPreset, lut: ColorGradeLut) -> String {
    let fx_body = match fx {
        VisualFxPreset::ChromaticGlitch => {
            "// ChromaticGlitch\n\
             let shift = u.intensity * 0.012 * sin(u.time * 18.0);\n\
             let r = textureSample(tex, samp, uv + vec2<f32>(shift, 0.0)).r;\n\
             let g = textureSample(tex, samp, uv).g;\n\
             let b = textureSample(tex, samp, uv - vec2<f32>(shift, 0.0)).b;\n\
             var col = vec3<f32>(r, g, b);"
        }
        VisualFxPreset::VhsScanlines => {
            "// VhsScanlines\n\
             let jitter = sin(uv.y * 120.0 + u.time * 24.0) * 0.002 * u.intensity;\n\
             var col = textureSample(tex, samp, uv + vec2<f32>(jitter, 0.0)).rgb;\n\
             let scan = 1.0 - 0.18 * u.intensity * (0.5 + 0.5 * sin(uv.y * 720.0));\n\
             col = col * scan;"
        }
        VisualFxPreset::DustAndAsh => {
            "// DustAndAsh\n\
             var col = textureSample(tex, samp, uv).rgb;\n\
             let cell = floor((uv + vec2<f32>(0.0, u.time * 0.05)) * 180.0);\n\
             let h = fract(sin(dot(cell, vec2<f32>(12.9898, 78.233))) * 43758.5453);\n\
             if (h > 0.994) { col = col + vec3<f32>(0.25 * u.intensity); }"
        }
        VisualFxPreset::BeatZoomPulse => {
            "// BeatZoomPulse\n\
             let pulse = 1.0 - 0.03 * u.intensity * abs(sin(u.time * 4.71));\n\
             let puv = (uv - vec2<f32>(0.5)) * pulse + vec2<f32>(0.5);\n\
             var col = textureSample(tex, samp, puv).rgb;"
        }
        VisualFxPreset::NoirFlicker => {
            "// NoirFlicker\n\
             var col = textureSample(tex, samp, uv).rgb;\n\
             let flick = 1.0 - 0.12 * u.intensity * fract(sin(u.time * 31.7) * 43758.5);\n\
             col = col * flick;"
        }
        VisualFxPreset::None => "var col = textureSample(tex, samp, uv).rgb;",
    };

    let lut_body = match lut {
        ColorGradeLut::NoirHighContrast => {
            "// NoirHighContrast\n\
             let l = dot(col, vec3<f32>(0.2126, 0.7152, 0.0722));\n\
             let s = (l - 0.5) * 1.35 + 0.5;\n\
             col = clamp((col * 0.45 + vec3<f32>(s * 0.55)) * 0.92, vec3<f32>(0.0), vec3<f32>(1.0));"
        }
        ColorGradeLut::TealAndOrange => {
            "// TealAndOrange\n\
             let l = dot(col, vec3<f32>(0.2126, 0.7152, 0.0722));\n\
             col = clamp(col * vec3<f32>(1.10, 0.98, 0.88) + (1.0 - l) * vec3<f32>(-0.04, 0.03, 0.14), vec3<f32>(0.0), vec3<f32>(1.0));"
        }
        ColorGradeLut::HorrorDesaturated => {
            "// HorrorDesaturated\n\
             let l = dot(col, vec3<f32>(0.2126, 0.7152, 0.0722));\n\
             col = clamp(vec3<f32>(l * 0.72, l * 0.76, l * 0.82) + col * vec3<f32>(0.20, 0.22, 0.26), vec3<f32>(0.0), vec3<f32>(1.0));"
        }
        ColorGradeLut::GoldenStoic => {
            "// GoldenStoic\n\
             col = clamp(col * vec3<f32>(1.12, 1.02, 0.82) + vec3<f32>(0.04, 0.01, 0.0), vec3<f32>(0.0), vec3<f32>(1.0));"
        }
        ColorGradeLut::None => "// None",
    };

    format!(
        "struct DarkFxUniforms {{\n\
             time: f32,\n\
             intensity: f32,\n\
             vignette: f32,\n\
             _pad: f32,\n\
         }};\n\
         @group(0) @binding(0) var tex: texture_2d<f32>;\n\
         @group(0) @binding(1) var samp: sampler;\n\
         @group(0) @binding(2) var<uniform> u: DarkFxUniforms;\n\n\
         @fragment\n\
         fn fs_main(@location(0) uv: vec2<f32>) -> @location(0) vec4<f32> {{\n\
             {fx_body}\n\
             {lut_body}\n\
             let d = distance(uv, vec2<f32>(0.5));\n\
             let vig = clamp(1.0 - u.vignette * d * d * 2.0, 0.15, 1.0);\n\
             return vec4<f32>(col * vig, 1.0);\n\
         }}\n"
    )
}
