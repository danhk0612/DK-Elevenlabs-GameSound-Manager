# 현재 상태

기준일: 2026-10-16 KST. **T00-A 진행 중: Windows 33개 자동 검사·별도 타 계정 pipe ACL 검사 통과. 사용자 실기·제품 IPC 통합·설계 결정 미완료.**

- main: `4c3537c55084b71feb5c42eb2340005bf922f197`. 브랜치 `task/t00-a-technical-spike`, [draft PR #1](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/pull/1).
- `spikes/t00-a/`의 Rust 공통 명령·엔진·독립 CLI·stdio MCP·Tauri/React/TS GUI·SQLite·합성 음원만 검증했다. 제품 스택·경로·수명 기본값은 채택하지 않았다. 제품 사운드 관리·API 연결·게임 적용 기능은 없다.
- 검증 코드 SHA `f8aa463bfd02fa84a10c39f32b353cf83d2c7872`의 [Windows CI](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/actions/runs/38809024678)에서 core 16 + lifecycle 11 + 실제 GUI 6 = **33개 자동 검사 통과**, 별도 named pipe ACL 검사도 통과했다. [artifact](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/actions/runs/38809024678/artifacts/12007313423)의 source SHA·전체 manifest 24개 hash·4개 PE x64를 확인했다. ZIP SHA256 `e7895dc45ebea7c7271e64e6dd4a10061a0fb443b871135aa70c1c80416b45a3`.
- 실제 WebView2 버튼 호출과 CLI 상태 일치, GUI 제출·native 창 종료 뒤 작업 지속·새 GUI 재연결, 합성 WAV 디코딩/재생 종료 이벤트를 확인했다. 스크린샷도 검토했다. **사용자 Windows 10/11 화면·스피커 청감은 미수령**이다.
- opt-in 자동 기동·기동 경쟁·busy 종료 거절·idle 종료/재기동·두 작업공간 독립성이 통과했다. Windows 백그라운드 엔진의 부모 stdio 핸들 상속으로 MCP EOF가 지연되던 결함을 수정했다. PowerShell 5.1 JSON 인자도 실제 실행 검사했다. idle 값은 fixture이며 사용자 전체 단일 엔진을 구현한 것이 아니다.
- dummy Credential Manager 쓰기/읽기/삭제, DPAPI CurrentUser 왕복 및 한글/공백 경로 FFmpeg/ffprobe·합성 원본 hash 보존 통과. 실제 Key·크레딧은 사용하지 않았다.
- 별도 native named pipe 후보에서 실제 다른 표준 로컬 계정의 SID로 허용 대조군 연결(0)·owner 전용 pipe 거절(5)·원래 신원 복귀를 확인했다. GitHub-hosted Windows runner에서만 임시 계정을 만들고 삭제 성공까지 검사했다. 일반 PC에서는 계정 생성 모드를 거절한다. 엔진은 여전히 TCP/token·작업공간 lock이며 엔진 IPC 통합·원격 차단은 미검증이다. 이전 익명 대조군·LocalService·계정 설명 제한 실패는 해당 SHA의 실패로 보존한다.
- 파일 확정/DB 기록 사이 종료에서 잔여 파일+interrupted를 재현했다. **파일/DB 자동 대조 복구는 미구현**이다. 직접 접근 대안 비교는 SQLite writer 잠금 fixture에 한정된다.

[정확한 SHA·명령·결과·재개 조건](docs/T00_A_RESULTS.md), [D01~D06 권고와 조건부 T00-B 계약](docs/T00_A_DECISIONS.md), [사용자 Windows 수집 절차](docs/T00_A_WINDOWS_CHECK.md).

다음 작업은 정확한 SHA의 사용자 화면·청감 결과를 수집하고 D01~D06을 검토하는 것이다. 사용자 범위 엔진·계정 예산 원장·파일/DB 대조 복구의 제안과 후속 Task 경계는 결정 문서에 추가했다. 이 제안은 미채택이며 제품 구현이 아니다. D01~D06은 모두 사용자 미결정이다. T00-A를 완료로 닫지 않으며 **T00-B 미착수**다.

API Key·크레딧·원격 생성, 실제 게임·음원 수정, 제품 라이선스·배포 확정, 직접 병합·Release/tag는 수행하지 않았다.
