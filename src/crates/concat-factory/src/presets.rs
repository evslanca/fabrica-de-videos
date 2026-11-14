//! Professional presets, editing styles, color palettes, and production details
//! for the AI Video Factory and MCP tools.

use serde::{Deserialize, Serialize};

/// Target platform preset with pre-configured aspect ratio, pacing, and audio targets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum PlatformPreset {
    /// YouTube 16:9 (Aula, Review, Documentário) — -14 LUFS, 1920x1080.
    #[default]
    Youtube16x9,
    /// YouTube Shorts 9:16 — hook rápido, legenda palavra por palavra.
    YoutubeShorts,
    /// Instagram Reels 9:16 — safe zones de Reels, ritmo dinâmico.
    InstagramReels,
    /// Instagram Feed 4:5 — 1080x1350, foco em retenção visual.
    InstagramFeed,
    /// Facebook Feed 1:1 — 1080x1080, legenda forte para autoplay mudo.
    FacebookFeed,
    /// TikTok Viral 9:16 — ritmo agressivo, cortes secos, SFX médios.
    TikTokViral,
    /// LinkedIn Profissional 16:9 — ritmo suave, paleta corporativa/clean.
    LinkedinPro,
    /// Podcast Clip 9:16 — foco no falante + legenda destacada.
    PodcastClip,
}

impl PlatformPreset {
    /// Parses a preset from a user/MCP string.
    pub fn from_str_loose(s: &str) -> Self {
        let lower = s.to_lowercase();
        if lower.contains("shorts") {
            Self::YoutubeShorts
        } else if lower.contains("reels") || (lower.contains("instagram") && !lower.contains("feed")) {
            Self::InstagramReels
        } else if lower.contains("instagram") && lower.contains("feed") {
            Self::InstagramFeed
        } else if lower.contains("facebook") {
            Self::FacebookFeed
        } else if lower.contains("tiktok") {
            Self::TikTokViral
        } else if lower.contains("linkedin") {
            Self::LinkedinPro
        } else if lower.contains("podcast") {
            Self::PodcastClip
        } else {
            Self::Youtube16x9
        }
    }

    /// Human-readable label.
    pub fn label(&self) -> &'static str {
        match self {
            Self::Youtube16x9 => "YouTube 16:9",
            Self::YoutubeShorts => "YouTube Shorts 9:16",
            Self::InstagramReels => "Instagram Reels 9:16",
            Self::InstagramFeed => "Instagram Feed 4:5",
            Self::FacebookFeed => "Facebook Feed 1:1",
            Self::TikTokViral => "TikTok Viral 9:16",
            Self::LinkedinPro => "LinkedIn Pro 16:9",
            Self::PodcastClip => "Podcast Clip 9:16",
        }
    }

    /// Whether this preset is vertical (9:16).
    pub fn is_vertical(&self) -> bool {
        matches!(
            self,
            Self::YoutubeShorts | Self::InstagramReels | Self::TikTokViral | Self::PodcastClip
        )
    }
}

/// Visual editing style (matching the Pro Studio cards).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum EditingStyle {
    /// Tela cheia de elementos e interfaces em 3D + cartão vertical flutuante à direita.
    IlustrativaPip,
    /// Falante de um lado com zoom vivo, pontos entrando um a um do outro lado.
    LadoALado,
    /// Fundo branco, interface recriada em código inclinada em 3D + câmera próxima.
    ReviewSoftware,
    /// Fala direta com zoom in/out contínuo focado nas frases-chave.
    ZoomDinamico,
    /// Títulos e listas grandes animados como slides vivos + bolha redonda no canto.
    ApresentacaoAnimada,
    /// IA escolhe cena por cena misturando os estilos acima.
    #[default]
    AutomaticoMix,
}

impl EditingStyle {
    /// Parses an editing style from a user/MCP string.
    pub fn from_str_loose(s: &str) -> Self {
        let lower = s.to_lowercase();
        if lower.contains("ilustrativa") || lower.contains("pip") {
            Self::IlustrativaPip
        } else if lower.contains("lado") {
            Self::LadoALado
        } else if lower.contains("review") || lower.contains("software") {
            Self::ReviewSoftware
        } else if lower.contains("zoom") {
            Self::ZoomDinamico
        } else if lower.contains("apresenta") || lower.contains("curso") {
            Self::ApresentacaoAnimada
        } else {
            Self::AutomaticoMix
        }
    }
}

