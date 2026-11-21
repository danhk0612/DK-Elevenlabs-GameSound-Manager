# 다음 Codex — 자동화 우선 구현 계속

## 현재 유효한 사용자 지시

- “ai가 자동화 가능한 작업을 위한 개발에 집중해. 실제 화면(UI)는 최소한도로.”
- 후속 “실기, 점검 무시하고 진행해”. 이전 T00-A 실기/점검 대기·T00-B 미착수 지시를 해제했다. 실기 자료를 다시 요청하며 멈추지 않는다. 미확인을 성공으로 기록하지 않는다.
- D01~D06 위임 채택은 유지한다. 병합·Release/tag·공개 배포·실제 Key/크레딧 사용·현재 게임/실제 음원 변경은 별도다.

## 최신 사용자 요청 — Windows/NAS 설정 파일

사용자는 Windows에서 NAS 파일을 편집하고 Codex CLI는 Docker인 환경이므로 key/한도를 파일에 넣도록 요청했다. [설정 안내](WINDOWS_NAS_SETUP.md)의 명시적 --config를 지원한다. 최상단 elevenlabs.local.json은 실제 편집 파일이며 Git 제외다. Git 예제 elevenlabs.config.example.json에는 절대 실제 key를 넣지 않는다. 기존 '평문 파일 입력 금지'는 이 최신 명시적 요청이 우선한다. 파일을 cat/내용 출력하지 말고 --check-config로 검사한다. user가 설정 완료를 알리면 같은 config로 Docker-local private runtime의 engine 세션을 열고 계정/명시 한도를 확인한다. 원본 설정을 덮어쓰거나 설정 한도를 임의 증액하지 않는다. 실제 서비스 호출/비용은 설정된 허용·범위 내에서만 진행한다. 기존 stdin/FD/Windows Credential Manager·원본 보존·T05 경계는 유지한다. 정확한 최신 SHA/CI는 CURRENT_STATE.md를 대조한다.

## 최신 후속 우선순위 — Linux 생성 코드 구현 이후

v0.9 code 32828ea/run 44834877114의 Windows·Linux CI는 모두 성공했다. 사용자의 후속 “진행해”로 v0.10 [Linux 인증 세션/HTTPS](T02_LINUX_PROVIDER.md)를 구현했다. engine --session 전용 private stdin·메모리 account/key/TTL/허용·한도와 숨김 입력/private key FD 런처, fixed-origin reqwest/rustls 전송이 공통 큐에 연결된다. 일반 CLI/MCP에는 key·live 정책 변경 명령을 추가하지 않는다. default autostart는 키 없이 유지한다. 원본/후보/예산·단일 POST/불확실 상태·T05 경계를 유지한다.

source 54d2fbf2fa5809b251857aac23527743be7afff8/run 44839231602는 Windows·Linux 전체 성공했다. 이후 HEAD 변경이 있으면 CURRENT_STATE.md와 정확한 source/CI를 대조하고 실패가 있으면 먼저 수정한다. Rust 43개(실제 FFmpeg 포함)·Python 18개와 실제 Linux metadata-only 인증 세션/재시작/무저장·로컬 TLS 회귀를 확인했다. 실제 ElevenLabs 요청/생성·요금제/비용·청감은 미검증이다. 사용자의 Key·서비스 quota·허용 범위가 안전한 세션으로 준비되기 전에는 라이브 요청을 하지 않는다. 키를 채팅/인자/환경 변수/평문 JSON 파일로 요구하지 않는다. 준비 전에도 무인 실행 경로와 기존 한도/전송/중단 회귀를 개발할 수 있다. 실제 키 없이 또 다른 wrapper/UI만 늘려 실제 생성 완료라고 보고하지 않는다. 아래 v0.8 지시문은 과거 기반이며 이 우선순위가 앞선다.

## 재개 지시문

