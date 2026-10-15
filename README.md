# DK-Elevenlabs-GameSound-Manager

Windows용 범용 게임 사운드 제작·관리 프로그램. BGM, SFX, 캐릭터 Voices를 게임 프로젝트별로 관리하며, 사람은 GUI, 외부 AI는 CLI·MCP로 같은 작업 규칙을 사용합니다.

**현재는 초기 설계 단계입니다. 실행 프로그램, 설치 파일 및 사운드 관리 기능은 아직 없습니다.**

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

다음 작업은 **T00-A: Windows 기술 검증 및 설계 결정 준비**입니다. 기술 스택은 제안 상태이며 최종 사용자 결정과 검증을 거쳐 채택합니다.

제품 라이선스와 공개 배포 정책은 미결정입니다. 이 저장소에는 API Key, 사용자 음원, 게임 데이터, 빌드된 외부 바이너리를 저장하지 않습니다.
