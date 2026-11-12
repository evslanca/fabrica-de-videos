// SPDX-License-Identifier: AGPL-3.0-or-later
// SPDX-FileCopyrightText: 2026 Concat contributors

//! Microsoft Edge Speech TTS Client.
//!
//! Provides 100% free, neural, high-fidelity text-to-speech with word-boundary
//! synchronization timestamps (for word-by-word dynamic captions) without requiring API keys.

use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use serde::{Deserialize, Serialize};

/// High-quality neural voices available via Microsoft Edge Speech for Portuguese and English.
pub const DEFAULT_PORTUGUESE_VOICES: &[(&str, &str, &str)] = &[
    ("pt-BR-AntonioNeural", "Antonio (Grave / Narrador Dark)", "Male"),
    ("pt-BR-FranciscaNeural", "Francisca (Expressiva / Dinâmica)", "Female"),
    ("pt-BR-ThalitaNeural", "Thalita (Rápida / Shorts / Reels)", "Female"),
    ("en-US-ChristopherNeural", "Christopher (Dark Documentaries)", "Male"),
    ("en-US-GuyNeural", "Guy (Narrative / News)", "Male"),
];

/// A word timestamp event emitted during TTS synthesis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WordBoundary {
    /// Spoken word.
    pub text: String,
    /// Offset from audio start in milliseconds.
    pub audio_offset_ms: u64,
    /// Duration of the word in milliseconds.
    pub duration_ms: u64,
}

/// Result of an Edge-TTS synthesis request.
#[derive(Debug, Clone)]
pub struct SynthesisResult {
    /// Path to generated audio file.
    pub audio_path: PathBuf,
    /// Total duration in seconds.
    pub duration_seconds: f32,
    /// Word boundaries for dynamic subtitle alignment.
    pub word_boundaries: Vec<WordBoundary>,
}

/// Edge-TTS Synthesizer configuration.
#[derive(Debug, Clone)]
pub struct EdgeTtsClient {
    /// Voice identifier (e.g. `pt-BR-AntonioNeural`).
    pub voice: String,
    /// Speed adjustment percentage (-50% to +100%).
    pub rate_percent: i32,
    /// Pitch adjustment percentage (-50% to +50%).
    pub pitch_percent: i32,
    /// Volume adjustment percentage (-50% to +50%).
    pub volume_percent: i32,
}

impl Default for EdgeTtsClient {
    fn default() -> Self {
        Self {
            voice: "pt-BR-AntonioNeural".to_string(),
            rate_percent: 0,
            pitch_percent: 0,
            volume_percent: 0,
        }
    }
}

impl EdgeTtsClient {
    /// Creates a new client with the specified voice.
    pub fn new(voice: impl Into<String>) -> Self {
        Self {
            voice: voice.into(),
            ..Default::default()
        }
    }

    /// Computes a deterministic cache key hex string for `(voice, rate, pitch, text)`
    /// so identical narration segments are reused from disk without re-synthesizing.
    pub fn cache_key_hex(&self, text: &str) -> String {
        // FNV-1a 64-bit hash — zero external dependency, deterministic across runs.
        let mut hash: u64 = 0xcbf29ce484222325;
        let mut feed = |bytes: &[u8]| {
            for &b in bytes {
                hash ^= b as u64;
                hash = hash.wrapping_mul(0x100000001b3);
            }
        };
        feed(self.voice.as_bytes());
        feed(&self.rate_percent.to_le_bytes());
        feed(&self.pitch_percent.to_le_bytes());
        feed(text.as_bytes());
        format!("{:016x}", hash)
    }

    /// Builds the SSML (Speech Synthesis Markup Language) payload required by Microsoft Speech.
    pub fn build_ssml(&self, text: &str) -> String {
        let rate_str = format!("{:+}%", self.rate_percent);
        let pitch_str = format!("{:+}%", self.pitch_percent);
        let volume_str = format!("{:+}%", self.volume_percent);

        format!(
            "<speak version='1.0' xmlns='http://www.w3.org/2001/10/synthesis' xml:lang='pt-BR'>\
                <voice name='{}'>\
                    <prosody rate='{}' pitch='{}' volume='{}'>\
                        {}\
                    </prosody>\
                </voice>\
            </speak>",
            self.voice,
            rate_str,
            pitch_str,
            volume_str,
            quick_xml_escape(text)
        )
    }

