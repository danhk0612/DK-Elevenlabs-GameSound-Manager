# T00-A 격리 실험

제품 코드/기본값이 아니다. 실험 전용 폴더만 사용하며 외부 API와 게임 경로를 받지 않는다.
Windows 결과 수집: [절차](../../docs/T00_A_WINDOWS_CHECK.md).

Rust core/engine/CLI/MCP와 Tauri 2 + React/TypeScript GUI를 비교한다.
수동 기동 엔진, loopback TCP + 매 요청 token은 수명/동시성 실험용이다.
제품 IPC 후보인 사용자 ACL named pipe·자동 기동·idle 종료는 이 실험에서 채택하지 않는다.
`T00_SPIKE_ROOT`가 필요하며 임시 경로로 지정한다. 설치 폴더와 별개다.

개발: `npm ci && npm run build`, `cargo build --locked --release`,
Windows GUI: `cargo build --locked --release --features gui --bin spike-gui`.
실행/검사 명령은 scripts/verify.py 참조. stdout에는 CLI JSON/MCP JSON-RPC만 출력한다.
모의 작업은 DB 예약 → 임시 파일 → 최종 파일 → DB 확정이며 비용은 실험용 정수다.
원격 과금/서비스 멱등성/제품 보안이나 정식 MCP 클라이언트 호환성을 증명하지 않는다.
