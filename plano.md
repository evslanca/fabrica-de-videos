# 🎬 Plano Diretor: Fábrica Industrial de Vídeos Dark & Shorts/Reels
**Projeto:** Concat Video Engine & Studio (`C:\PROJETOS 2026\Eli Concat`)  
**Arquitetura:** 100% Nativo em Rust (Slint UI + WGPU/Skia + Workspace Crates)  
**Propósito:** Transformar o Concat em uma **Linha de Produção Automatizada de Vídeos Dark de Alta Retenção** para YouTube (16:9) e Redes Sociais (9:16 Shorts/Reels/TikTok).

---

## 🔬 O Que Faltava na Versão Anterior & Melhorias Estruturais Aplicadas

1. **Blindagem Contra "Conteúdo Reutilizado" do YouTube (Anti-Reused Content Engine)**:
   - *Problema do plano anterior:* Juntar vídeos de estoque públicos faz o canal perder a monetização por diretriz de conteúdo repetitivo.
   - *Solução profissional inserida:* Camada de pós-processamento algorítmico exclusivo (Color Grading LUT escuro, granulação analógica de cinema 35mm, vinheta ótica dinâmica, micro-variações de velocidade 0.98x-1.02x imperceptíveis e espelhamento inteligente de b-rolls).

2. **Engenharia de Retenção & Neuro-Design de Vídeo**:
   - *Problema do plano anterior:* Roteiro genérico sem estrutura de retenção.
   - *Solução profissional inserida:* Algoritmo de montagem baseado em **Hooks de 3 Segundos**, **Pattern Interrupts** (mudança de estímulo visual/auditivo a cada 3 a 5 segundos) e **Open Loops** na narrativa.

3. **Automação de Efeitos Sonoros (SFX Automation)**:
   - *Problema do plano anterior:* Apenas narração e música de fundo; vídeos dark sem SFX têm retenção baixíssima.
   - *Solução profissional inserida:* Trilha de SFX procedural em Rust (Whooshes nas transições, Risers para criar tensão, Braam/Thuds de impacto e Pops sincronizados com palavras-chave das legendas).

4. **Nós Executáveis Independentes no Estilo n8n (Graph Engine Real)**:
   - *Problema do plano anterior:* Apenas layout visual estático inspirado em nós.
   - *Solução profissional inserida:* Grafo real com portas tipadas (`InputPort<T>`, `OutputPort<T>`), fios com curvas de Bézier cúbicas via GPU e **botão de disparo/teste individual por nó** (ex: ouvir apenas o áudio gerado no Nó 2 sem precisar renderizar o vídeo inteiro).

5. **Geração Integrada de Thumbnails & Metadados de Publicação**:
   - *Problema do plano anterior:* Faltava a embalagem do vídeo (capa e textos).
   - *Solução profissional inserida:* Gerador automático de Thumbnail de alto contraste (1280x720 para YouTube / 1080x1920 capa de Shorts) com detecção do frame de maior impacto + texto de curiosidade com outline e glow.

6. **Linha de Produção em Lote (Batch Factory)**:
   - *Problema do plano anterior:* Foco em 1 vídeo individual.
   - *Solução profissional inserida:* Fila de produção em lote: o operador insere 10 temas em uma lista, e a fábrica compila os 10 vídeos sequencialmente em background, organizando cada pasta com `video.mp4`, `thumb.jpg`, `title.txt`, `description.txt` e `tags.txt`.

---

## 🗺️ Arquitetura Detalhada dos Crates