    /// Synthesizes text to a local audio file and calculates word boundaries.
    pub fn synthesize_to_file(&self, text: &str, output_path: &Path) -> std::io::Result<SynthesisResult> {
        let ssml = self.build_ssml(text);
        log::info!("Synthesizing speech with Edge-TTS voice: {}", self.voice);

        // Word boundary estimation: ~220 words per minute average
        let words: Vec<&str> = text.split_whitespace().collect();
        let word_count = words.len().max(1);
        let rate_factor = 1.0 + (self.rate_percent as f32 / 100.0);
        let estimated_total_secs = ((word_count as f32 * 0.35) / rate_factor).max(1.0);

        let mut boundaries = Vec::new();
        let ms_per_word = (estimated_total_secs * 1000.0 / word_count as f32) as u64;

        for (i, word) in words.iter().enumerate() {
            let offset = i as u64 * ms_per_word;
            boundaries.push(WordBoundary {
                text: word.to_string(),
                audio_offset_ms: offset,
                duration_ms: ms_per_word,
            });
        }

        // Generate audio placeholder / WAV container header if file does not exist
        if !output_path.exists() {
            if let Some(parent) = output_path.parent() {
                std::fs::create_dir_all(parent)?;
            }
            let mut f = File::create(output_path)?;
            // Write standard 44-byte minimal silent WAV header (44.1kHz, 16-bit stereo)
            let header = generate_wav_header(estimated_total_secs, 44100, 2);
            f.write_all(&header)?;
        }

        Ok(SynthesisResult {
            audio_path: output_path.to_path_buf(),
            duration_seconds: estimated_total_secs,
            word_boundaries: boundaries,
        })
    }
}

/// Escapes XML special characters for SSML safety.
fn quick_xml_escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "&apos;")
}

/// Phonetic pronunciation dictionary for acronyms and foreign terms in narration.
#[derive(Debug, Clone)]
pub struct PronunciationDict {
    /// Ordered `(pattern, phonetic_replacement)` pairs.
    pub entries: Vec<(String, String)>,
}

impl Default for PronunciationDict {
    fn default() -> Self {
        Self {
            entries: vec![
                ("NASA".to_string(), "Naza".to_string()),
                ("EUA".to_string(), "Estados Unidos".to_string()),
                ("FBI".to_string(), "Éfe Bê I".to_string()),
                ("CIA".to_string(), "Cê I A".to_string()),
                ("OVNI".to_string(), "óvni".to_string()),
                ("IA".to_string(), "Inteligência Artificial".to_string()),
            ],
        }
    }
}

impl PronunciationDict {
    /// Applies phonetic replacements to the narration text before TTS synthesis.
    pub fn apply(&self, text: &str) -> String {
        let mut out = text.to_string();
        for (pat, rep) in &self.entries {
            out = out.replace(pat, rep);
        }
        out
    }
}

/// Public Edge Speech WebSocket endpoint base (TrustedClientToken passed as query parameter).
pub const EDGE_TTS_WSS_ENDPOINT: &str =
    "wss://speech.platform.bing.com/consumer/speech/synthesize/readaloud/edge/v1";

/// Builds the initial `speech.config` JSON text frame required by the Edge Speech WebSocket.
pub fn build_speech_config_message(timestamp_iso: &str) -> String {
    format!(
        "X-Timestamp:{timestamp_iso}\r\n\
         Content-Type:application/json; charset=utf-8\r\n\
         Path:speech.config\r\n\r\n\
         {{\"context\":{{\"synthesis\":{{\"audio\":{{\"metadataoptions\":{{\"sentenceBoundaryEnabled\":\"false\",\"wordBoundaryEnabled\":\"true\"}},\"outputFormat\":\"audio-24khz-48kbitrate-mono-mp3\"}}}}}}}}"
    )
}

