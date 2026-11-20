# 현재 상태

기준일: 2026-11-20 UTC(한국 11-21). **v0.10.0 Linux 인증 세션·ElevenLabs HTTPS 구현. AI 자동화 우선·최소 UI·실기 대기 해제 유지. 실제 서비스 생성/과금은 미검증이다.**

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

Rust 환경은 /tmp/dk-cargo·/tmp/dk-rustup·Zig CC/AR/rust-lld wrapper(CFLAGS=-fno-sanitize=all), Python /tmp/msv-groups-tools/python3, FFmpeg /tmp/dk-audio-tools다. 도구/합성 인증서·음원·key는 Git에 넣지 않는다. CI Linux job에 실제 credential-session smoke와 로컬 TLS 회귀를 연결했고 Windows job은 기존 native/GUI/CLI·MCP·배치/패키징을 유지한다. 새 v0.10 코드 SHA/CI 결과는 push 후 기록하며 아직 성공으로 표시하지 않는다.

## 남은 범위

Linux 실제 API 키·서비스 권한/quota·허용 생성 범위/비용이 안전한 인증 세션으로 준비돼야 실제 GET/생성·결과 파일·과금/청감을 검증할 수 있다. 현재는 구현 및 로컬 합성/모의 검증 완료이며 실제 사운드 제작 성공이 아니다. 키를 채팅/인자/env/평문 파일로 받지 않고 실제 Key 없이 라이브 요청하지 않는다.

T04 전체·제품 완성/T05 착수는 선언하지 않는다. 지원 AI 제품 MCP 등록, 원격 preview 재생·한도 증액/초기화·다른 PC 공유/불확실 비용 정산, Windows 실제 FFmpeg/취소·사용자 Windows 11 화면/청감은 별도다. D08 loop tag/LUFS/true peak·D07 승인/게임 적용은 후속 범위다.

사이트/게임 개발은 계속 일시 정지한다. 병합·Release/tag·공개 배포·실제 Key/크레딧 사용·사용자 음원/게임/사이트/운영 DB·사이트 빌드/배포는 수행하지 않았다. 게임 수정 목록은 사이트 games/02-deck-expedition/POST_AUDIO_BACKLOG.md를 유지한다.