/// Pre-configured color palettes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ColorPalette {
    #[default]
    AzulGlass,
    Neon,
    Fogo,
    Dinheiro,
    RosaChoque,
    Mono,
    ArcoIris,
}

impl ColorPalette {
    /// Parses a palette from a user/MCP string.
    pub fn from_str_loose(s: &str) -> Self {
        let lower = s.to_lowercase();
        if lower.contains("neon") {
            Self::Neon
        } else if lower.contains("fogo") || lower.contains("fire") {
            Self::Fogo
        } else if lower.contains("dinheiro") || lower.contains("money") || lower.contains("green") {
            Self::Dinheiro
        } else if lower.contains("rosa") || lower.contains("pink") {
            Self::RosaChoque
        } else if lower.contains("mono") {
            Self::Mono
        } else if lower.contains("arco") || lower.contains("rainbow") {
            Self::ArcoIris
        } else {
            Self::AzulGlass
        }
    }

    /// Returns `(highlight_hex, effects_hex, caption_hex)` for the palette.
    pub fn colors(&self) -> (&'static str, &'static str, &'static str) {
        match self {
            Self::AzulGlass => ("#3B82F6", "#60A5FA", "#93C5FD"),
            Self::Neon => ("#A855F7", "#22D3EE", "#E879F9"),
            Self::Fogo => ("#F97316", "#EF4444", "#FACC15"),
            Self::Dinheiro => ("#22C55E", "#10B981", "#FACC15"),
            Self::RosaChoque => ("#EC4899", "#F43F5E", "#FDE047"),
            Self::Mono => ("#F8FAFC", "#94A3B8", "#FFFFFF"),
            Self::ArcoIris => ("#38BDF8", "#A855F7", "#FACC15"),
        }
    }
}

/// Background behavior during Motion SaaS scenes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum BackgroundMode {
    #[default]
    EscuroTrocando,
    BrancoTrocando,
    AlternarEscuroBranco,
    EuDecido,
}

/// Camera and element motion intensity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MotionRhythm {
    Suave,
    #[default]
    Medio,
    Agressivo,
}

/// Caption / subtitle display mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CaptionMode {
    #[default]
    PalavraPorPalavra,
    FraseCurta,
    SoPalavrasChave,
    SemLegenda,
}

/// Automatic silence/mistake cutting mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum CutMode {
    #[default]
    CortarPausasEErros,
    SoPausasLongas,
    NaoCortar,
}

/// Sound effects intensity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum SfxLevel {
    #[default]
    Baixo,
    Medio,
    SemSfx,
}

/// Background music mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum MusicMode {
    #[default]
    SemMusica,
    MusicaSuave,
    MusicaDinamica,
}

/// Brand/tool logo and icon rendering style.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum LogoStyle {
    #[default]
    Logos3dReais,
    IconesSimples,
    NaoMostrar,
}

/// Rendering / motion engine (100% native, no external placeholders).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum RenderEngine {
    #[default]
    EuDecido,
    NativoFfmpeg,
    RapidoDraft,
}

/// First 3 seconds hook strategy.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum HookMode {
    #[default]
    GanchoForteTitulo,
    EuDecido,
}

/// Extra Pro: Transition style between scenes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TransitionStyle {
    CorteSeco,
    #[default]
    WhooshBlur,
    ZoomMorph,
    GlitchFlash,
}

/// Extra Pro: Color grading LUT.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum ColorGradeLut {
    #[default]
    Natural,
    DarkCinema,
    TealAndOrange,
    CrispTech,
}

/// Complete professional configuration for the AI Video Factory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactoryProConfig {
    pub platform: PlatformPreset,
    pub editing_style: EditingStyle,
    pub color_palette: ColorPalette,
    pub highlight_color: String,
    pub effects_color: String,
    pub caption_color: String,
    pub background: BackgroundMode,
    pub motion_rhythm: MotionRhythm,
    pub caption_mode: CaptionMode,
    pub cut_mode: CutMode,
    pub sfx_level: SfxLevel,
    pub music_mode: MusicMode,
    pub logo_style: LogoStyle,
    pub render_engine: RenderEngine,
    pub hook_mode: HookMode,
    // Extras Pro (além das imagens)
    pub transition_style: TransitionStyle,
    pub color_grade: ColorGradeLut,
    pub auto_ducking: bool,
    pub safe_zones: bool,
    /// Advanced AI enhancements (Viral Score, Auto-Reframe, Text Editing, Eye Contact, B-Roll/Emojis, Studio Sound).
    #[serde(default)]
    pub ai: crate::ai_enhancements::AiEnhancementsConfig,
}