```text
danhk0612/DK-Elevenlabs-GameSound-Manager에서 자동화 우선 개발을 계속해.
최신 main·작업 브랜치·열린 PR #1·작업 트리를 확인하고 AGENTS.md,
PROJECT.md, REQUIREMENTS.md, ARCHITECTURE.md, CURRENT_STATE.md, TASKS.md,
AI_WORKFLOW.md, app/README.md와 D01~D06 결정을 읽어.

현재 app/는 T04 다건 작업 정의 보강 v0.8.0, SID별 Windows 엔진,
독립 CLI·MCP 46개 도구·최소 연결/목록/재생/저장 검사 GUI다.
docs/T01_LOCAL_LIBRARY.md, docs/T02_GENERATION.md, docs/T03_AUDIO.md와 docs/T04_ASYNC_AUDIO.md와 docs/T04_REVIEW.md와 docs/T04_AUTOMATION.md와 docs/T04_BATCH.md를 읽어. spikes/t00-a는 과거 격리 실험으로 유지해.
실기/점검 대기로 개발을 막지 말고 기존 확인 결과와 미확인을 구분해 기록해.
최신 Windows 빌드 실패가 있으면 먼저 구현 오류를 수정하고,
그 외에는 v0.7 코드 3f47e5852942dd28d951354484ecd8d37107e582의 Windows run 44751508856은 실제 engine CLI/MCP 독립 기동·작업 정의 재호출/재연결·원본 보존 및 개발 패키징까지 성공했다. 초기 timeout은 inherited stdout reader 정리 수정으로 해소됐다. 최신 실행 상태를 대조하고 v0.8 코드 6af11ecf00b59554a4ef98b725e5e48bd3a528a2의 Windows run 44830379629는 Python 18개·실제 batch 선택/공유 source/CLI·MCP parity/JSONL/fail-stop·keep-going/보존과 개발 패키징까지 성공했다. sound_batch.py의 전체 사전 검사·ID 충돌·JSONL·선택 재호출·확정 오류 keep-going을 유지하고 최신 head 변경에 맞는 CI 상태를 대조해. 이후 지원 AI 제품 클라이언트 호환·필요한 사운드/작업 이력 등 T04 자동화 보강으로 진행해. app/automation의 선택형 Python 클라이언트와 CLI offline validate를 유지해. 자체 stdio 클라이언트를 AI 제품 연결 성공으로 취급하지 마. 검토 제출/철회·현재 일치 확인·후보 이력은 이미 구현했다. pending을 승인으로 취급하지 마. async audio.submit/get/list/cancel과 기존 동기 명령을 유지하고 원본 보존·snapshot·취소/등록 경계·timeout·복구를 보강해. CLI/MCP native stdio smoke를 실제 AI 클라이언트 연결 성공으로 확대하지 마. D08 기본값은 임의 확정하지 말고 호출의 명시적 출력/검사 조건을 유지해. T05 승인/게임 적용(D07)은 별도 설계 범위이며 착수하지 마. T02는 라이브 미검증/원격 preview 재생 미구현을 남겨 두되 실기 대기로 개발을 멈추지 마. 실제 Key/허용 비용이
없으면 모의 응답과 오프라인 경로를 구현하고 라이브 생성은 하지 마.
D09는 아직 미결정이므로 미설정 한도를 무제한으로 해석하지 마.

각 기능의 공통 명령·CLI/MCP 구조화 입출력·오류·무인 동작을 함께 만든다.
GUI는 필수 상태/실패·재생/비교·설정·사람 승인 범위로 최소화한다.
원본/승인본을 보존하고 실제 사용자 자료는 테스트 입력으로 사용하지 마.
실제 Key를 인자/로그/DB/Git에 노출하지 말고 실제 게임 적용은 하지 마.

기존 작업 브랜치와 PR #1을 재사용하고 코드/상태/Task/문서를 반영해.
push·PR 갱신·빌드 CI는 허용한다. 직접 병합·Release/tag·공개 배포는 하지 마.
```

[현재 기반 사용법](../app/README.md), [현재 상태](../CURRENT_STATE.md), [Task 계약](../TASKS.md).
과거 Windows CI 33개·별도 pipe ACL 증거와 미수령 사용자 실기는 [T00-A 결과](T00_A_RESULTS.md)에 보존한다. 과거 실행 SHA와 새 제품 기반의 SHA를 혼동하지 않는다.
