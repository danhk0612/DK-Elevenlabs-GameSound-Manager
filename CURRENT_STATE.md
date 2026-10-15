# 현재 상태

기준일: 2026-10-15. 단계: **T00-A 진행 중, Windows 실기/설계 게이트 미완료. 제품 구현 전.**

- 시작 main: `4c3537c55084b71feb5c42eb2340005bf922f197`. 작업 시작 시 기존 구현·열린 PR 없음.
- 브랜치: `task/t00-a-technical-spike`, [검토/결과 수집 PR #1](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/pull/1).
- `spikes/t00-a/`에만 Rust 공통 요청·단일 엔진·독립 CLI·stdio MCP·Tauri 2/React/TS 최소 GUI, 임시 SQLite/파일·합성 음원 검사, dummy 자격증명 비교 스크립트와 Windows CI를 격리했다. 제품 스택·경로·수명 기본값으로 채택하지 않았다.
- 실제 사운드 관리·ElevenLabs 연결·사람 승인·게임 적용 제품 기능은 여전히 없다.

## 검증

[정확한 SHA·명령·환경·결과](docs/T00_A_RESULTS.md), [D01~D06 근거와 조건부 T00-B 계약](docs/T00_A_DECISIONS.md), [Windows 실기 결과 수집](docs/T00_A_WINDOWS_CHECK.md).

Linux에서 Rust 빌드, GUI UI 타입 검사/빌드, 실제 엔진/CLI/MCP 프로세스, 중복 기동·동시 ID/모의 예산·클라이언트 종료 후 작업·재연결, kill/정상 종료 후 중단 상태·예약 보존, SQLite 무결성, 한글/공백 경로 FFmpeg/ffprobe·합성 원본 hash 보존이 통과했다.

파일 확정과 DB 기록 사이 종료에서 잔여 파일+interrupted 상태를 재현했다. **자동 파일/DB 대조 복구는 구현하지 않았다.** 직접 라이브러리 접근 대안은 SQLite writer 잠금 fixture 비교에 한정된다.

Windows 코드 SHA `80993133331c134e8a6bea12c7a9c8e0f7493a4e`의 [CI 38693413115](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/actions/runs/38693413115)가 통과했다. GUI/CLI/엔진/MCP 빌드·16개 자동 검사 및 [artifact](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/actions/runs/38693413115/artifacts/11953730364)의 SHA·파일 해시·PE x64를 확인했다. Credential Manager dummy 쓰기/읽기/삭제와 DPAPI CurrentUser 왕복도 통과했다. GUI 화면·GUI 공통 명령 실제 실행·창 종료/재연결·WebView2 청감은 사용자 결과 미수령. CI 성공이나 GUI 프로세스 생존과 구분한다. Credential Manager/DPAPI는 dummy 검사만 허용하며 실제 Key는 사용하지 않았다.

## 차단과 다음 작업

T00-A를 완료로 닫지 않는다. 남은 Windows 실기, 사용자 ACL IPC·사용자 전체 소유권·클라이언트 자동 기동/idle 수명 검증을 확보한다. 현재 loopback TCP/token·작업공간별 lock은 제품 계약이 아니다.

D01~D06 사용자 결정 없음. 근거 확인·실기 후 고성능 Chat/Work에서 결정하고, T00-A 완료와 문서 반영 뒤 T00-B는 저성능 Work에 인계한다. 조건부 계약 작성만 했으며 T00-B에 착수하지 않았다.

API Key·크레딧·원격 생성, 실제 게임·음원 수정, 제품 라이선스·정식 배포 방식 확정, 직접 병합·Release/tag·공개 제품 배포는 수행하지 않았다.