impl Default for FactoryProConfig {
    fn default() -> Self {
        let palette = ColorPalette::AzulGlass;
        let (h, e, c) = palette.colors();
        Self {
            platform: PlatformPreset::Youtube16x9,
            editing_style: EditingStyle::AutomaticoMix,
            color_palette: palette,
            highlight_color: h.to_string(),
            effects_color: e.to_string(),
            caption_color: c.to_string(),
            background: BackgroundMode::EscuroTrocando,
            motion_rhythm: MotionRhythm::Medio,
            caption_mode: CaptionMode::PalavraPorPalavra,
            cut_mode: CutMode::CortarPausasEErros,
            sfx_level: SfxLevel::Baixo,
            music_mode: MusicMode::SemMusica,
            logo_style: LogoStyle::Logos3dReais,
            render_engine: RenderEngine::EuDecido,
            hook_mode: HookMode::GanchoForteTitulo,
            transition_style: TransitionStyle::WhooshBlur,
            color_grade: ColorGradeLut::Natural,
            auto_ducking: true,
            safe_zones: false,
            ai: crate::ai_enhancements::AiEnhancementsConfig::default(),
        }
    }
}

impl FactoryProConfig {
    /// Applies sensible defaults for a given [`PlatformPreset`].
    pub fn apply_platform_preset(&mut self, preset: PlatformPreset) {
        self.platform = preset;
        match preset {
            PlatformPreset::Youtube16x9 => {
                self.editing_style = EditingStyle::IlustrativaPip;
                self.motion_rhythm = MotionRhythm::Medio;
                self.caption_mode = CaptionMode::FraseCurta;
                self.safe_zones = false;
            }
            PlatformPreset::YoutubeShorts | PlatformPreset::InstagramReels | PlatformPreset::TikTokViral => {
                self.editing_style = EditingStyle::ZoomDinamico;
                self.motion_rhythm = MotionRhythm::Agressivo;
                self.caption_mode = CaptionMode::PalavraPorPalavra;
                self.hook_mode = HookMode::GanchoForteTitulo;
                self.safe_zones = true;
            }
            PlatformPreset::InstagramFeed | PlatformPreset::FacebookFeed => {
                self.editing_style = EditingStyle::ApresentacaoAnimada;
                self.motion_rhythm = MotionRhythm::Medio;
                self.caption_mode = CaptionMode::PalavraPorPalavra;
                self.safe_zones = true;
            }
            PlatformPreset::LinkedinPro => {
                self.editing_style = EditingStyle::LadoALado;
                self.motion_rhythm = MotionRhythm::Suave;
                self.caption_mode = CaptionMode::FraseCurta;
                self.color_palette = ColorPalette::AzulGlass;
                self.safe_zones = false;
            }
            PlatformPreset::PodcastClip => {
                self.editing_style = EditingStyle::ZoomDinamico;
                self.motion_rhythm = MotionRhythm::Suave;
                self.caption_mode = CaptionMode::PalavraPorPalavra;
                self.cut_mode = CutMode::SoPausasLongas;
                self.safe_zones = true;
            }
        }
        let (h, e, c) = self.color_palette.colors();
        self.highlight_color = h.to_string();
        self.effects_color = e.to_string();
        self.caption_color = c.to_string();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn platform_preset_applies_vertical_and_safe_zones() {
        let mut cfg = FactoryProConfig::default();
        cfg.apply_platform_preset(PlatformPreset::InstagramReels);
        assert!(cfg.platform.is_vertical());
        assert!(cfg.safe_zones);
        assert_eq!(cfg.editing_style, EditingStyle::ZoomDinamico);
        assert_eq!(cfg.motion_rhythm, MotionRhythm::Agressivo);
    }

    #[test]
    fn palette_colors_update_correctly() {
        let pal = ColorPalette::from_str_loose("neon");
        assert_eq!(pal, ColorPalette::Neon);
        let (h, e, c) = pal.colors();
        assert_eq!(h, "#A855F7");
        assert_eq!(e, "#22D3EE");
        assert_eq!(c, "#E879F9");
    }
}
