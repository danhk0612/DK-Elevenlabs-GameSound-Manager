# 현재 상태

기준일: 2026-11-20. **v0.4.0 T03 로컬 편집·변환·분석 자동화 구현. AI 자동화 우선·최소 UI, 실기/점검 대기 해제 유지.**

## 지시와 저장소

사용자 “실기, 점검 무시하고 진행해”에 따라 Windows 화면/청감 결과를 요청하며 개발을 멈추지 않는다. 미확인을 통과로 바꾸지 않는다. D01~D06 위임 채택 유지, D08 제품 기본값·D09 기본 한도 수치는 미결정이다.

main `4c3537c55084b71feb5c42eb2340005bf922f197`, T02 기반 `4119217e4cc0b0cf162998faf810563a2418620c`에서 기존 `task/t00-a-technical-spike`와 [draft PR #1](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/pull/1)을 재사용한다. 새 구현의 정확한 SHA는 PR head/Git 기록과 artifact source-sha.txt를 따른다.

## 현재 구현

[app 사용법](app/README.md), [T01 로컬 계약](docs/T01_LOCAL_LIBRARY.md), [T02 연결·생성 계약](docs/T02_GENERATION.md), [T03 편집·분석 계약](docs/T03_AUDIO.md).

- 공통 명령 **39개 / CLI JSON / MCP 38개 도구**. 명시적 project_id와 공통 입력 검증을 사용한다. SID별 단일 엔진·owner-only local pipe·자동 기동·busy 종료 거절·5분 idle은 유지한다.
- 로컬 프로젝트/자료·캐릭터/대사·원본/후보 SHA·출처·부모·copying journal과 no-replace 게시/복구를 유지한다. 생성·편집은 candidate만 만들고 게임 파일을 적용/승인하지 않는다.
- T02 Credential Manager/고정 HTTPS WinHTTP adapter, 계정·Voice/모델·사용량 조회, workspace 공유 예산·미설정 차단·단일 POST 큐/취소/불확실 상태·수신 파일 등록 복구와 합성 mock을 유지한다. 실제 청구액 상한 보장은 하지 않는다.
- T03 명시적 신뢰 도구 경로·hash/version 설정, trim/gain/선형 fade·rate/channel·WAV PCM16/OGG Vorbis q4/MP3 128k 변환. 작업마다 출력 조건을 요구하며 D08 기본값을 임의 확정하지 않는다.
- ffprobe 정보 + 첫 오디오 스트림의 전체 채널 PCM에서 길이·sample peak/RMS·threshold 샘플 수·quiet/무음 구간·min/max 파형·선택 loop seam 수치를 측정한다. 검사 조건·입력 hash·도구 버전/hash를 저장하고 주관 적합/승인을 분리한다. LUFS/true peak는 측정하지 않는다.
- catalog schema 2→3 transaction migration으로 audio_tools/audio_operations를 추가했다. 프로젝트 DB schema 1은 유지한다. local audio 명령은 writer의 동기 실행이며 status는 별도 응답한다. 원본 전후 hash·대상 예약·operation snapshot, 새 결과 파일·output_ready → 후보 등록, 중단 상태/명시적 등록 복구를 제공한다.
- GUI는 기존 중립색 목록/설정/재생 화면을 유지하며 선택 버전의 저장된 분석 ID 조회·파형/수치, 별도 비교 재생기만 추가했다. 생성/편집/분석 실행은 CLI/MCP를 우선한다.

## 확인한 결과

- T02 `4119217e4cc0b0cf162998faf810563a2418620c`의 [Windows run 44739283875](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/actions/runs/44739283875)은 기본 bundled 테스트·engine/CLI/MCP/GUI 빌드·패키징 성공이다. 새 T03의 Windows 실행 증거는 아니다.
- T03 Linux musl Rust 1.100.0·Zig CC/AR, 기본 bundled SQLite로 **26개 통과**: `DK_TEST_FFMPEG=/tmp/dk-audio-tools/ffmpeg DK_TEST_FFPROBE=/tmp/dk-audio-tools/ffprobe cargo test --locked --manifest-path app/Cargo.toml --lib -- --include-ignored`. 실제 FFmpeg/ffprobe 7.0.2 static, 합성 신호/한글·공백 경로만 사용했다. 바이너리 출처/hash는 T03 계약에 기록했다.
- 알려진 PCM peak/RMS/길이·무음/파형·stereo/loop seam·NaN/부분 PCM 거절, 실제 trim/gain/fade/resample/채널 변환·WAV/OGG/MP3, 원본 보존·후보·중복/프로젝트 경계·잘못된 입력/출력 충돌·재시작/등록 복구·변조/도구 변경과 프로세스 timeout/출력 제한을 확인했다. T01/T02 회귀도 유지했다.
- 외부 도구 없는 기본 `cargo test --locked --lib`는 Linux **22개 통과 / 실제 FFmpeg 통합 4개 ignore**다. Windows CI 기본 테스트에서 이 4개까지 실행했다고 취급하지 않는다.
- Windows x64 engine/CLI/MCP cargo check는 외부 SQLite link 선언 + `--no-default-features`로 통과했다. 기본 bundled Windows 링크/실제 실행과 구분한다. Node 24.18.1 TypeScript/Vite GUI build 통과.
- native 진입점 build·CLI schema와 MCP handshake/list/call/EOF의 38개 도구, typed audio 입력·추가 필드 거절·출력 schema를 확인했다. fmt·JSON/문서 링크·whitespace 확인도 통과했다. Windows GUI 실제 동작/비교·외부 FFmpeg 재생·사용자 청감과 실제 유료 API는 미확인이다.

로컬 Rust 도구는 /tmp/dk-cargo, /tmp/dk-rustup, /tmp/dk-rust-tools를 사용한다. CFLAGS=-fno-sanitize=all과 Zig CC/AR·rust-lld wrapper로 bundled SQLite를 빌드했다. 과거 시스템 SQLite/static musl·임시 동적 CRT SIGSEGV와 sanitizer link 실패는 기본 bundled 빌드로 해결했다. 도구/fixture/음원은 Git에 넣지 않는다.

새 Windows CI는 기본 회귀와 engine/CLI/MCP/GUI 빌드, README·LOCAL_LIBRARY.md·GENERATION.md·AUDIO.md·source SHA·manifest 패키징 대상이다. 해당 head 결과를 별도로 확인하며 수동 실기를 기다리지 않는다.

## 남은 범위와 다음 작업

T03 기본 자동화 경로를 구현했으며 전체 수용 완료로 닫지 않는다. 긴 로컬 작업의 async 큐/취소·검토 제출/이력 조회 개선 등 T04 자동화 보강이 다음이다. D08 게임별 기본 프로필, loop tag·LUFS/true peak·전문 타임라인은 없다. 실제 AI 클라이언트 호환은 stdio smoke와 구별해 기록한다. T05 승인/게임 적용(D07)은 별도 설계 범위이며 아직 착수하지 않는다.

T02 실제 Key 수명/요금제/무료 권한·과금·청감은 미확인, Voice preview는 URL 반환까지만 제공한다. 한도 증액/초기화·다른 PC 공유·불확실 비용 정산도 후속이다. 이 미확인을 개발 대기 조건으로 사용하지 않는다.

T00-A source 828e745의 [과거 Windows 33개/별도 ACL·ZIP 증거](docs/T00_A_RESULTS.md)와 미수령 사용자 실기를 보존한다. 과거 [증거 검사 CLI](spikes/t00-a/tools/README.md)는 현재 개발 승인 상태 판정기가 아니다.

게임 개발은 일시 정지다. 게임 요구는 사이트 games/02-deck-expedition/POST_AUDIO_BACKLOG.md에 보관했다. 병합·Release/tag·공개 배포·실제 Key/크레딧·사용자 음원/게임 소스/DB/사이트 빌드/배포 변경은 수행하지 않았다.
