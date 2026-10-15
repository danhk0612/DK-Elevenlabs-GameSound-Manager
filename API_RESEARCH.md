# ElevenLabs 및 기술 근거

확인일: 2026-10-15. 아래 URL은 공식 페이지를 직접 열어 접근 가능 여부와 내용을 확인했다. 문서 확인이며 계정 인증·실제 API 호출 성공 증거가 아니다. 모델·요금·필드는 구현 시 다시 확인한다.

## 핵심 API

| 기능 | 공식 문서에 기재된 경로 | 근거 | 프로젝트 상태 |
| --- | --- | --- | --- |
| 계정 저장 Voice 목록 | GET /v2/voices | [List voices](https://elevenlabs.io/docs/api-reference/voices/search) | 문서 확인, 실제 계정·페이지네이션 미검증 |
| 구독·사용량 | GET /v1/user/subscription | [Get user subscription](https://elevenlabs.io/docs/api-reference/user/subscription/get) | tier·character_count·character_limit 등 확인, 실제 계정 미검증 |
| TTS | POST /v1/text-to-speech/{voice_id} | [Create speech](https://elevenlabs.io/docs/api-reference/text-to-speech/convert) | 문서 확인, 생성 미실행 |
| SFX | POST /v1/sound-generation | [Create sound effect](https://elevenlabs.io/docs/api-reference/text-to-sound-effects/convert) | 문서 확인, 생성 미실행 |
| API Key | xi-api-key 인증 | [Authentication](https://elevenlabs.io/docs/api-reference/authentication) | Key별 scope·credit quota 문서 확인, 자격증명 미수령 |

Voice 목록 응답에는 voice_id·preview_url·관련 계정/등급 정보가 있다. 목록 포함 여부가 모든 모델·모든 설정으로 생성 가능함을 보장하지 않는다. 계정에 실제 저장된 Voice ID로 TTS가 되는지 T02에서 확인한다. Voice 상세·설정의 개별 endpoint, 모델별 호환성은 T02에서 공식 문서와 계정 응답으로 추가 검증한다.

SFX 문서는 모델·duration_seconds·loop와 출력 형식의 제약을 설명한다. 출력 품질 옵션에는 요금제 제한이 있다. 특정 길이·모델·출력 형식을 제품 기본값으로 확정하지 않았다. 서비스 제약과 로컬 변환 옵션을 구분한다.

구독 정보의 필드 이름을 모든 생성 종류의 정확한 비용 산식으로 해석하지 않는다. 사용량 추정·예약·실제 소모·외부 사용·응답 유실을 구분해야 한다. API Key quota는 앱 내 한도를 대체하지 않으며 앱이 계정 전체의 추가 과금을 완전히 방지한다고 보장할 수 없다.

## 후속 기능과 계정 검증 한계

[Voice Design](https://elevenlabs.io/docs/api-reference/text-to-voice/design) 공식 페이지가 검색되었으나 이번 단계에서 미리보기→저장→재사용 전 과정을 확인하지 않았다. Remix·Voice 원격 수정·원격 생성 이력·Music API의 상세 endpoint·요금제 제한은 미검증이다. 구현됐다고 취급하지 않는다.

[공식 요금표](https://elevenlabs.io/pricing)는 T02에서 실제 계정 조건과 함께 다시 확인한다. 이번 단계는 무료 계정에서 모든 API 기능을 사용할 수 있다는 결론을 내리지 않는다. 실제 사용자 Key·크레딧을 사용하지 않았으며 원격 생성·Voice 변경·삭제를 수행하지 않았다.

## 외부 기술 근거

| 근거 | 확인한 사실 | 남은 실측 |
| --- | --- | --- |
| [Tauri prerequisites](https://tauri.app/start/prerequisites/) | Windows 개발에 C++ Build Tools·WebView2·Rust 준비 필요 | Windows 빌드·재생·배포 |
| [Tauri process model](https://tauri.app/concept/process-model/) | core와 WebView 프로세스 구조 | 별도 엔진/CLI 수명과 IPC |
| [SQLite WAL](https://www.sqlite.org/wal.html) | reader/writer 동시성, 단일 writer, 로컬 공유 메모리 제약 | 큐·파일 상태·종료 복구 |
| [MCP stdio](https://modelcontextprotocol.io/specification/2026-07-28/basic/transports/stdio) | stdin/stdout JSON-RPC, 로그 stderr | 협상 버전·클라이언트 호환 |
| [FFmpeg legal](https://www.ffmpeg.org/legal.html) | 빌드 구성에 따라 LGPL/GPL 조건이 다름 | 사용할 바이너리·소스·고지·배포 조건 |

제품 라이선스와 FFmpeg 바이너리의 라이선스는 별도 결정이다. 바이너리·정확한 빌드 설정을 선정하지 않았으므로 현재 재배포 적합 판정을 내리지 않는다.
