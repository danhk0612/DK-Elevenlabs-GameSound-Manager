# DK GameSound — T00-B 자동화 실행 기반

Windows 11 x64를 첫 대상으로 개발하는 최소 실행 기반이다. 현재 명령은 version/status/shutdown이며 사운드 등록·생성·편집·게임 적용은 아직 제공하지 않는다. 실제 Key·크레딧·사용자 자료를 사용하지 않는다. T00-A의 실험 실행 파일과 서로 다르다.

## 실행

빌드 산출물의 `dk-sound-engine.exe`, `dk-sound-cli.exe`, `dk-sound-mcp.exe`, 선택적 `dk-sound-gui.exe`를 같은 폴더에 둔다. CLI/MCP에는 WebView2가 필요 없고 GUI에는 별도 WebView2 Runtime이 필요하다. 현 단계에 SQLite DB/FFmpeg 실행은 없다.

```powershell
.\dk-sound-cli.exe version
.\dk-sound-cli.exe status
'{"protocol_version":1,"op":"status"}' | .\dk-sound-cli.exe request
.\dk-sound-cli.exe shutdown
```

stdout은 JSON 한 개, 성공 exit 0/오류 exit 1이다. `--help`만 일반 텍스트다. JSON 요청은 protocol_version=1과 op만 허용하며 추가 필드/알 수 없는 명령을 거절한다. 상태에 engine_pid·버전·연결 수·5분 idle 정책과 `sound_features_available:false`, `jobs.available:false`를 표시한다. 이 상태를 작업 큐 구현으로 해석하지 않는다.

version/status는 엔진이 없으면 같은 폴더의 실행 파일을 자동 기동한다. shutdown은 엔진이 없을 때 새로 띄우지 않고 engine_not_running을 반환한다. 경로는 셸 명령 문자열로 조립하지 않는다. 엔진 자식 stdio는 모두 null로 연결하여 CLI/MCP 종료를 가로막지 않는다. 요청 전 연결은 최대 10초 대기하며 전송 후 응답 유실은 재제출하지 않는다. 오류는 안정된 code만 전달하고 개인 경로/OS 오류 원문을 노출하지 않는다.

## MCP stdio 설정 예시

해당 로컬 실행 파일을 실행할 수 있는 MCP 클라이언트에서 경로를 지정한다. 웹 ChatGPT 직접 연결을 보장하지 않는다.

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

newline JSON-RPC stdio, protocolVersion `2025-11-25`만 제공한다. initialize → notifications/initialized → tools/list, tools/call 순서다. 도구는 `sound_version`, `sound_status`이며 arguments는 `{}` 또는 생략한다. content text와 structuredContent에 같은 공통 응답을 반환하고 실패면 isError=true다. stdout에는 프로토콜만 쓰고 EOF에서 종료한다. 입력/IPC frame은 64KiB까지 허용하며 batch 요청과 원격 HTTP는 제공하지 않는다. CLI만 정상 엔진 shutdown을 제공하며 MCP에는 승인/적용 도구가 없다.

## 엔진 소유와 수명

- 사용자 SID별 `Global\\DKGameSound-{SID}-engine-v1` mutex를 보유한 엔진 하나만 실행한다. 프로젝트·현재 폴더·Windows 세션이 달라도 같은 SID의 `\\.\pipe\DKGameSound-{SID}-engine-v1`을 사용한다. 별도 endpoint/token 파일이 없으며 작업공간별 엔진을 만들지 않는다.
- pipe와 mutex에는 현재 owner SID만 허용하는 보호 DACL을 지정하고, pipe에는 first-instance 보호와 remote-client rejection을 설정한다. 제품 IPC 코드는 T00-A의 TCP/token을 사용하지 않는다. 같은 SID의 악성 프로세스/관리자에 대한 보안 격리 보장은 아니다.
- 엔진은 서비스/로그인 시작이 아니며 최초 요청에서 자동 기동한다. GUI 종료는 엔진 shutdown이 아니다. 각 조회는 요청 동안 pipe에 연결하며 GUI/MCP 프로세스가 열려 있다는 것만으로 상시 연결을 유지하지 않는다. 무요청 상태에는 열린 GUI/MCP가 있어도 5분 idle 종료 후 다음 조회에서 재기동할 수 있다.
- 연결 0·대기/실행 작업 0인 상태가 5분 연속이면 종료한다. 클라이언트가 다시 연결되면 타이머를 리셋한다. 정상 shutdown은 busy에서 거절하며 force 옵션은 없다. 현 단계에 작업 제출 기능이 없으므로 실제 작업 중 수명은 후속 큐 연결 시 확인한다.
- 강제 종료하면 OS가 pipe와 mutex 소유권을 해제한다. 다음 조회가 새 엔진을 시작하며 작업 자동 재실행은 없다. 현재 작업 저장소/복구 기능이 없는 것을 복구 성공으로 표현하지 않는다.

향후 설정은 LOCALAPPDATA 앱 루트, 자료는 사용자 선택 로컬 작업공간에 저장한다. 현재 명령은 설정/DB/오디오 파일을 만들지 않는다.

## 빌드

Windows Rust/MSVC build tools, Node 24, GUI용 WebView2 준비 후 app/에서:

```powershell
npm ci
npm run build
cargo build --locked --release --bins
cargo build --locked --release --features gui,custom-protocol --bin dk-sound-gui
```

[T00-B workflow](../.github/workflows/t00-b-build.yml)는 정확한 SHA의 컴파일/개발용 artifact만 수행한다. 사용자 지시에 따라 Windows 화면/청감·장시간 실기를 개발 착수 게이트로 기다리지 않는다. 빌드 artifact는 정식 Release/배포 승인이 아니다.

공식 구현 근거: [Tokio pipe server/security attributes](https://docs.rs/tokio/latest/tokio/net/windows/named_pipe/struct.ServerOptions.html), [pipe client](https://docs.rs/tokio/latest/tokio/net/windows/named_pipe/struct.ClientOptions.html), [Windows named pipe security](https://learn.microsoft.com/en-us/windows/win32/ipc/named-pipe-security-and-access-rights), [Tauri Rust 호출](https://v2.tauri.app/develop/calling-rust/). 문서와 제품 실측을 구분한다.
