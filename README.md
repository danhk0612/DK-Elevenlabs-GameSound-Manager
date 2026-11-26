# DK-Elevenlabs-GameSound-Manager

Windows용 범용 게임 사운드 제작·관리 프로그램. BGM, SFX, 캐릭터 Voices를 게임 프로젝트별로 관리하며, 사람은 GUI, 외부 AI는 CLI·MCP로 같은 작업 규칙을 사용합니다.

## 시작하기

[Windows 11 시작 안내](app/START_HERE.md) · [CLI·MCP 설정과 실행](app/README.md)

- 사람은 남색 Fluent GUI에서 프로젝트와 원본 음원을 등록하고, AI가 제출한 후보를 재생·비교한다.
- AI는 같은 프로젝트에 CLI JSON 또는 stdio MCP로 접근한다. 필요한 사운드 등록, 후보 제작·분석, 검토 제출과 일괄 작업을 지원한다.
- 실제 SFX와 짧은 음성 생성은 확인했다. 음악 생성 성공과 사람 승인 후 게임 적용·복구는 남은 작업이다.

## 개발 문서

| 문서 | 내용 |
| --- | --- |
| [PROJECT.md](PROJECT.md) | 목적, 사용 흐름, 범위 |
| [REQUIREMENTS.md](REQUIREMENTS.md) | 확정 요구사항, 완료 조건, 미결정 사항 |
| [ARCHITECTURE.md](ARCHITECTURE.md) | 권장 구조와 기술 검증 항목 |
| [CURRENT_STATE.md](CURRENT_STATE.md) | 실제 구현·검증·차단 상태 |
| [TASKS.md](TASKS.md) | 작업 순서와 다음 Work 시작 지시문 |
| [AI_WORKFLOW.md](AI_WORKFLOW.md) | 작업 환경 분담과 인수인계 |
| [API_RESEARCH.md](API_RESEARCH.md) | 공식 API 근거와 확인 한계 |

2026-11-20 사용자 지시로 **AI 자동화 우선·최소 UI**, **실기/점검 대기 해제**를 반영했습니다. `app/`는 제품 실행 기반이고 `spikes/t00-a/`는 과거 격리 실험입니다. D01~D06 위임 선택을 유지하며 미수령 Windows 사용자 화면·청감을 성공으로 기록하지 않습니다. [현재 구현/빌드 상태](CURRENT_STATE.md), [현재 결정](docs/T00_A_DECISIONS.md), [과거 검증 증거](docs/T00_A_RESULTS.md)를 구분합니다.

제품 라이선스와 공개 배포 정책은 미결정입니다. 이 저장소에는 API Key, 사용자 음원, 게임 데이터, 빌드된 외부 바이너리를 저장하지 않습니다.
