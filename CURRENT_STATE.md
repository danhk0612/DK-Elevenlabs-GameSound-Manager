# 현재 상태

## v0.10.0 · Windows x64 배포 기준

2026-11-27 사용자가 현재 구현 버전 배포를 승인했다. 릴리스 대상은 GUI·엔진·CLI·MCP와 사용 설명서, 도구 스키마, 자동화 예제를 포함한 ZIP이다. FFmpeg와 사용자 음원·자격증명은 별도다. main 릴리스 CI에서 Windows·Linux 검증과 체크섬 대조 후 [릴리스](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/releases/tag/v0.10.0)에 게시한다. Windows 11 청감, AI 제품별 검증, T05는 남아 있다.

## 2026-11-26 · AI 자동화와 Windows 라이브러리

Windows 11 남색 Fluent 화면에서 프로젝트 생성과 원본 가져오기가 기존 공통 엔진을 호출한다. Windows 패키지에 START_HERE.md와 빌드된 CLI의 tools.json을 포함하도록 구성했다. Linux에서 TypeScript/Vite 빌드와 기존 Python 회귀 27개 통과. Chromium의 모의 IPC로 생성·등록 요청과 세 화면 전환, 390·660·1040px 레이아웃을 확인했다. 이전 소스 82d8b6f의 Windows/Linux run 44846395069는 성공으로 확인했다.

제품·검증 SHA `cbb6bef40938164404091e8ae2ef23380d47eb08`의 [Windows/Linux 검증 45843456108](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/actions/runs/45843456108) 두 job 성공. Windows Server 2022 x64 WebView2 실제 창에서 프로젝트 생성 → WAV 등록 → 재생 종료, 독립 CLI에서 동일 SHA-256, 세 화면 전환을 확인했다. Linux headless·배치 회귀도 성공했다. [Windows 시험 ZIP](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/actions/runs/45843456108/artifacts/15130970121)은 2026-12-10까지 보관된다. 시험 ZIP 크기 7,881,712바이트, SHA-256 `29c12f1c6850c006acc2bcdd3a9f2791b0c068c583d85bcea4cf53804129dceb`. 사용자 Windows 11 청감·정식 배포는 미검증이다.

## 최신 — 월 10,000크레딧/음악·짧은 음성 (2026-11-21)

확장 코드 source `82d8b6f1212ae99e27b0bde38c58f63864527236`, [Windows/Linux CI 44846395069](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/actions/runs/44846395069)는 진행 중이다. 문서 후속 커밋은 실행 source와 구분한다. Linux musl/Rust 1.100.0/Zig·FFmpeg 7.0.2/Python 3.14 환경에서 cargo test --lib -- --include-ignored 45개 통과, HTTP status 보존 보완 후 cargo test --lib 38개 통과/외부 도구 7개 ignore, Python unittest discover 27개 통과, native bins·Windows x64 no-default-features 타입 검사·ci_transport_smoke.py --local-linux/CLI·MCP parity·fmt/diff를 통과했다. GUI 변경/사이트 build는 없다.

최종 source의 새 엔진을 자격 증명 없이 자동 기동해 기존 7개 생성 job을 동일 요청으로 재제출했다. 성공 6개/음악 실패 1개를 그대로 반환하고 ledger 8회/예약 1,250을 유지했다. 추가 POST/실패 재생성 없이 재연결을 확인했으며 엔진은 종료했다. 이후 새 생성에는 원래 NAS config 세션을 다시 열어야 한다.

[계약/실제 결과](docs/T02_MUSIC_SHORT_VOICES.md). job.submit music_v1/명시 instrumental·길이/MP3·bgm 후보를 기존 예산/단일 POST/공통 CLI·MCP에 추가했다. 실패 HTTP status 숫자 보존과 과금 없는 sound_starter.py 전체 사전 검사/선택/Voice 타입·대사 연결/예약 합계를 제공한다. 초기 8항목 예약 1,800/계획 상한 2,000이며 사용자 설정은 수정하지 않았다. 새 GUI·DB schema·명령 수 변경은 없다.

실제 TTS 6개 성공/header 비용 합계 15·0.601~0.914초·전체 decode·review pending/current match를 확인했다. NAS Git 제외 user-data/starter-pack의 listen.html에서 재생할 수 있다. 음악 첫 POST는 provider_http_error로 실패했고 status는 당시 미보존/원인 미확인이다. 재시도/두 번째 곡 생성 없이 예약 750 유지. 기존 SFX까지 누적 ledger 8회/보수 예약 1,250이며 실제 잔액/최종 청구액과 다르다. 서비스 usage 반영은 지연 중이다. 음악 실제 생성 성공/청감·게임 적용/T05는 미완료다.

