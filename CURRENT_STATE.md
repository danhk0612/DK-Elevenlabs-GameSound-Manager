# 현재 상태

기준일: 2026-11-20 KST. **T01 로컬 라이브러리 v0.2.0 구현. AI 자동화 우선·최소 UI, 실기/점검 대기 해제 유지.**

## 지시와 저장소

사용자 “실기, 점검 무시하고 진행해”에 따라 Windows 화면/청감 결과를 요청하며 개발을 멈추지 않는다. 미확인을 통과로 바꾸지는 않는다. D01~D06 위임 채택을 유지한다.

main `4c3537c55084b71feb5c42eb2340005bf922f197`, 기존 `task/t00-a-technical-spike`와 [draft PR #1](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/pull/1)을 재사용한다. 현재 정확한 구현 SHA는 PR head/Git 기록을 따른다.

## 현재 구현

[app 사용법](app/README.md), [T01 명령/저장/복구 계약](docs/T01_LOCAL_LIBRARY.md).

- T00-B SID별 단일 Windows 엔진·owner-only/local-only named pipe·자동 기동·5분 idle·busy 종료 거절 기반을 유지한다.
- 공통 명령 18개(기본 3 + 프로젝트/자료 15), CLI JSON request와 엔진 없이 schema 조회, stdio MCP 17개 도구. 모든 프로젝트 자료 요청에 project_id를 요구하고 공통 카탈로그에서 필드/형식/ID/페이지를 검증한다. 전역 current project는 없다.
- LOCALAPPDATA catalog DB와 사용자 선택 로컬 작업공간의 프로젝트별 schema 1 SQLite, 원본/후보/부모 버전·SHA256·출처·복사 journal·이벤트. 불명 DB/미래 schema·network workspace·관리 symlink/reparse point·대소문자 ID 충돌·장치 예약명을 거절한다.
- 원본 읽기/복사, 최대 512 MiB, 별도 .part, 원본과 후보를 분리하고 no-replace hard link로 게시한다. 기존 ready 버전 hash 불일치/ID 충돌을 자동 덮어쓰지 않는다. copying 중단 대조/재개를 제공하며 완전한 파일이면 외부 원본 없이 ready로 복구한다. 전원 장애 전체/임의 외부 파일 복구는 보장하지 않는다.
- 단일 writer 스레드가 로컬 자료 명령을 순차 처리한다. queued/running을 상태에 표시하고 정상 busy 종료를 거절한다. 클라이언트 종료 후에도 복사는 계속된다. 이는 원격 생성 작업/계정 예산 큐가 아니다. 응답 최대 300초 후 유실돼도 자동 재제출하지 않는다.
- 필요한 사운드 작성/검색/파일 버전 연결, 캐릭터/외부 Voice ID 참조, 대사/언어/음성 requirement 연결. 페이지는 최대 10개이며 escape 후 전송 크기를 고려해 더 줄일 수 있다. put 선택 필드 생략은 빈 값/연결 없음이다.
- 최소 GUI는 상태, 프로젝트/파일/버전 목록·검색/선택, 해시를 대조한 16 MiB 미리듣기만 제공한다. 등록/수정 폼을 늘리지 않았다. 코덱 재생 지원·청감/음질 검사는 별도다.

## 확인한 결과

- 이전 T00-B source `693c42a461bb67fecc369f5edd715415e6fe4da2`의 [Windows build run 44720496158](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/actions/runs/44720496158)에서 engine/CLI/MCP/최소 GUI 링크·패키징이 성공했다. 이는 새 T01 코드의 Windows 실행 증거가 아니다.
- T01 Linux x86_64 musl의 격리 Rust 1.100.0 + Zig 0.17.1 CC/AR, bundled SQLite 기본 feature로 `cargo test --lib` **12개 통과**, 독립 진입점 cargo build 통과. 프로젝트/같은 이름·ID 격리, 원본/후보 보존·중복 ID/입력 충돌, publish/DB 사이 중단과 원본 없는 복구, 부분 복사 재개, ready 변조 보존, 참조/kind/입력/미래 schema/symlink 거절, escape 후 페이지 byte 한도를 확인했다.
- 실제 native CLI schema와 MCP stdin/stdout/EOF/공통 오류 smoke에서 17개 도구와 잘못된 project_id 거절을 확인했다. Linux IPC/프로그램 지원을 추가한 것은 아니다.
- Windows x64 대상 engine/CLI/MCP cargo check는 로컬 C cross-linker 부재로 `--no-default-features`와 외부 SQLite link 선언을 사용해 타입 검사했다. Windows 정식 기본 bundled 빌드/실행의 증거로 확대하지 않는다.
- Node 24.18.1 TypeScript/Vite 최소 UI build 통과. fmt, JSON/문서 링크, whitespace도 확인한다. 사용자 화면·실제 청감과 새 GUI의 Windows 네이티브 실행은 미확인이다. 이를 개발 선행 차단으로 사용하지 않는다.

로컬 도구는 /tmp/dk-cargo, /tmp/dk-rustup, /tmp/dk-rust-tools의 rust-lld wrapper·Zig CC/AR를 사용했다. CC 환경은 ZIG cache를 /tmp로 분리하고 CFLAGS=-fno-sanitize=all로 별도 sanitizer 런타임 의존을 제거했다. 최초 시스템 동적 SQLite와 static-musl 조합은 SIGSEGV, 동적 CRT 임시 시도는 host build script SIGSEGV, Zig 기본 sanitizer object는 link 실패였다. 제품 의존성을 시스템 SQLite로 바꾸지 않고 기본 bundled SQLite 빌드로 해결해 최종 12개를 통과했다. 임시 도구/fixture는 Git에 포함하지 않는다.

새 T01 head의 Windows workflow는 bundled SQLite 포함 기본 설정으로 빌드하고 README·LOCAL_LIBRARY.md·source SHA·manifest를 개발 artifact에 묶는다. 해당 run의 실제 결과를 별도로 확인하며 사용자 실기를 기다리지 않는다.

## 미구현/다음

T02 실제 Credential Manager/API 연결·Voice 조회/TTS/SFX·계정 한도, T03 편집/파형/기술 검사, T05 사람 승인/게임 적용/복구는 미구현이다. API Key와 허용 비용이 없으면 T02 모의 응답·오프라인 경로부터 개발한다. 실제 원격 생성은 하지 않는다. D09 수치는 미결정이며 미설정 한도를 무제한으로 해석하지 않는다.

T00-A source 828e745의 [과거 33개 Windows 자동 검사/별도 pipe ACL/ZIP 증거](docs/T00_A_RESULTS.md)와 미수령 사용자 실기는 그대로 보존한다. [증거 검사 CLI](spikes/t00-a/tools/README.md)는 현재 작업 승인 상태 판정기가 아니다.

실제 게임 개발은 일시 정지다. 사용자의 최신 게임 수정사항은 사이트 저장소 games/02-deck-expedition/POST_AUDIO_BACKLOG.md에 후속 요구로만 보관했다. 직접 병합·Release/tag·공개 배포·실제 Key/크레딧·실제 사용자 음원/게임 소스/DB/사이트 빌드·배포 변경은 수행하지 않았다.
