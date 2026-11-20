# 현재 상태

기준일: 2026-11-20 KST. **T00-B 최소 실행 기반 구현. 사용자 지시로 실기·점검 대기 해제.**

## 현재 유효한 지시

사용자 “ai가 자동화 가능한 작업을 위한 개발에 집중해. 실제 화면(UI)는 최소한도로.”에 이어 **“실기, 점검 무시하고 진행해”**로 이전 T00-A 실기 대기와 T00-B 착수 금지를 해제했다. 실기 미확인을 성공으로 기록하지 않고 개발을 진행한다. D01~D06 위임 채택은 유지하며, 미수령 결과를 다시 요구하며 대기하지 않는다.

main은 `4c3537c55084b71feb5c42eb2340005bf922f197`이고 기존 `task/t00-a-technical-spike` 및 [draft PR #1](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/pull/1)을 재사용한다. 기존 브랜치 이름은 보존하지만 현재 작업 범위는 T00-B까지 확장됐다. 정확한 최신 구현 SHA는 PR head와 Git 기록으로 확인한다.

## 구현

[app/](app/README.md)에 Rust 공통 명령/응답, Windows 엔진·독립 CLI·stdio MCP·최소 Tauri/React/TS GUI를 추가했다.

- SID별 Global mutex + owner SID protected DACL의 로컬 named pipe, 원격 클라이언트 거절 옵션, first-instance 보호. 작업공간·현재 폴더와 별개로 같은 SID 엔진을 사용한다. TCP/token fixture를 제품에 복사하지 않았다.
- version/status에서 최초 자동 기동, child stdio 분리, 연결/응답 timeout, 전송 이후 자동 재제출 없음. shutdown은 미실행 엔진을 시작하지 않는다.
- 연결/queued/running 모두 0인 5분 연속 idle 종료, 재연결 시 타이머 초기화, busy 정상 종료 거절. 현재 작업 제출/DB/큐 기능은 없고 응답에 jobs.available=false로 표시한다. 실제 작업 중 수명/복구는 후속 작업 큐 연결 시 확인한다.
- CLI JSON request/response와 종료 코드, MCP 2025-11-25 initialize/notification/tools/list/call 및 구조화 응답. 도구는 sound_version/sound_status만 제공한다. 추가 인자/알 수 없는 명령/강제 종료/승인/적용을 제공하지 않는다.
- GUI는 상태 조회/재연결 버튼과 결과/오류만 표시한다. 조회마다 pipe에 연결하고 창 자체를 상시 연결로 간주하지 않는다. GUI/MCP가 열려 있어도 요청이 없으면 idle 종료, 다음 요청에서 자동 재기동한다.
- 별도 lockfile과 [Windows 빌드 workflow](.github/workflows/t00-b-build.yml)를 추가했다. 이전 T00-A CI는 spike 경로 변경 때만 자동 실행해 제품 개발에서 33개 과거 점검을 반복하지 않는다.

## 이번 확인과 미확인

Linux x86_64 musl의 /tmp 격리 Rust 1.100.0·Node 24.18.1 환경에서 다음을 확인했다. 저장소 밖 도구 설치만 사용했고 사이트/운영 환경은 변경하지 않았다.

- cargo check --locked --manifest-path app/Cargo.toml --bins 통과.
- cargo check --locked --manifest-path app/Cargo.toml --bins --target x86_64-pc-windows-msvc 통과. Windows 엔진/CLI/MCP 타입 검사이며 Windows 링크/실행/GUI 빌드 성공으로 확대하지 않는다.
- cargo test --locked --manifest-path app/Cargo.toml --lib: 요청 검증, busy/idle, shutdown, MCP 협상/인자 검사 4개 통과.
- cargo build --locked --manifest-path app/Cargo.toml --bins 통과. Linux 실행 파일의 unsupported_platform 거절 및 실제 MCP stdin/stdout/EOF smoke 통과. Linux용 엔진을 지원한다는 뜻은 아니다.
- npm ci 및 app/의 npm run build(TypeScript/Vite) 통과. Rust fmt/문서 링크/whitespace도 확인한다.

로컬 Rust linker가 없어 /tmp/dk-rust-tools/rustc-wrapper가 rust-lld와 /tmp/dk-link-libs의 musl libc/libgcc 링크를 지정했다. 재현 가능한 일반 개발 명령은 app/README.md에 있다. 이 로컬 컴파일의 Windows 직접 실행·GUI 네이티브 링크/실행·다른 SID/원격 접속·실제 5분 수명은 미확인이다. 새 workflow의 실제 결과와 artifact는 해당 head의 run으로 별도 확인한다. 사용자의 대기 해제 지시에 따라 이를 개발 착수 차단으로 사용하지 않는다.

## 보존한 T00-A 증거

실행 source `828e74540ae0aa0013d6ec3425389cb0631b0b1a`, [run 44713166683](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/actions/runs/44713166683), [artifact 14618906066](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/actions/runs/44713166683/artifacts/14618906066)의 Windows CI 33개 및 별도 native pipe ACL 성공은 [원시 증거](docs/t00-a-evidence/refresh-20261120/artifact-inspection.json)·[결과](docs/T00_A_RESULTS.md)에 그대로 보존한다. 사용자 Windows 화면/실제 청감은 미수령이며 새 app/ 구현의 Windows 실행 증거가 아니다. 파일/DB 자동 대조 복구와 계정 예산은 과거 spike에서도 미구현이다. [T00-A 증거 검사 CLI](spikes/t00-a/tools/README.md)는 정적 증거 대조 도구이며 현재 작업 승인 상태의 판정기가 아니다.

## 다음 구현

T01 프로젝트·로컬 사운드 관리: 원본 복사/보존·프로젝트 격리·자산/후보 메타데이터·필요 사운드 목록·캐릭터/대사의 공통 명령·CLI/MCP부터 만든다. UI 확장은 최소화한다. T02 원격 생성·실제 Key/비용, T03 편집, T05 승인/게임 적용은 각각 후속 범위다.

직접 병합·Release/tag·공개 배포·실제 Key/크레딧·실제 게임/음원 변경은 수행하지 않았다. 사이트 게임 개발은 계속 일시 정지다.