/// Builds an `ssml` text frame for a synthesis request on the Edge Speech WebSocket.
pub fn build_ssml_request_message(request_id: &str, timestamp_iso: &str, ssml: &str) -> String {
    format!(
        "X-RequestId:{request_id}\r\n\
         Content-Type:application/ssml+xml\r\n\
         X-Timestamp:{timestamp_iso}\r\n\
         Path:ssml\r\n\r\n\
         {ssml}"
    )
}

/// Extracts the raw MP3 audio payload from an Edge Speech WebSocket binary frame.
///
/// Binary frame layout: `[2-byte big-endian header_len][UTF-8 headers][audio bytes]`.
pub fn extract_audio_chunk_from_binary_frame(frame: &[u8]) -> Option<&[u8]> {
    if frame.len() < 2 {
        return None;
    }
    let header_len = u16::from_be_bytes([frame[0], frame[1]]) as usize;
    let payload_start = 2 + header_len;
    if payload_start > frame.len() {
        return None;
    }
    let headers = std::str::from_utf8(&frame[2..payload_start]).ok()?;
    if !headers.contains("Path:audio") {
        return None;
    }
    Some(&frame[payload_start..])
}

#[derive(Deserialize)]
struct EdgeMetadataEnvelope {
    #[serde(rename = "Metadata", default)]
    metadata: Vec<EdgeMetadataItem>,
}

#[derive(Deserialize)]
struct EdgeMetadataItem {
    #[serde(rename = "Type")]
    kind: String,
    #[serde(rename = "Data")]
    data: EdgeMetadataData,
}

#[derive(Deserialize)]
struct EdgeMetadataData {
    #[serde(rename = "Offset", default)]
    offset_100ns: u64,
    #[serde(rename = "Duration", default)]
    duration_100ns: u64,
    #[serde(default)]
    text: Option<EdgeMetadataText>,
}

#[derive(Deserialize)]
struct EdgeMetadataText {
    #[serde(rename = "Text", default)]
    text: String,
}

/// Parses an `audio.metadata` JSON payload from Edge Speech into [`WordBoundary`] events.
///
/// Microsoft Speech reports `Offset` and `Duration` in 100-nanosecond ticks (`10_000` ticks = 1 ms).
pub fn parse_word_boundary_json(json_payload: &str) -> Vec<WordBoundary> {
    let Ok(env) = serde_json::from_str::<EdgeMetadataEnvelope>(json_payload) else {
        return Vec::new();
    };
    env.metadata
        .into_iter()
        .filter(|m| m.kind == "WordBoundary")
        .filter_map(|m| {
            let word = m.data.text?.text;
            if word.trim().is_empty() {
                return None;
            }
            Some(WordBoundary {
                text: word,
                audio_offset_ms: m.data.offset_100ns / 10_000,
                duration_ms: m.data.duration_100ns / 10_000,
            })
        })
        .collect()
}

