# 새 Codex 인수인계 — T00-A 마무리

아래 지시문을 고성능 Codex에서 실행한다. 이 문서는 T00-B 구현 승인이 아니다.

```text
danhk0612/DK-Elevenlabs-GameSound-Manager의 T00-A만 이어서 진행해.

최신 main·작업 브랜치·열린 PR·작업 트리를 먼저 확인해.
기존 task/t00-a-technical-spike와 draft PR #1을 재사용해.
main이 초기 준비 상태라면 작업 브랜치의 CURRENT_STATE.md와 TASKS.md를
진행 상태 기준으로 사용해. AGENTS.md와 루트 기준 문서, API_RESEARCH.md,
docs/T00_A_RESULTS.md, docs/T00_A_WINDOWS_CHECK.md, docs/T00_A_DECISIONS.md를 읽어.

1. PR과 사용자 메시지에 Windows 실기 결과와 명시적인 D01~D06 결정이 있는지 확인해.
   과거 자동 검사 33개와 별도 pipe ACL 성공은 그대로 보존해.
   실기 결과가 없다면 최신 검증 가능 artifact와 명령을 제공하고 미수령으로 남겨.
2. 실기 결과는 실행한 ZIP의 workflow run head, source-sha.txt, SHA256SUMS.txt와
   user-automatic.json 및 user-observation.json의 SHA를 대조해.
   Windows 버전·실행 결과, GUI 상태 조회·제출·닫기/재연결,
   합성 WAV가 실제로 들렸는지 각각 확인해. 빈 항목은 미검증이야.
   artifact가 만료됐다면 현재 코드와 워크플로의 차이를 확인한 뒤
   같은 검증 브랜치에서 CI를 다시 실행하고 새 run과 SHA를 안내해.
   날짜가 다른 ZIP의 SHA나 해시를 재사용하지 마.
3. D01~D06을 결정 문서의 후보와 근거로 검토하고 사용자 결정만 반영해.
   별도 named pipe ACL 성공은 엔진 IPC 통합·원격 차단 검증이 아니야.
   D06에서 제품 IPC, 사용자 범위 소유권과 수명 정책을 확인하고,
   실험의 TCP/token이나 fixture 값을 제품 기본값으로 승격하지 마.
4. 증거를 CURRENT_STATE.md·TASKS.md·관련 docs에 기록하고 PR #1을 갱신해.
   상태와 결정이 모두 준비되면 T00-A 완료 조건을 대조하고 후속
   T00-B 최소 version/status 구현 지시문을 제시해. 이번 작업에서는 착수하지 마.

기존 브랜치에 필요한 문서 반영·push·PR 갱신·검증 CI는 허용해.
직접 병합·Release/tag·제품 공개 배포·T00-B 구현·실제 Key/크레딧 사용·
실제 게임/음원 변경은 하지 마. 실기나 결정이 없으면 완료로 닫지 마.
```

## 검토할 결정

| ID | 사용자에게 제시할 선택 |
| --- | --- |
| D01 | 실기 결과를 바탕으로 Tauri 2/React/TypeScript + Rust, SQLite, FFmpeg, stdio MCP 후보를 채택할지 |
| D02 | 실기 확인 OS·아키텍처와 제품 지원 범위. Windows 11 x64에서 성공해도 Windows 10 지원 증거는 별도 |
| D03 | 실험 ZIP과 제품 제공 방식 분리. 설치형/포터블 및 런타임 동봉을 언제 정할지 |
| D04 | 앱 설정과 사용자 지정 작업공간 분리, 원본 복사 정책과 경로의 세부 결정 시점 |
| D05 | Credential Manager 우선 권고와 DPAPI 대안, 제품 인증정보 구현 시점 |
| D06 | 사용자별 단일 엔진, 제품 IPC, 자동 기동, GUI 종료 후 작업 유지, busy/idle 종료 기준 |

권고는 결정으로 간주하지 않는다. 기술 판단은 고성능 Codex에서 마무리하고, 채택 범위가 명확해진 T00-B 구현은 별도 저성능 Codex에서 진행한다.

## 2026-11-20 재개용 증거 갱신

위 지시문 전체를 유지한다. 새 [artifact 내부 검증](t00-a-evidence/refresh-20261120/artifact-inspection.json)을 완료했고, [정확한 실기 ZIP·명령](T00_A_WINDOWS_CHECK.md)을 제공했다. 검증 source SHA는 `828e74540ae0aa0013d6ec3425389cb0631b0b1a`다. 후속 문서 HEAD와 실행 SHA를 구분한다. 사용자 JSON/청감은 미수령이다. 후속 사용자가 기술 선택을 위임해 D01~D06을 채택했으며 [구조화 결정 기록](t00-a-evidence/refresh-20261120/delegated-decisions.json)을 먼저 읽는다. 위 원문 지시 중 사용자 결정 확인은 이 위임 기록으로 대조하고 실기 게이트는 그대로 유지한다. 다음 실행은 PR/사용자 메시지의 새 결과부터 다시 확인하고, 없다면 이 게이트를 닫거나 T00-B에 착수하지 않는다.
