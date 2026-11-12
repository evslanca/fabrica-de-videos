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
- [ ] **1.4** Sistema de Serialização de Fluxos (`.darkflow`): Salvar e carregar estruturas completas de workflow em JSON.

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
- [ ] **2.2** Desenvolver provider `OpenAiTts` (`concat-speech::tts::openai`) com suporte aos modelos `tts-1` e `tts-1-hd`.
- [ ] **2.3** Desenvolver provider `ElevenLabsTts` (`concat-speech::tts::elevenlabs`) com controle de estabilidade, similaridade e modelo `eleven_multilingual_v2`.
- [ ] **2.4** Cache de áudio com hash criptográfico SHA-256 do texto + voz + velocidade.
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
- [ ] **5.1** Gerador de Legendas por Palavra em `concat-project`:
  - [ ] Segmentação de frases em blocos de 1 a 3 palavras para ritmo veloz de leitura (Shorts).
  - [ ] Destaque cromático dinâmico (a palavra pronunciada no instante exato acende em Amarelo Fluorescente, Verde Limão ou Ciano, enquanto as adjacentes permanecem em Branco/Cinza).
- [ ] **5.2** Safe Zones Inteligentes (Zonas de Proteção Visual):
  - [ ] **Modo Shorts / Reels / TikTok (9:16)**: Manter legendas restritas ao terço central (evitar sobreposição pelo título na base, botões de like/comentário à direita e topo do app).
  - [ ] **Modo YouTube Padrão (16:9)**: Centralização no terço inferior clássico com barra de fundo translúcida ou sombra de alto contraste.
- [ ] **5.3** Renderização nativa vetorial via Skia/WGPU para garantir nitidez máxima sem perda de performance.

---

### Fase 6: Interface Slint Avançada com Nós Estilo n8n & Drag-and-Drop
- [X] **6.1** Construir o Canvas de Workflow (`ui/workspace/factory-pane.slint`):
  - [X] Fundo infinito com grid pontilhado industrial.
  - [ ] Suporte a Pan (arrastar com botão direito/meio) e Zoom suave.
  - [X] Desenho das curvas de Bézier entre portas (estrutura SVG/wgpu definida).
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
  - [X] Integrado ao `panes::Msg` e ao `panes/mod.rs`.

---

### Fase 7: Sistema Físico de LEDs de Status & VU Meters em Tempo Real
- [X] **7.1** Componente de LED de Status Físico (`ui/primitives/status-led.slint`):
  - [X] Design com glow luminoso e enum `LedState` (Off, Ready, Working, Rendering, Success, Error).
  - [X] Estados codificados: cinza / verde suave / âmbar / azul / verde brilhante / vermelho.
  - [X] Integrado ao barrel `primitives.slint` e exportado para todos os componentes factory.
- [X] **7.2** LED embutido em cada `FactoryNodeCard` refletindo estado em tempo real.
- [X] **7.3** Barras de progresso industriais integradas em cada card de nó e no painel de lote.
- [ ] **7.4** Integração do VU Meter Estéreo ao player de pré-visualização da fábrica.
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
- [ ] **9.1** Gerenciador de Chaves de API (`concat-host::AppDirs`):
  - [ ] Armazenamento local protegido para chaves opcionais (OpenAI, ElevenLabs, Pexels API Key pessoal).
- [ ] **9.2** Garantia do Modo "Zero Reais / Zero Chaves":
  - [ ] Validação estrita: se nenhuma chave for inserida, o aplicativo opera com 100% de suas funções ativas através de Edge-TTS, mídias públicas e biblioteca local de SFX.
- [ ] **9.3** Seletor de Idioma e Dicionário de Pronúncia:
  - [ ] Tabela de substituição fonética para siglas ou palavras difíceis (ex: "NASA" -> "Naza", "EUA" -> "Estados Unidos").

---

### Fase 10: Testes Rigorosos, Performance & Build de Produção
- [ ] **10.1** Testes Unitários e de Integração:
  - [ ] Teste de conexão e síntese de áudio Edge-TTS em múltiplos idiomas.
  - [ ] Teste de consistência do cálculo de alinhamento das legendas Hormozi.
  - [ ] Teste do pipeline de auto-ducking com medição de atenuação de decibéis.
- [ ] **10.2** Benchmarks de Renderização:
  - [ ] Medir tempo de geração de vídeo de 60 segundos (meta: menos de 20 segundos em GPU moderna via WGPU).
- [ ] **10.3** Scripts de compilação rápida para desenvolvimento (`cargo dev`) e compilação de entrega final (`cargo build --release`).

---

## 🎯 Como Operar e Atualizar Este Documento
- Sempre que uma tarefa for implementada e validada com testes, marque `[X]`.
- O plano foi desenhado para ser executado de forma sequencial, garantindo que o núcleo de áudio e dados esteja sólido antes da montagem da interface gráfica no Slint.
