# T00-A 격리 실험

제품 코드/기본값이 아니다. 실험 전용 폴더만 사용하며 외부 API와 게임 경로를 받지 않는다.
Windows 결과 수집: [절차](../../docs/T00_A_WINDOWS_CHECK.md).

Rust core/engine/CLI/MCP와 Tauri 2 + React/TypeScript GUI를 비교한다.
수동 기동 엔진, loopback TCP + 매 요청 token은 수명/동시성 실험용이다.
제품 IPC 후보인 사용자 ACL named pipe·자동 기동·idle 종료는 이 실험에서 채택하지 않는다.
`T00_SPIKE_ROOT`가 필요하며 임시 경로로 지정한다. 설치 폴더와 별개다.

개발: `npm ci && npm run build`, `cargo build --locked --release`,
Windows GUI: `cargo build --locked --release --features gui,custom-protocol --bin spike-gui`.
실행/검사 명령은 scripts/verify.py 참조. stdout에는 CLI JSON/MCP JSON-RPC만 출력한다.
모의 작업은 DB 예약 → 임시 파일 → 최종 파일 → DB 확정이며 비용은 실험용 정수다.
원격 과금/서비스 멱등성/제품 보안이나 정식 MCP 클라이언트 호환성을 증명하지 않는다.

추가 수명 후보(실험에 명시적으로 설정할 때만): `T00_SPIKE_AUTOSTART=1`,
`T00_SPIKE_IDLE_MS=500..60000`. 미설정 시 기존 수동 기동·idle 무기한 유지.
`shutdown_if_idle`은 작업 중 거절, `shutdown`은 중단 상태를 남기고 종료한다.
수명 검사는 scripts/lifecycle.py, 실제 WebView2 명령·닫기·재연결·재생 이벤트는
scripts/gui-webdriver.py와 Microsoft Edge WebDriver로 검사한다. GUI 테스트는
실제 앱 바이너리를 사용하며 invoke mocking과 autoplay 우회 옵션을 쓰지 않는다.
스크린샷과 ended 이벤트는 사람의 청감 확인을 대체하지 않는다.

scripts/pipe-acl.ps1은 현재 사용자 SID DACL의 native named pipe를 별도로 생성하고,
다른 실제 로컬 계정 토큰의 접속을 permissive control과 비교한다.
CI는 `-UseEphemeralCiUser`로 일회용 GitHub-hosted Windows runner에서만
임시 계정을 생성하고 finally 삭제까지 검사한다. 사용자 PC에서는 이 모드를 거절한다.
기본 LocalService 후보는 토큰 발급 권한에 따라 실패할 수 있으며 성공을 보장하지 않는다. 엔진은 계속 TCP/token
스파이크이며 named pipe로 전환하거나 사용자 전체 singleton을 채택하지 않았다.