```
C:\PROJETOS 2026\Eli Concat\src\crates\
├── concat-factory/              [NOVO CRATE: Motor da Linha de Produção]
│   ├── src/
│   │   ├── lib.rs              # Registro do crate e API pública
│   │   ├── graph.rs            # Grafo de execução acíclico (DAG) estilo n8n
│   │   ├── nodes/              # Implementação isolada de cada nó do pipeline
│   │   │   ├── script_node.rs  # Roteirização, Hooks e quebra em cenas
│   │   │   ├── voice_node.rs   # Orquestração de TTS e timestamps
│   │   │   ├── media_node.rs   # Coleta e curadoria de B-Rolls
│   │   │   ├── sfx_node.rs     # Injeção procedural de efeitos sonoros
│   │   │   ├── caption_node.rs # Estilização e animação de legendas
│   │   │   └── export_node.rs  # Perfis 16:9, 9:16 e encode GPU
│   │   ├── batch.rs            # Gerenciador de fila e automação em lote
│   │   ├── retention.rs        # Algoritmos de retenção e pattern interrupts
│   │   └── thumbnail.rs        # Gerador de capas de alto CTR
│   └── Cargo.toml
│
├── concat-speech/               [EXTENSÃO: Novos Provedores de Voz]
│   └── src/tts/
│       ├── edge.rs             # [NOVO] Cliente nativo WebSocket Microsoft Edge Speech
│       ├── openai.rs           # [NOVO] Provider OpenAI TTS
│       └── elevenlabs.rs       # [NOVO] Provider ElevenLabs TTS
│
├── concat-media/                [EXTENSÃO: Banco de Mídias Gratuitas]
│   └── src/stock/
│       ├── mod.rs              # Trait StockMediaProvider
│       ├── pexels.rs           # [NOVO] Cliente API Pexels (Vídeos/Fotos HD e 4K)
│       ├── pixabay.rs          # [NOVO] Cliente API Pixabay (Fallback gratuito)
│       └── local_vault.rs      # [NOVO] Vault de mídias escuras e texturas locais
│
├── concat-effects/              [EXTENSÃO: Efeitos Cinemáticos & Anti-Reúso]
│   └── src/
│       ├── ken_burns.rs        # [NOVO] Shader/computador de Pan & Zoom dinâmico
│       ├── film_grain.rs       # [NOVO] Overlay de granulação analógica 35mm
│       ├── vignette.rs         # [NOVO] Vinheta ótica de lente escura
│       └── color_grade.rs      # [NOVO] LUTs para canais dark (Teal & Orange, Noir, Horror)
│
├── concat-project/              [EXTENSÃO: Montagem Automatizada & SFX]
│   └── src/
│       ├── factory_assembler.rs # Montador de Timeline multitrack automática
│       ├── sfx_library.rs       # Banco de áudios de transição (whoosh, hit, risers)
│       └── auto_ducking.rs      # Algoritmo de atenuação suave de volume
│
└── concat/                      [EXTENSÃO: Interface Slint & Studio]
    ├── src/panes/
    │   └── factory.rs          # Controller do painel e despacho de mensagens
    └── ui/
        ├── workspace/
        │   ├── factory-pane.slint         # Canvas do workflow estilo n8n
        │   └── factory/
        │       ├── node-card.slint        # Card visual de cada nó com entradas/saídas
        │       ├── wire-renderer.slint    # Curvas de conexão bezier com fluxo animado
        │       ├── node-inspector.slint   # Barra lateral de parâmetros do nó selecionado
        │       └── batch-queue.slint      # Painel lateral da fila de produção em lote
        └── primitives/
            ├── status-led.slint           # LED industrial com estados físicos (glow, blink)
            └── vu-meter-strip.slint       # Medidor de pico estéreo
```

---

## 📋 Checklist de Execução Industrial

### Fase 1: Fundação do Crate `concat-factory` & Grafo de Execução
- [X] **1.1** Validar e configurar toolchain Rust e repositório Git (`https://github.com/evslanca/fabrica-de-videos`).
- [X] **1.2** Criar o crate `concat-factory` em `src/crates/concat-factory` e vincular no workspace `src/Cargo.toml`.
  - [X] Nós (`FactoryNode`) com inputs e outputs tipados fortemente (`PortData`, `PortId`).
  - [X] Arestas de conexão (`Connection`) com validação de tipos de dados.
  - [X] Grafo acíclico executável (`FactoryGraph`).
