# 현재 상태

기준일: 2026-11-20 KST. **T00-A 진행 중: Windows 33개 자동 검사·별도 타 계정 pipe ACL 검사 통과. 사용자 실기·제품 IPC 통합 미완료, D01~D06 위임 결정 반영.**

- main: `4c3537c55084b71feb5c42eb2340005bf922f197`. 브랜치 `task/t00-a-technical-spike`, [draft PR #1](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/pull/1).
- `spikes/t00-a/`의 Rust 공통 명령·엔진·독립 CLI·stdio MCP·Tauri/React/TS GUI·SQLite·합성 음원만 검증했다. 제품 스택·경로·수명은 아래 사용자 위임 결정으로 채택했으나 제품 구현·통합 검증은 하지 않았다. 제품 사운드 관리·API 연결·게임 적용 기능은 없다.
- 과거 검증 기준 코드 SHA `f8aa463bfd02fa84a10c39f32b353cf83d2c7872`의 [Windows CI](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/actions/runs/38809024678)에서 core 16 + lifecycle 11 + 실제 GUI 6 = **33개 자동 검사 통과**, 별도 named pipe ACL 검사도 통과했다. [artifact](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/actions/runs/38809024678/artifacts/12007313423)의 source SHA·전체 manifest 24개 hash·4개 PE x64를 확인했다. ZIP SHA256 `e7895dc45ebea7c7271e64e6dd4a10061a0fb443b871135aa70c1c80416b45a3`.
- 실제 WebView2 버튼 호출과 CLI 상태 일치, GUI 제출·native 창 종료 뒤 작업 지속·새 GUI 재연결, 합성 WAV 디코딩/재생 종료 이벤트를 확인했다. 스크린샷도 검토했다. **사용자 Windows 10/11 화면·스피커 청감은 미수령**이다.
- opt-in 자동 기동·기동 경쟁·busy 종료 거절·idle 종료/재기동·두 작업공간 독립성이 통과했다. Windows 백그라운드 엔진의 부모 stdio 핸들 상속으로 MCP EOF가 지연되던 결함을 수정했다. PowerShell 5.1 JSON 인자도 실제 실행 검사했다. idle 값은 fixture이며 사용자 전체 단일 엔진을 구현한 것이 아니다.
- dummy Credential Manager 쓰기/읽기/삭제, DPAPI CurrentUser 왕복 및 한글/공백 경로 FFmpeg/ffprobe·합성 원본 hash 보존 통과. 실제 Key·크레딧은 사용하지 않았다.
- 별도 native named pipe 후보에서 실제 다른 표준 로컬 계정의 SID로 허용 대조군 연결(0)·owner 전용 pipe 거절(5)·원래 신원 복귀를 확인했다. GitHub-hosted Windows runner에서만 임시 계정을 만들고 삭제 성공까지 검사했다. 일반 PC에서는 계정 생성 모드를 거절한다. 엔진은 여전히 TCP/token·작업공간 lock이며 엔진 IPC 통합·원격 차단은 미검증이다. 이전 익명 대조군·LocalService·계정 설명 제한 실패는 해당 SHA의 실패로 보존한다.
- 파일 확정/DB 기록 사이 종료에서 잔여 파일+interrupted를 재현했다. **파일/DB 자동 대조 복구는 미구현**이다. 직접 접근 대안 비교는 SQLite writer 잠금 fixture에 한정된다.

[정확한 SHA·명령·결과·재개 조건](docs/T00_A_RESULTS.md), [D01~D06 권고와 조건부 T00-B 계약](docs/T00_A_DECISIONS.md), [사용자 Windows 수집 절차](docs/T00_A_WINDOWS_CHECK.md).

11월 20일 재확인: 과거 ZIP은 만료됐지만, 새 [run 44713166683](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/actions/runs/44713166683) / [artifact 14618906066](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/actions/runs/44713166683/artifacts/14618906066)는 유효하며 2026-12-04 18:49:13 KST 만료 예정이다. source SHA는 `828e74540ae0aa0013d6ec3425389cb0631b0b1a`, ZIP SHA256은 `368d9d873265edade0fa7203d947353e032795144d26e0bac2c313cc98b8a631`다. ZIP CRC, manifest 전체 24개 파일 hash, 원시 보고서 4개의 source SHA, 실행 파일 4개의 PE AMD64 및 포장된 스크립트/README 9개의 Git 원문 일치를 검사했다. 이 새 run의 core 16 + lifecycle 11 + GUI 6과 별도 pipe ACL 성공을 보존했다. 새 GUI 캡처도 확인했지만 청감 증거는 아니다. [다운로드 무결성 영수증](docs/t00-a-evidence/refresh-20261120/artifact-inspection.json)과 원시 보고서를 따른다. PR Conversation/리뷰에는 실기 결과가 없고 현재 사용자도 화면/청감 결과를 제공하지 않아 실기는 미수령이다. 후속 사용자 메시지의 기술 판단 위임에 따라 D01~D06을 결정했다. 다음 작업은 아래 정확한 ZIP으로 사용자 화면·청감 결과를 받고 결정을 검토하는 것이다. [새 Codex 지시문](docs/NEXT_CODEX.md)을 따른다. 사용자 범위 엔진·계정 예산 원장·파일/DB 대조 복구의 제안과 후속 Task 경계는 결정 문서에 추가했다. 기술 선택은 사용자 위임으로 채택했지만 계정 예산 원장/파일 복구 등 후속 구현은 하지 않았다. [위임 결정 원문과 항목별 근거](docs/t00-a-evidence/refresh-20261120/delegated-decisions.json)를 보존한다. T00-A를 완료로 닫지 않으며 **T00-B 미착수**다.

API Key·크레딧·원격 생성, 실제 게임·음원 수정, 제품 라이선스·배포 확정, 직접 병합·Release/tag는 수행하지 않았다.

앞선 D01~D06 위임 반영 커밋은 문서와 선택한 CI 증거만 보존했다. Rust/UI/검사 스크립트/lockfile/워크플로를 바꾸지 않았고 Windows EXE를 Linux에서 실행하지 않았다. 문서 HEAD를 위 실행 SHA의 바이너리 검증으로 둔갑시키지 않는다. 제품 기능을 추가하거나 현재 게임을 변경하지 않았다.

위임 채택 요약: Tauri 2/React/TS + Rust/SQLite/FFmpeg/stdio MCP, Windows 11 x64 우선, 초기 포터블 ZIP·외부 런타임, LOCALAPPDATA 설정 + 사용자 선택 로컬 작업공간/원본 복사, Credential Manager(T02), SID별 local named pipe 엔진·최초 자동 기동·GUI 종료 후 작업 유지·busy 종료 거절·클라이언트/작업 0 상태 5분 idle 종료. 실제 Windows 사용자 화면·청감은 여전히 미수령이므로 T00-A를 완료로 닫지 않는다.

## AI 자동화 우선 후속 변경

사용자의 “ai가 자동화 가능한 작업을 위한 개발에 집중해. 실제 화면(UI)는 최소한도로” 지시를 PROJECT/REQUIREMENTS/ARCHITECTURE/TASKS에 반영했다. 각 기능에서 CLI·MCP 구조화 계약과 무인 회귀를 함께 만들고 T04는 연결 완성/호환 검증을 담당한다. 상태·실패·후보 재생/비교·설정·사람 승인에 필요한 최소 GUI만 우선한다.

T00-A 범위에 [증거 검사 CLI](spikes/t00-a/tools/README.md)를 구현했다. ZIP digest/CRC/manifest·report source SHA/33개 자동 검사·별도 ACL과 사용자 JSON source SHA/파일 hash/필수 결과를 읽기 전용 대조한다. 누락·불일치·후속 사람 검토 항목을 JSON과 종료 코드로 보고한다. Linux Python 3.14에서 회귀 12개 통과. 보관된 artifact 14618906066 ZIP으로 manifest 24개·CI 33개·별도 ACL 통과 및 사용자 두 JSON 미수령 → exit 2/pending을 확인했다. 전용 tools/와 문서/ignore만 변경했고 기존 Windows Rust/UI/collector/scripts/lockfile/워크플로는 바꾸지 않았다. 기존 Windows 실행 source SHA 828e745와 새 도구 커밋 SHA를 구분한다. 새 Python 도구의 Windows 실행은 미검증이다. 이 도구는 GitHub 조회·실행 주체/실제 청감 인증·D01~D06 채택·Task 완료·병합·T00-B 착수를 자동화하지 않는다.
