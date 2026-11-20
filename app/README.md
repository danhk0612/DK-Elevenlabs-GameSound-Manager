# DK GameSound — 사운드 관리·후보 생성 v0.3.0

AI가 CLI·stdio MCP로 프로젝트, 필요한 사운드, 파일·버전, 캐릭터·대사를 관리하는 Windows용 로컬 도구다. GUI는 목록과 재생에 필요한 최소 화면을 제공한다. T02 ElevenLabs 계정·한도·TTS/SFX 후보 생성 경로와 Key 없는 mock을 제공한다. **편집·파형/품질 분석·승인/게임 적용은 아직 제공하지 않는다.**

[계정·생성 계약과 CLI 예시](../docs/T02_GENERATION.md)를 먼저 확인한다. 실제 API Key/허용 비용을 사용한 검증은 수행하지 않았다.

[전체 명령·입력·저장·중단 복구 계약](../docs/T01_LOCAL_LIBRARY.md)을 따른다. 다운로드한 개발 ZIP에는 같은 내용을 `LOCAL_LIBRARY.md`로 동봉한다. T02 계약은 `GENERATION.md`로 동봉한다.

## 빠른 시작

`dk-sound-engine.exe`, `dk-sound-cli.exe`, `dk-sound-mcp.exe`, 선택적 `dk-sound-gui.exe`를 같은 폴더에 둔다. Windows 11 x64를 우선 개발 대상으로 한다. CLI/MCP는 WebView2 없이 동작하고 GUI에는 별도 WebView2 Runtime이 필요하다. SQLite는 기본 빌드에 정적으로 포함한다. 로컬 관리/mock에는 FFmpeg/API Key가 필요하지 않다.

```powershell
.\dk-sound-cli.exe schema
.\dk-sound-cli.exe version
.\dk-sound-cli.exe status
'{"protocol_version":1,"op":"project.list"}' | .\dk-sound-cli.exe request
```

실제 파일을 등록하기 전에 기존 로컬 작업공간 폴더를 정하고 project.create로 프로젝트를 만든다. [호출 예시와 필드](../docs/T01_LOCAL_LIBRARY.md)를 사용한다. 데이터는 LOCALAPPDATA 인덱스와 선택한 작업공간의 프로젝트별 DB/원본/후보 폴더에 저장한다. 실행 파일 폴더에 사용자 자료를 두지 않는다.

CLI stdout은 JSON 한 개이며 성공 exit 0/오류 exit 1이다. schema는 엔진 없이 공통 MCP 도구 스키마를 출력한다. --help만 일반 텍스트다. 모든 프로젝트 자료 호출에 project_id를 넣고 안정된 lowercase ID를 사용한다. 파일 import는 읽기·복사이며 외부 원본 이동/수정이나 기존 ready 버전 덮어쓰기를 하지 않는다.

오류/응답 유실 후 새 ID로 무조건 반복하지 않는다. asset.versions/file과 project.recover로 상태를 대조하고 같은 입력/같은 ID로 재개한다. 기존 이전 버전 엔진이 실행 중이면 이전 CLI의 shutdown 또는 5분 idle 종료 후 새 실행 파일 세트를 사용한다. 버전/경로가 다른 바이너리를 한 폴더에 섞지 않는다.

## MCP stdio 설정

해당 로컬 프로그램을 실행할 수 있는 MCP 클라이언트에서 설정한다. 웹 ChatGPT 직접 연결을 보장하지 않는다.

```json
{
  "mcpServers": {
    "dk-gamesound": {
      "command": "C:\\Tools\\DKGameSound\\dk-sound-mcp.exe",
      "args": []
    }
  }
}
```

newline JSON-RPC stdio, protocolVersion 2025-11-25. initialize → notifications/initialized → tools/list/call 순서다. 31개 도구의 inputSchema를 읽고 arguments를 제출한다. 이름은 sound_project_create, sound_asset_import처럼 op의 마침표를 밑줄로 바꾼 형태다. protocol_version/op를 arguments에 넣지 않는다. content text/structuredContent는 같은 공통 결과이며 실패는 isError=true다. shutdown은 CLI만 제공한다. 원격 계정·Voice·모델 조회와 후보 생성 도구를 포함한다. 승인/게임 적용 도구는 없다.

## 엔진·최소 GUI

SID별 Global mutex와 owner-only protected DACL의 local named pipe를 사용한다. remote-client rejection과 first-instance 보호를 설정한다. 같은 사용자의 프로젝트/현재 폴더/Windows 세션이 달라도 한 엔진에 연결한다. 같은 SID의 악성 프로세스/관리자를 차단하는 보안 제품은 아니다.

최초 요청에서 자동 기동하며 child stdio를 분리한다. 엔진 연결은 최대 10초, 응답은 최대 300초 대기하고 전송 후 자동 재제출하지 않는다. 단일 writer 스레드가 로컬 작업을 순차 처리하며 상태에 queued/running을 표시한다. GUI/클라이언트를 닫아도 복사·큐에 등록된 생성은 계속된다. 원격 생성 중 status/job.get/list를 별도 조회할 수 있다. 정상 shutdown은 busy에서 거절하고 force 옵션은 없다. 연결/queued/running이 모두 0인 상태가 5분 지속되면 idle 종료한다. 요청마다 pipe를 연결하므로 창이나 MCP가 열려 있다는 이유만으로 엔진을 상시 유지하지 않는다.

GUI는 프로젝트 목록/명시적 선택, 파일 검색/종류·버전 목록, 해시 대조 후 재생을 제공한다. Key·생성 허용·한도 설정과 연결 해제만 GUI 쓰기로 제공한다. 미리듣기 최대 16 MiB, WebView2 지원 코덱에 한정하며 보관 허용 크기 512 MiB와 구분한다. 파형/주관 청감/품질 적합 판정은 현재 기능이 아니다.

## 빌드

Windows Rust/MSVC build tools, Node 24, GUI용 WebView2를 준비한 뒤 app/에서 실행한다.

```powershell
npm ci
npm run build
cargo build --locked --release --bins
cargo build --locked --release --features gui,custom-protocol --bin dk-sound-gui
```

[빌드 workflow](../.github/workflows/t00-b-build.yml)는 정확한 SHA의 실행 파일·명령 문서·manifest를 개발 artifact로 묶는다. 사용자 지시에 따라 Windows 화면/청감·장시간 실기를 기다리지 않고 구현을 진행한다. artifact는 정식 Release/공개 배포 승인이 아니다. 일반 로컬 회귀는 `cargo test --locked --lib`로 실행할 수 있다.

공식 구현 근거: [rusqlite](https://docs.rs/rusqlite/0.37.0/rusqlite/), [SQLite user_version](https://www.sqlite.org/pragma.html#pragma_user_version), [RustCrypto SHA-2](https://docs.rs/sha2/0.10.9/sha2/), [Tokio named pipe](https://docs.rs/tokio/latest/tokio/net/windows/named_pipe/struct.ServerOptions.html), [Tauri Rust 호출](https://v2.tauri.app/develop/calling-rust/). 문서와 실제 실행 증거를 구분한다.