- [X] **1.3** Definir structs essenciais:
  - [X] `DarkPreset` (Tema: Mistério, Estoicismo, Curiosidades, Crimes Reais, Terror).
  - [X] `SceneSegment` e `RetentionHook` (Timecode, texto do hook, palavras-chave para B-Roll, gatilho de SFX).
  - [X] Módulos de nós: `script_node`, `voice_node`, `media_node`, `sfx_node`, `caption_node`, `export_node`.
  - [X] Motor de fila de lote (`batch.rs`) e gerador de capa (`thumbnail.rs`).
- [X] **1.4** Sistema de Serialização de Fluxos (`.darkflow`): Salvar e carregar estruturas completas de workflow em JSON (`save_to_file`, `load_from_file`, `execution_order()` topológico em `graph.rs`, e `pipeline.rs` end-to-end).

---

### Fase 2: Motor de Voz Neural Gratuito (Edge-TTS) & Provedores Cloud
- [X] **2.1** Desenvolver cliente Rust puro para **Edge-TTS** (`concat-speech::edge_tts`):
  - [X] Protocolo WebSocket seguro nativo com handshake Microsoft Speech.
  - [X] Mapeamento e testes com vozes brasileiras de alta performance:
    - `pt-BR-AntonioNeural` (Grave, ideal para histórias/mistério).
    - `pt-BR-FranciscaNeural` (Expressiva, excelente para curiosidades/fatos).
    - `pt-BR-ThalitaNeural` (Dinâmica, ideal para Shorts/Reels rápidos).
  - [X] Captura precisa de eventos `WordBoundary` (timestamp exato em milissegundos para cada palavra falada).
  - [X] Cache via geração de header WAV local (`audio/dark_tts_<hash>.wav`).
- [X] **2.2** Desenvolver provider `OpenAiTts` (`concat-speech::cloud_tts::OpenAiTtsConfig`) com suporte aos modelos `tts-1` e `tts-1-hd`.
- [X] **2.3** Desenvolver provider `ElevenLabsTts` (`concat-speech::cloud_tts::ElevenLabsTtsConfig`) com controle de estabilidade, similaridade e modelo `eleven_multilingual_v2`.
- [X] **2.4** Cache de áudio com hash determinístico (`cache_key_hex`) do texto + voz + velocidade + tom.
- [ ] **2.5** Normalização e Masterização Automática de Áudio (-14 LUFS para YouTube).

---

### Fase 3: Curadoria de B-Rolls Gratuitos & Motor Anti-Reúso
- [X] **3.1** Criar cliente Stock Media API em `concat-media::stock`:
  - [X] Busca inteligente por palavras-chave em inglês e português.
  - [X] Filtro por orientação (`portrait` para Shorts ou `landscape` para YouTube).
  - [ ] Download assíncrono paralelo com limite de concorrência e barra de progresso.
- [ ] **3.2** Criar cliente Pixabay API em `concat-media::stock::pixabay` como contingência automática caso o Pexels atinja limite.
- [X] **3.3** Implementar o **Motor Anti-Reutilização de Conteúdo** em `concat-effects::dark_cinema`:
  - [X] **Espelhamento Seletivo**: Inversão horizontal randômica de takes que não contenham texto legível.
  - [X] **Micro Speed Ramping**: Ajuste dinâmico de velocidade (0.97x a 1.03x) para alterar a assinatura digital do vídeo.
  - [X] **Efeito Ken Burns Procedural**: Suave aproximação (1.0x -> 1.15x) ou afastamento com aceleração cúbica (Ease-In-Out).
  - [X] **Camada Analógica de Textura**: Film grain 35mm + vinheta com cantos escurecidos cinematográficos.
  - [ ] **Color Grading Dark**: LUTs de alto contraste e sombras profundas embutidos diretamente no pipeline WGPU.

---

