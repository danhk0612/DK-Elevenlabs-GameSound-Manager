# DK-Elevenlabs-GameSound-Manager

Windows용 범용 게임 사운드 제작·관리 프로그램. BGM, SFX, 캐릭터 Voices를 게임 프로젝트별로 관리하며, 사람은 GUI, 외부 AI는 CLI·MCP로 같은 작업 규칙을 사용합니다.

**현재 v0.3.0: 로컬 관리와 T02 계정/한도·TTS/SFX 후보 생성 경로를 구현했습니다.** 공통 명령 32개·MCP 31개 도구, 최소 연결 설정/목록/재생 GUI를 제공합니다. Key 없는 mock 회귀를 확인했으며 실제 유료 API 호출은 하지 않았습니다. [생성 계약과 예시](docs/T02_GENERATION.md)를 확인하세요. 편집·게임 적용은 후속 단계입니다. [빌드와 실행](app/README.md)을 확인하세요.

ElevenLabs TTS·SFX 연동과 외부에서 만든 로컬 사운드 관리를 계획합니다. AI의 생성·편집·검증 결과는 후보로 보관하며, 실제 게임에 적용하려면 사람의 명시적 승인이 필요합니다.

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
