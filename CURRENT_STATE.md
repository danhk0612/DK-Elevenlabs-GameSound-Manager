# 현재 상태

기준일: 2026-11-20. **v0.3.0 T02 계정·한도·후보 생성 경로 구현. AI 자동화 우선·최소 UI, 실기/점검 대기 해제 유지.**

## 지시와 저장소

사용자 “실기, 점검 무시하고 진행해”에 따라 Windows 화면/청감 결과를 요청하며 개발을 멈추지 않는다. 미확인을 통과로 바꾸지는 않는다. D01~D06 위임 채택을 유지한다.

main `4c3537c55084b71feb5c42eb2340005bf922f197`, T01 기반 `fa8c5a9881a06de153e6b53124947ba4ce0591a4`에서 기존 `task/t00-a-technical-spike`와 [draft PR #1](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/pull/1)을 재사용한다. 이 변경의 정확한 구현 SHA는 PR head/Git 기록과 artifact source-sha.txt를 따른다.

## 현재 구현

[app 사용법](app/README.md), [T01 로컬 계약](docs/T01_LOCAL_LIBRARY.md), [T02 연결·생성 계약](docs/T02_GENERATION.md).

- SID별 단일 Windows 엔진·owner-only/local-only named pipe·자동 기동·5분 idle·busy 종료 거절을 유지한다.
- 공통 명령 32개/CLI JSON/MCP 31개 도구. 모든 프로젝트 자료는 명시적 project_id를 요구한다. schema 조회에는 엔진이 필요 없다.
- catalog schema 1→2 transaction migration, 프로젝트 DB schema 1 유지. 기존 원본/후보의 SHA·출처·부모·no-replace 게시·copying journal/복구를 유지한다. 최초 생성도 candidate로만 등록한다.
- Credential Manager 현재 사용자 Key·유료 생성 허용·한도 정책. GUI password 입력/삭제 이외의 Key 조회/CLI/MCP 입력·DB/로그 저장·fallback은 없다. GUI는 기존 목록/해시 재생 + 필수 연결 설정만 제공한다.
- WinHTTP 고정 HTTPS host·redirect 금지·timeout·32 MiB 응답 제한. 계정/사용량·Voice/설정·모델 조회와 TTS/SFX POST adapter, Key 없는 합성 mock을 구현했다. 실제 Key/서비스 호출은 하지 않았다.
- workspace identity를 통한 여러 프로젝트/계정 별칭 공유 예산, 미설정 차단, 요청·크레딧 예약 transaction. D09 기본 숫자는 확정하지 않았다. 실제 정책은 GUI 명시 설정·기존보다 작은 값만 반영·누적 수명 전체/자동 초기화 없음이다. **실제 청구액 상한은 보장하지 않는다.**
- persistent 비동기 생성 큐, 안정된 job ID/snapshot·대상 버전 예약, 단일 POST, 취소/재시작 불확실 상태·수신 파일 등록 복구. 중단 요청은 자동 재전송하지 않는다. status/job.get/list는 원격 요청 중 별도 조회하며 다른 로컬 명령/취소는 네트워크 작업 뒤에서 처리될 수 있다.

## 확인한 결과

- T01 `fa8c5a9881a06de153e6b53124947ba4ce0591a4`의 [Windows run 44736369277](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/actions/runs/44736369277)은 engine/CLI/MCP/GUI 빌드·패키징 성공이다. 이는 새 T02 실행 증거가 아니다.
- T02 Linux musl Rust 1.100.0·Zig 0.17.1 CC/AR + 기본 bundled SQLite로 `cargo test --locked --manifest-path app/Cargo.toml --lib` **19개 통과**, `cargo build --locked --manifest-path app/Cargo.toml --bins` 통과.
- mock TTS/SFX 후보 생성·기존 원본 보존·정책 미설정·workspace별 프로젝트/별칭 합산·ID 재사용/충돌·취소·재시작 interrupted/unknown·네트워크 실패/429 단일 POST·비용 초과 후속 차단·수신 파일 재등록·대사 snapshot/secret/입력 거절을 확인했다. T01의 경로/schema/참조/파일 중단 회귀도 유지했다.
- native CLI schema와 실제 MCP stdio initialize/list/call/EOF에서 **31개 도구**, secret 추가 필드 거절, 원격 도구 annotation을 확인했다. Linux 제품 지원을 추가한 것은 아니다.
- Windows x64 engine/CLI/MCP `cargo check --locked --manifest-path app/Cargo.toml --bins --target x86_64-pc-windows-msvc --no-default-features` 통과. 외부 SQLite link 선언을 사용한 타입 검사이며 정식 bundled Windows 링크/실행과 구별한다. Credential Manager/WinHTTP 선언도 이 검사에 포함됐다.
- Node 24.18.1 TypeScript/Vite 최소 UI build 통과. Rust fmt·JSON/문서 링크·git diff whitespace를 확인했다. Windows GUI native 빌드는 해당 head CI로 별도 확인한다. 실제 Key·요금제 권한·비용·청감·GUI 동작은 미확인이다.

로컬 도구: /tmp/dk-cargo, /tmp/dk-rustup, /tmp/dk-rust-tools의 rust-lld wrapper·Zig CC/AR. CFLAGS=-fno-sanitize=all로 bundled SQLite를 빌드했다. 과거 시스템 동적 SQLite/static musl·동적 CRT 임시 시도는 SIGSEGV, Zig 기본 sanitizer object는 link 실패였다. 최종 테스트는 시스템 SQLite 대체 없이 기본 bundled 설정을 사용했다. 임시 도구·합성 fixture·음원은 Git에 포함하지 않는다.

Windows workflow는 이제 기본 bundled 회귀 테스트와 engine/CLI/MCP/GUI 빌드를 수행하고 README·LOCAL_LIBRARY.md·GENERATION.md·source SHA·manifest를 개발 artifact로 묶는다. 새 head CI 결과는 별도로 기록하며 사용자 실기를 기다리지 않는다.

## 남은 범위와 다음 작업

T02 전체를 완료로 닫지 않는다. 실제 계정/Key 수명/무료 권한·과금 검증은 없으며 Voice preview는 URL 조회만 제공한다. 한도 증액/초기화·다른 PC 공유·불확실 비용 정산 UI도 없다. 이 미확인을 개발 대기 조건으로 사용하지 않는다. 다음은 남은 자동화 보강 또는 T03 기본 편집/파형·기술 검사다. D08 출력/검사 임계값의 미결정과 T00 FFmpeg 근거를 먼저 확인한다. T05 승인/게임 적용과 T06 배포는 미구현이다.

T00-A source 828e745의 [과거 33개 Windows 자동 검사/별도 pipe ACL/ZIP 증거](docs/T00_A_RESULTS.md)와 미수령 사용자 실기를 보존한다. [증거 검사 CLI](spikes/t00-a/tools/README.md)는 현재 작업 승인 상태 판정기가 아니다.

게임 개발은 일시 정지다. 게임 수정 요구는 사이트 games/02-deck-expedition/POST_AUDIO_BACKLOG.md에 보관했다. 직접 병합·Release/tag·공개 배포·실제 Key/크레딧·사용자 음원/게임 소스/DB/사이트 빌드·배포 변경은 수행하지 않았다.