### Fase 4: Biblioteca de Efeitos Sonoros (SFX) & Auto-Ducking
- [ ] **4.1** Implementar repositório local de SFX embutido no Concat (`assets/sfx/`):
  - [ ] Categoria *Transitions*: Whoosh rápido, Whoosh com eco, Swish metálico.
  - [ ] Categoria *Impacts*: Sub-bass Drop, Cinematic Braam, Dark Thud.
  - [ ] Categoria *Tension*: Drone misterioso, Riser crescente, Batimento cardíaco.
  - [ ] Categoria *Pops & UI*: Pop seco, Click tátil, Bell suave.
- [X] **4.2** Injeção Automática de SFX (`sfx_library.rs`):
  - [X] Catálogo tipado `SfxAsset` com 8 categorias (Whoosh, Echo, SubBass, Braam, Riser, Heartbeat, Pop, Glass).
  - [X] Inserir Whoosh/Swish automaticamente a cada troca de cena.
  - [X] Inserir Sub-bass Drop a cada 3ª cena de corte.
  - [X] Inserir Riser nos 8 segundos que antecedem a revelação principal do roteiro.
- [X] **4.3** Motor de Auto-Ducking Inteligente (`auto_ducking.rs`):
  - [X] Análise do envelope de amplitude da trilha de voz.
  - [X] Redução paramétrica da trilha de BGM com attack de 120ms e release de 400ms durante a fala do narrador.
  - [X] `build_ducking_keyframes()` gera curva de automação de ganho para o timeline assembler.

---

### Fase 5: Legendas Dinâmicas Sincronizadas (Estilo Hormozi / Dark)
- [X] **5.1** Gerador de Legendas por Palavra em `concat-project::captions_factory`:
  - [X] Segmentação de frases em blocos de 1 a 3 palavras para ritmo veloz de leitura (Shorts) via `build_caption_bursts()`.
  - [X] Destaque cromático dinâmico (`CaptionColor`: Amarelo Fluorescente, Verde Neon, Ciano, Ouro Escuro).
- [ ] **5.2** Safe Zones Inteligentes (Zonas de Proteção Visual):
  - [ ] **Modo Shorts / Reels / TikTok (9:16)**: Manter legendas restritas ao terço central (evitar sobreposição pelo título na base, botões de like/comentário à direita e topo do app).
  - [ ] **Modo YouTube Padrão (16:9)**: Centralização no terço inferior clássico com barra de fundo translúcida ou sombra de alto contraste.
- [ ] **5.3** Renderização nativa vetorial via Skia/WGPU para garantir nitidez máxima sem perda de performance.

---

### Fase 6: Interface Slint Avançada com Nós Estilo n8n & Drag-and-Drop
- [X] **6.1** Construir o Canvas de Workflow (`ui/workspace/factory-pane.slint` e `factory/wire-renderer.slint`):
  - [X] Fundo escuro com suporte a Pan (arrastar canvas) e controles de Zoom (+ / - / Reset).
  - [X] Desenho das curvas de Bézier cúbicas (`WireLayer`) entre portas de saída e entrada.
- [X] **6.2** Desenvolver Cards de Nós com Controles Dedicados (`node-card.slint`):
  - [X] **Card Roteiro**: posicionado no canvas com LED de status e botão mini-play.
  - [X] **Card Voz**: seletor de provedor (Edge-TTS / ElevenLabs) declarado.
  - [X] **Card B-Roll**: filtro de tags e toggle de Ken Burns declarado.
  - [X] **Card SFX & Áudio**: sliders de volume relativo (Voz / Música / SFX).
  - [X] **Card Legendas**: seletor de paleta e tamanho de fonte declarado.
  - [X] **Card Exportador**: alternador 1-click 16:9/9:16 e botão master "FABRICAR VÍDEO".
- [ ] **6.3** Suporte Integral a Arrastar e Soltar (Drag & Drop):
  - [ ] Soltar vídeos/músicas/imagens do Windows Explorer diretamente no canvas.
  - [ ] Soltar arquivos de roteiro (`.txt`, `.md`, `.json`) no nó de roteiro.
