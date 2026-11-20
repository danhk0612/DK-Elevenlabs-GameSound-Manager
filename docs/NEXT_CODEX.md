# 다음 Codex — 자동화 우선 구현 계속

## 현재 유효한 사용자 지시

- “ai가 자동화 가능한 작업을 위한 개발에 집중해. 실제 화면(UI)는 최소한도로.”
- 후속 “실기, 점검 무시하고 진행해”. 이전 T00-A 실기/점검 대기·T00-B 미착수 지시를 해제했다. 실기 자료를 다시 요청하며 멈추지 않는다. 미확인을 성공으로 기록하지 않는다.
- D01~D06 위임 채택은 유지한다. 병합·Release/tag·공개 배포·실제 Key/크레딧 사용·현재 게임/실제 음원 변경은 별도다.

## 재개 지시문

```text
danhk0612/DK-Elevenlabs-GameSound-Manager에서 자동화 우선 개발을 계속해.
최신 main·작업 브랜치·열린 draft PR #1·작업 트리를 확인하고 AGENTS.md,
PROJECT.md, REQUIREMENTS.md, ARCHITECTURE.md, CURRENT_STATE.md, TASKS.md,
AI_WORKFLOW.md, app/README.md와 D01~D06 결정을 읽어.

현재 app/는 T00-B의 공통 version/status/shutdown, SID별 Windows 엔진,
독립 CLI·stdio MCP·최소 상태 GUI다. spikes/t00-a는 과거 격리 실험으로 유지해.
실기/점검 대기로 개발을 막지 말고 기존 확인 결과와 미확인을 구분해 기록해.
T00-B Windows 빌드 실패가 있으면 먼저 구현 오류를 수정하고,
그 외에는 다음 T01 프로젝트·로컬 사운드 관리로 진행해.

각 기능의 공통 명령·CLI/MCP 구조화 입출력·오류·무인 동작을 함께 만든다.
GUI는 필수 상태/실패·재생/비교·설정·사람 승인 범위로 최소화한다.
원본/승인본을 보존하고 실제 사용자 자료는 테스트 입력으로 사용하지 마.
T01에서는 API 생성·유료 Key·게임 적용 기능을 만들지 마.

기존 작업 브랜치와 draft PR #1을 재사용하고 코드/상태/Task/문서를 반영해.
push·PR 갱신·빌드 CI는 허용한다. 직접 병합·Release/tag·공개 배포는 하지 마.
```

[현재 기반 사용법](../app/README.md), [현재 상태](../CURRENT_STATE.md), [Task 계약](../TASKS.md).
과거 Windows CI 33개·별도 pipe ACL 증거와 미수령 사용자 실기는 [T00-A 결과](T00_A_RESULTS.md)에 보존한다. 과거 실행 SHA와 새 제품 기반의 SHA를 혼동하지 않는다.