로컬 Linux Rust 45개(실제 FFmpeg 7개 포함)·Python 27개와 native bins/Windows x64 타입 검사를 확인했다. 마지막 HTTP status 보존 회귀를 포함한 최종 코드/CI 증거는 후속 SHA 기록과 대조한다. 기반 source 1c21d8e640f97364bc3e1767c0befd08e65aa892/run 44842948393은 Windows/Linux 전체 성공으로 확인했다. 현재 확장 코드의 정확한 SHA/새 CI는 별도 기록한다.

기준일: 2026-11-21 UTC. **v0.10.0 Linux 실제 인증·SFX 생성 1회/디코딩·재연결 확인. 최신 결과는 아래 실제 생성 절을 따르며 이전 미검증 문장은 당시 상태다. 최종 과금 정산/청감은 미확인이다.**

## 저장소와 범위

main `4c3537c55084b71feb5c42eb2340005bf922f197`, 작업 기반 `06379078ed1227c08c952889ab8038681f16e066`에서 기존 task/t00-a-technical-spike와 [PR #1](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/pull/1)을 재사용했다. PR open/non-draft를 유지하고 직접 병합하지 않는다. 최신 코드 SHA/CI와 후속 문서 SHA는 아래 증거 및 Git/PR source SHA로 구분한다.

사용자는 현재 AI가 쓸 수 있는 상태를 요구하며 Linux 인증/통신 구현에 “진행해”라고 지시했다. D01~D06의 Windows 제품 선택은 유지하고 Linux headless 경로를 확장한다. D08 기본 출력/검사·D09 기본 한도 수치는 미결정이며 명시 입력을 요구한다. 실기·청감 대기로 개발을 막거나 미확인을 성공으로 기록하지 않는다.

## 구현

- 공통 47개 명령/CLI JSON/MCP 46개 도구·catalog schema 5/프로젝트 DB schema 1이다. Windows SID별 owner pipe·자동 기동·busy shutdown/5분 idle과 GUI는 유지한다. Linux는 private DK_SOUND_ROOT·UID/0700·flock/0600 Unix socket·peer UID·기동 경쟁/잔여 socket/별도 process group을 사용한다.
- 공통 storage worker/queue/fast 조회·취소, 원본/후보·계보·해시·출처·no-replace 게시·중단 대조, mock/ElevenLabs 예산/queue·단일 POST·불확실 비용 보존·output_ready 등록 복구를 제공한다. [기반 계약](app/README.md).
- 명시한 FFmpeg/ffprobe로 별도 후보 편집·분석/파형/loop 수치, async 예약·진행/취소/최종 경계·중단 무재생, 검토 불변 snapshot/pending·withdrawn/현재 일치·후보 이력을 제공한다. 검토 제출은 승인이 아니다.
- 선택형 Python 단일/다건 작업 정의·전체 offline preflight/ID 충돌·JSON/JSONL·선택 재호출·확정 오류 keep-going을 유지한다. 현재 Codex 쉘에서 Linux native CLI/자체 stdio MCP를 호출할 수 있다. 앱 MCP 등록/타 AI 제품 수용을 확인한 것은 아니다.
- **v0.10 [Linux provider](docs/T02_LINUX_PROVIDER.md):** engine --session 전용 pipe/socket stdin에 명시 계정/key/허용/한도/TTL을 받아 메모리에만 보관한다. 일반 CLI/MCP key·live 정책 변경 명령을 추가하지 않는다. default autostart는 인증 없이 유지한다. TTL 만료/다른 계정/재시작 차단, core/dumpable 제한, Zeroizing 입력/저장 key를 적용한다. 선택형 `automation/linux_session.py`는 숨김 입력/private key FD로 foreground engine을 시작한다. Python/HTTP/TLS 내부 모든 복사·swap 제거까지 보장하지 않는다.
- Linux reqwest 0.13.5/rustls/ring의 고정 api.elevenlabs.io HTTPS, 인증서/호스트 검증, retry/redirect/proxy/Referer/TLS key logging 차단, connect 10초/전체 120초·32 MiB bounded body, 비200 본문 미반환·정적 전송 오류를 구현했다. 공통 workspace identity/명시 예산·POST attempted/outcome_unknown/후보 정책에 연결된다. 실제 청구액 상한 보장이 아니다.
- Windows Credential Manager/WinHTTP·최소 GUI 동작은 유지한다. Linux GUI/secret 저장 서비스·T05 승인/게임 적용은 추가하지 않는다.

## 검증

이전 v0.9 source `32828ea59e73d8b12c525acc7210f73ffbab3114`의 [Windows/Linux run 44834877114](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/actions/runs/44834877114)은 **두 job 모두 성공**했다. 실제 Linux CLI/MCP/배치/FFmpeg·Windows native/GUI·CLI/MCP/패키징 증거이며 새 v0.10의 실행 증거로 확대하지 않는다. 이전 상세 증거는 Git과 각 계약에 보존한다.

v0.10 로컬 Linux x86_64 musl/Rust 1.100.0/Python 3.11+·FFmpeg/ffprobe 7.0.2:

- `DK_TEST_FFMPEG=/tmp/dk-audio-tools/ffmpeg DK_TEST_FFPROBE=/tmp/dk-audio-tools/ffprobe cargo test --locked --manifest-path app/Cargo.toml --lib -- --include-ignored`: **43개 전부 통과**(기존 실제 FFmpeg 7개+신규 Linux 4개 포함). 신규 검사는 세션/TTL/계정/필드·고정 origin/path와 실행 중 생성하는 인증서/가짜 키의 로컬 TLS 200/비용 헤더·401/429·302 무추적·응답 크기·유실 단일 시도·미신뢰 인증서/평문 HTTP 거절이다. 실제 ElevenLabs 요청은 없다.
- `python3 -m unittest discover -s app/automation -p 'test_sound_*.py'`: **18개 통과**.
- `python3 app/automation/ci_linux_session_smoke.py --bin-dir app/target/debug`: 실제 engine stdin 크기/필드 거절·계정 scope·secret/예산 IPC 거절·metadata-only·가짜 키 무출력/무저장·재시작 미설정·private key FD 런처 통과. 실제 HTTP 0회다.
- `ci_linux_smoke.py --bin-dir app/target/debug --ffmpeg /tmp/dk-audio-tools/ffmpeg --ffprobe /tmp/dk-audio-tools/ffprobe`: private root/endpoint 보존·8개 동시 singleton·중복 기동 거절·stale/crash 재기동·mock 예산/ID·실제 CLI/MCP FFmpeg async 편집/검사·검토/원본 보존을 확인한다. 최종 v0.10 native bins에서 전부 통과했다.
- native bins build, Windows x64 no-default-features 타입 검사, Node 24 TypeScript/Vite build, fmt/diff 통과. Windows 타입 검사는 native 링크/Windows 실행 증거가 아니다.

Rust 환경은 /tmp/dk-cargo·/tmp/dk-rustup·Zig CC/AR/rust-lld wrapper(CFLAGS=-fno-sanitize=all), Python /tmp/msv-groups-tools/python3, FFmpeg /tmp/dk-audio-tools다. 도구/합성 인증서·음원·key는 Git에 넣지 않는다. CI Linux job에 실제 credential-session smoke와 로컬 TLS 회귀를 연결했고 Windows job은 기존 native/GUI/CLI·MCP·배치/패키징을 유지한다. v0.10 최초 코드 source `3a5c52ca1a525785a6cc96ffd4e761a04a7cbfff`의 [run 44839019647](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/actions/runs/44839019647)에서 Windows·Linux 두 job 모두 성공했다. 최종 source `54d2fbf2fa5809b251857aac23527743be7afff8`는 private key FD의 허용 prefix 뒤 초과 바이트도 거절하도록 보강하고 실제 metadata-only session smoke를 재통과했다. [run 44839231602](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/actions/runs/44839231602)의 Windows·Linux 두 job도 모두 성공했다. Linux Rust 43개·Python 18개·실제 CLI/MCP/배치/FFmpeg/인증 세션, Windows native/GUI·실제 CLI/MCP/배치·개발 패키징 회귀를 확인했다. 이 CI도 실제 ElevenLabs key/서비스 요청은 사용하지 않았다. 이후 상태 문서 커밋과 실행 source SHA를 구분한다.

## 남은 범위

Linux 실제 API 키·서비스 권한/quota·허용 생성 범위/비용이 안전한 인증 세션으로 준비돼야 실제 GET/생성·결과 파일·과금/청감을 검증할 수 있다. 현재는 구현 및 로컬 합성/모의 검증 완료이며 실제 사운드 제작 성공이 아니다. 키를 채팅/인자/env/평문 파일로 받지 않고 실제 Key 없이 라이브 요청하지 않는다.

T04 전체·제품 완성/T05 착수는 선언하지 않는다. 지원 AI 제품 MCP 등록, 원격 preview 재생·한도 증액/초기화·다른 PC 공유/불확실 비용 정산, Windows 실제 FFmpeg/취소·사용자 Windows 11 화면/청감은 별도다. D08 loop tag/LUFS/true peak·D07 승인/게임 적용은 후속 범위다.

사이트/게임 개발은 계속 일시 정지한다. 병합·Release/tag·공개 배포·실제 Key/크레딧 사용·사용자 음원/게임/사이트/운영 DB·사이트 빌드/배포는 수행하지 않았다. 게임 수정 목록은 사이트 games/02-deck-expedition/POST_AUDIO_BACKLOG.md를 유지한다.

최종 source 54d2fbf2fa5809b251857aac23527743be7afff8의 [Windows 개발 artifact](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/actions/runs/44839231602/artifacts/14679409749)는 LINUX_PROVIDER.md/인증 세션 런처·smoke를 포함한다. GitHub metadata reported SHA-256: `720bffa7b937eeddd461c7271f488fb5a8244e6777e29cb93d83ab5fb52df8f2`. 이 기록은 artifact 다운로드 후 독립 해시 대조와 구분하며 Release/공개 배포가 아니다.

## 최신 사용자 요청 — Windows/NAS 파일 입력

2026-11-21, 사용자 Windows/NAS 파일 편집·Codex CLI Docker 연결 환경에 따라 [명시적인 NAS 설정 파일 입력](docs/WINDOWS_NAS_SETUP.md)을 구현했다. 최상단 elevenlabs.local.json을 key 빈 값/생성 false/한도 0으로 한 번 만들고 Git 제외했다. 이후 사용자 값을 출력/덮어쓰지 않는다. --config는 BOM/CRLF/합성 0777·bounded strict JSON과 --check-config 무기동/무HTTP 검사, Docker-local runtime과 기존 engine stdin 전달을 지원한다. 기존 숨김 입력/private FD·Windows Credential Manager/엔진 규칙/schema/GUI는 유지한다.

Linux Python 22개(기존 18+설정 4)와 실제 engine metadata-only session smoke의 NAS 파일 입력/재시작·키 무출력·원본 파일 보존을 통과했다. Rust 제품 코드는 변경하지 않았다. 실제 key/서비스 HTTP는 사용하지 않았다. 신규 source `28a2a654a23a8efe3e71b47c64fbaeed7a1e9187`의 [Windows/Linux run 44841185626](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/actions/runs/44841185626)은 진행 중이다. 이 코드의 Linux Python 22개·실제 engine NAS file/FD 세션 smoke·원본 보존은 로컬에서 통과했고 새 CI를 성공으로 기록하지 않는다. 이후 상태 문서 커밋과 실행 source를 구분한다. 이전 '평문 파일 입력 없음'은 최신 사용자 명시 요청으로 이 입력 모드에 한해 변경됐다. 입력 파일은 NAS에 남는 평문이며 Git 제외가 NAS 공유 ACL/백업 접근 제어를 대신하지 않는다.

## 최신 — 실제 계정 인증과 SFX 생성 확인

사용자가 NAS 파일에 key/생성 허용·한도를 입력하고 진행을 지시해 [실제 결과](docs/T02_LIVE_RESULTS.md)를 확인했다. SFX v3 500ms POST 1회 succeeded, MP3 8,820 bytes/SHA 13ce7c3f028bd67bc3c517d3ddf1d7bc35c6bacea992a767e84db8f590cf0079, character-cost header 5, 실제 decoded 480ms/stereo/44.1kHz·명시 threshold 초과 sample 0, review pending/현재 일치 true다. ledger는 보수적 예약 200을 유지하며 같은 ID를 재시작 후 다시 제출해도 요청 수 1/기존 결과를 반환했다. 추가 생성/자동 재시도 없음. 실제 청감과 최종 서비스 비용 반영 완료를 주장하지 않는다.

새 기본 state 부모 쓰기 실패 시 기존 ledger가 없는 경우만 private OS temp dk-gamesound-UID로 선택하는 보완과 Linux status transport 표시 수정을 적용했다. 실제 NAS 설정은 수정하지 않았다. 소리/analysis/receipt는 Git 제외 user-data/live-check에 보관한다. key·원격 계정 identity·사용자 설정·음원은 Git/CI에 포함하지 않는다. 게임/사이트 적용은 하지 않았다.

Python 23개/실제 NAS config 세션 smoke·Rust 36개(외부 도구 7개 ignore)·native bins build·Windows x64 타입 검사·fmt/diff를 확인했다. 실제 파일은 별도의 라이브 FFmpeg 분석까지 성공했다. 신규 코드 source `1c21d8e640f97364bc3e1767c0befd08e65aa892`의 [Windows/Linux run 44842948393](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/actions/runs/44842948393)은 진행 중이다. 로컬 라이브 결과/회귀와 새 CI 완료 여부를 구분하며 이후 문서 커밋을 실행 source로 취급하지 않는다. 이전 NAS config source 28a2a654a23a8efe3e71b47c64fbaeed7a1e9187/run 44841185626은 Windows/Linux 전체 성공이다.