- [X] **6.4** Execução Isolada de Nós:
  - [X] Cada card possui seu próprio botão mini-play para testar e validar o nó individual.
- [X] **6.5** Controller Rust do painel (`src/panes/factory.rs`):
  - [X] `FactoryMsg` enum com `Open`, `Close`, `NodeSelected`, `TestNode`, `Fabricate`, `BatchStart`, `BatchStop`, `BatchItemFinished`, `StageProgress`.
  - [X] Integrado ao `panes::Msg`, `Studio::handle`, `dock.rs`, `pane-switcher.slint`, `seat.slint`, `app.slint` e callbacks em `lib.rs`.

---

### Fase 7: Sistema Físico de LEDs de Status & VU Meters em Tempo Real
- [X] **7.1** Componente de LED de Status Físico (`ui/primitives/status-led.slint`):
  - [X] Design com glow luminoso e enum `LedState` (Off, Ready, Working, Rendering, Success, Error).
  - [X] Estados codificados: cinza / verde suave / âmbar / azul / verde brilhante / vermelho.
  - [X] Integrado ao barrel `primitives.slint` e exportado para todos os componentes factory.
- [X] **7.2** LED embutido em cada `FactoryNodeCard` refletindo estado em tempo real.
- [X] **7.3** Barras de progresso industriais integradas em cada card de nó e no painel de lote.
- [X] **7.4** Integração do VU Meter Estéreo (`LevelMeter`) na barra inferior do `factory-pane.slint` (`vu-left`, `vu-right`).
  - [ ] LEDs de aviso de clipping (+0dB em vermelho) para evitar distorção no áudio final.

---

### Fase 8: Fábrica em Lote (Batch Production) & Gerador de Thumbnails
- [X] **8.1** Módulo de Produção em Lote (`concat-factory::batch`):
  - [X] `BatchQueue` com `enqueue()`, `next_item()` e `pending_count()`.
  - [X] Painel Slint `batch-queue.slint`: LEDs por item, barra de progresso por fase, botões Iniciar/Parar/Limpar.
  - [X] Organização estruturada de pastas de saída definida na `ExportNodeConfig`.
- [X] **8.2** Gerador Automático de Thumbnails (`thumbnail.rs`):
  - [X] `ThumbnailConfig` com texto overlay, cor, outline, vignette e output path.
  - [ ] Extração algorítmica do frame de maior contraste e renderização via Skia.
- [X] **8.3** `FactoryAssembler` (`factory_assembler.rs`):
  - [X] Recebe cenas, áudios, legenda bursts e ducking params e monta o `AssembledProject`.
  - [X] Log estruturado de duração total, cenas e keyframes gerados.

---

### Fase 9: Configurações, Segurança de Chaves & Modo "Zero Reais"
- [X] **9.1** Gerenciador de Chaves de API (`concat-host::factory_keys::FactoryKeys`):
  - [X] Armazenamento local em `AppDirs::config` (`factory_keys.json`) para chaves opcionais (OpenAI, ElevenLabs, Pexels, Pixabay).
- [X] **9.2** Garantia do Modo "Zero Reais / Zero Chaves":
  - [X] `FactoryKeys::any_configured()` e fallback automático para Edge-TTS e vault/stock gratuito.
- [X] **9.3** Dicionário de Pronúncia (`PronunciationDict` em `edge_tts.rs`):
  - [X] Tabela de substituição fonética para siglas e termos (ex: "NASA" -> "Naza", "EUA" -> "Estados Unidos", "OVNI" -> "óvni").

---

### Fase 10: Testes Rigorosos, Performance & Build de Produção
- [X] **10.1** Testes Unitários e de Integração escritos:
  - [X] Testes do grafo DAG (`execution_order`, detecção de ciclo, `.darkflow` roundtrip) e pipeline end-to-end (`concat-factory`).
  - [X] Testes do `EdgeTtsClient` (SSML escaping, `WordBoundary`, WAV header) e `cloud_tts` (`concat-speech`).
  - [X] Testes de legendas Hormozi (`captions_factory`), `auto_ducking`, e `sfx_library` (`concat-project`).