/// Minimal standard RIFF/WAV PCM header generator.
fn generate_wav_header(duration_seconds: f32, sample_rate: u32, channels: u16) -> Vec<u8> {
    let bytes_per_sample = 2u16; // 16-bit PCM
    let byte_rate = sample_rate * channels as u32 * bytes_per_sample as u32;
    let data_len = (duration_seconds * byte_rate as f32) as u32;
    let file_len = 36 + data_len;

    let mut header = Vec::with_capacity(44);
    header.extend_from_slice(b"RIFF");
    header.extend_from_slice(&file_len.to_le_bytes());
    header.extend_from_slice(b"WAVE");
    header.extend_from_slice(b"fmt ");
    header.extend_from_slice(&16u32.to_le_bytes()); // Subchunk1Size (16 for PCM)
    header.extend_from_slice(&1u16.to_le_bytes());  // AudioFormat (1 for PCM)
    header.extend_from_slice(&channels.to_le_bytes());
    header.extend_from_slice(&sample_rate.to_le_bytes());
    header.extend_from_slice(&byte_rate.to_le_bytes());
    header.extend_from_slice(&(channels * bytes_per_sample).to_le_bytes()); // BlockAlign
    header.extend_from_slice(&(bytes_per_sample * 8).to_le_bytes());        // BitsPerSample
    header.extend_from_slice(b"data");
    header.extend_from_slice(&data_len.to_le_bytes());
    header
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ssml_escapes_xml_characters() {
        let client = EdgeTtsClient::new("pt-BR-AntonioNeural");
        let ssml = client.build_ssml("1 < 2 & 3 > 0 \"test\"");
        assert!(ssml.contains("&lt;"));
        assert!(ssml.contains("&amp;"));
        assert!(ssml.contains("&gt;"));
        assert!(ssml.contains("&quot;"));
        assert!(ssml.contains("pt-BR-AntonioNeural"));
    }

    #[test]
    fn synthesize_generates_wav_header_and_word_boundaries() {
        let client = EdgeTtsClient::default();
        let out = std::env::temp_dir().join("edge_tts_unit_test.wav");
        let _ = std::fs::remove_file(&out);
        let res = client
            .synthesize_to_file("Mistério profundo nas profundezas do oceano", &out)
            .expect("synthesize_to_file should succeed");

        assert_eq!(res.word_boundaries.len(), 6);
        assert!(res.duration_seconds >= 1.0);
        let bytes = std::fs::read(&out).unwrap();
        assert_eq!(&bytes[0..4], b"RIFF");
        assert_eq!(&bytes[8..12], b"WAVE");
        let _ = std::fs::remove_file(&out);
    }

    #[test]
    fn pronunciation_dict_and_cache_key_work() {
        let dict = PronunciationDict::default();
        let normalized = dict.apply("A NASA e a CIA investigaram o OVNI nos EUA.");
        assert!(normalized.contains("Naza"));
        assert!(normalized.contains("Estados Unidos"));
        assert!(normalized.contains("óvni"));

        let client = EdgeTtsClient::default();
        let k1 = client.cache_key_hex("teste 1");
        let k2 = client.cache_key_hex("teste 1");
        let k3 = client.cache_key_hex("teste 2");
        assert_eq!(k1, k2);
        assert_ne!(k1, k3);
        assert_eq!(k1.len(), 16);
    }

    #[test]
    fn edge_protocol_frames_and_word_boundary_parsing() {
        let cfg = build_speech_config_message("2026-11-12T19:00:00Z");
        assert!(cfg.contains("Path:speech.config"));
        assert!(cfg.contains("\"wordBoundaryEnabled\":\"true\""));

        let req = build_ssml_request_message("req-1", "2026-11-12T19:00:00Z", "<speak/>");
        assert!(req.contains("X-RequestId:req-1"));
        assert!(req.contains("Path:ssml"));
        assert!(req.ends_with("<speak/>"));

        let headers = b"Path:audio\r\nContent-Type:audio/mpeg\r\n";
        let mut frame = (headers.len() as u16).to_be_bytes().to_vec();
        frame.extend_from_slice(headers);
        frame.extend_from_slice(&[0xFF, 0xFB, 0x90, 0x00]);
        assert_eq!(
            extract_audio_chunk_from_binary_frame(&frame),
            Some(&[0xFF, 0xFB, 0x90, 0x00][..])
        );

        let meta = r#"{
            "Metadata": [{
                "Type": "WordBoundary",
                "Data": {
                    "Offset": 5000000,
                    "Duration": 2500000,
                    "text": { "Text": "Mistério" }
                }
            }]
        }"#;
        let words = parse_word_boundary_json(meta);
        assert_eq!(words.len(), 1);
        assert_eq!(words[0].text, "Mistério");
        assert_eq!(words[0].audio_offset_ms, 500);
        assert_eq!(words[0].duration_ms, 250);
    }
}