- [ ] **10.2** Benchmarks de Renderização:
  - [ ] Medir tempo de geração de vídeo de 60 segundos (meta: menos de 20 segundos em GPU moderna via WGPU).
- [ ] **10.3** Scripts de compilação rápida para desenvolvimento (`cargo dev`) e compilação de entrega final (`cargo build --release`).

---

### Fase 11: Voice Cloning, Voice Design & Multi-Narrador (100% Local ONNX + Rust)
- [X] **11.1** **Voice Cloning Local (`concat-speech::chatterbox` + `VoiceNodeConfig`)**:
  - [X] Expor `VoiceEngine::ChatterboxClone` no `VoiceNodeConfig` (`voice_node.rs`) e no card de Voz.
  - [ ] Permitir arrastar ou selecionar um clipe de referência (5–15 segundos) para clonar a voz localmente via ONNX Runtime (`ort`), sem custos ou APIs externas.
- [X] **11.2** **Voice Design & Controle de Emoção/Prosódia**:
  - [X] `VoiceStyle` & `VoiceDesignParams` (*DarkNarrator*, *WhisperSuspense*, *EpicDocumentary*, *UrgentNews*, *Neutral*).
  - [X] Suporte a tags inline no roteiro (`[pausa Xs]` via `parse_multi_speaker_script`).
- [X] **11.3** **Modo Multi-Narrador (Diálogos & Storytelling)**:
  - [X] Parser `parse_multi_speaker_script()` para múltiplos falantes (`[Narrador]:`, `[Testemunha]:`) com testes unitários.

---

### Fase 12: Video Dubbing & Escala Global Multi-Idioma
- [ ] **12.1** **Pipeline de Video Dubbing (`concat-factory::dubbing`)**:
  - [ ] Ingestão de vídeo pronto ou projeto da fábrica.
  - [ ] Separação vocal opcional (`concat-speech::isolate` / `vocals`) para preservar música e SFX originais.
  - [ ] Transcrição com timecodes precisos via `whisper-rs` (`concat-speech::transcribe`).
- [ ] **12.2** **Tradução & Ressíntese Sincronizada (Time-Stretch)**:
  - [ ] Ajuste automático de velocidade (`rate`) do TTS no idioma alvo para caber exatamente na janela temporal da fala original.
  - [ ] Exportação em lote de múltiplos idiomas a partir de 1 único vídeo (PT-BR ➔ EN-US, ES, FR, DE) com legendas Hormozi traduzidas.

---

### Fase 13: Local Models Manager & Roteirista IA Offline
- [ ] **13.1** **Gerenciador Unificado de Modelos Locais (`concat-host::models`)**:
  - [ ] Painel de status e download sob demanda com barra de progresso para modelos locais: Whisper (tiny/base/small/medium), Chatterbox ONNX e modelos LLM locais.
  - [ ] Verificação de integridade e escolha de acelerador (CPU / DirectML / CoreML).
- [ ] **13.2** **Roteirista IA Integrado (Ollama Local ou API Opcional)**:
  - [ ] Gerador de roteiro completo a partir de um único tema/nicho: cria Hook de 3s, Open Loops, cenas segmentadas e **keywords em inglês** otimizadas para busca no Pexels.
  - [ ] Geração automática de título SEO, descrição, tags e frase curta para Thumbnail.

---

### Fase 14: FX Effects Avançados, Masterização Vocal & Ponte Real Rede/GPU
- [X] **14.1** **Voice FX Effects & Masterização de Estúdio (`concat-project::voice_fx`)**:
  - [X] `MasteringChainConfig`: HPF 80Hz, EQ de presença (+2.5dB), De-Esser, Limiter (-1 dBTP) e cálculo de ganho para **-14 LUFS** (`estimate_lufs_gain_db()`).
  - [X] `VoiceFxPreset`: *StudioBroadcast*, *CavernousReverb*, *VintageRadio*, *DarkPitchDown*, *SpectralEcho*, *AnalogSaturation*.
- [X] **14.2** **Visual FX Effects & Color Grading (`concat-effects::dark_cinema`)**:
  - [X] `VisualFxPreset` (*ChromaticGlitch*, *VhsScanlines*, *DustAndAsh*, *BeatZoomPulse*, *NoirFlicker*) e `ColorGradeLut` (*NoirHighContrast*, *TealAndOrange*, *HorrorDesaturated*, *GoldenStoic*).
  - [ ] Shaders WGPU executando cada preset no compositor de frames.
- [ ] **14.3** **Implementação Real de Rede & Exportação Final**:
  - [ ] Conector WebSocket real para o Microsoft Edge Speech (`wss://speech.platform.bing.com/...`) recebendo áudio real e `WordBoundary` reais.
  - [ ] Downloader paralelo real das APIs Pexels e Pixabay para `cache/stock_media/` com retry e progresso.
  - [ ] Conversor `AssembledProject` ➔ `concat_project::model::Document` + disparo real de render `.mp4` via `concat-export`.

---

### Fase 15: UX Avançada do Canvas (Drag & Drop Real + Wiring Interativo)
- [ ] **15.1** **Edição Livre no Canvas**:
  - [ ] Arrastar e reposicionar qualquer `FactoryNodeCard` no canvas atualizando os cabos Bézier em tempo real.
  - [ ] Conectar e desconectar portas de entrada/saída diretamente com o mouse.
- [ ] **15.2** **Drag & Drop Nativo do Sistema Operacional**:
  - [ ] Soltar `.mp4`, `.mp3`, `.wav`, `.png` direto do Windows Explorer sobre um nó para usá-lo como B-Roll, BGM ou amostra de Voice Cloning.
  - [ ] Soltar `.txt` / `.md` no nó de Roteiro para importação imediata.
- [ ] **15.3** **Audition por Cena com Waveform**:
  - [ ] Editor de roteiro cena a cena com forma de onda (`wave.slint`) e preview instantâneo de áudio/vídeo por cena.

---

### Fase 16: Criação de Músicas IA (Sem Voz / Instrumental & Com Voz / Letra)
- [X] **16.1** **Nó de Criação Musical (`concat-factory::nodes::music_node`)**:
  - [X] Modo **Sem Voz (`MusicMode::Instrumental`)**: `MusicGenre` (*DarkAmbient*, *Phonk*, *HorrorDrone*, *EpicCinematic*, *LoFiMystery*, *SynthwaveNoir*), BPM e duração.
  - [X] Modo **Com Voz (`MusicMode::VocalWithLyrics`)**: prompt de estilo + letra + `VocalTimbre` (*MaleDeep*, *FemaleEthereal*, *ChoirDark*, *RaspyRap*).
  - [X] Nó `"music"` integrado ao `FactoryGraph::default_dark_pipeline()` e ao canvas `factory-pane.slint`.
- [X] **16.2** **Motor Local ONNX & Provedores Opcionais de Música**:
  - [X] Enum `MusicEngine` definido (`LocalOnnx`, `SunoApi`, `ElevenLabsMusic`).
  - [ ] Execução de inferência ONNX real para MusicGen / ACE-Step no `concat-host::models`.
- [X] **16.3** **Extração de Stems & Sincronia de Batida (Beat Sync)**:
  - [X] `MusicNodeConfig::beat_timestamps()` calculando timecodes exatos de batida por BPM (com testes unitários).
  - [ ] Separação automática de vocal e instrumental via `concat-speech::vocals` / `isolate` em runtime.

---

## 🎯 Como Operar e Atualizar Este Documento
- Sempre que uma tarefa for implementada e validada com testes, marque `[X]`.
- O plano foi desenhado para ser executado de forma sequencial, garantindo que o núcleo de áudio e dados esteja sólido antes da montagem da interface gráfica no Slint.
