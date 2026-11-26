# T02 Linux 실제 SFX 생성 확인 — 2026-11-21 UTC

사용자가 NAS 설정 파일에 API key·생성 허용/한도를 입력하고 진행을 지시했다. 내용을 출력하지 않고 --check-config로 검사한 후 기존 공통 CLI/엔진으로 실행했다. 실제 key·계정 identity·원격 응답 전문·사용자 설정·음원은 Git에 넣지 않는다.

## 확인 결과

- 실제 account.create/get/refresh: Linux 메모리 인증 세션과 /v1/user 인증·workspace 연결 성공.
- 실제 POST **1회**: SFX model eleven_text_to_sound_v3, duration_ms=500, mp3_44100_128, 단일 dry wooden UI click 프롬프트. reserve_credits=200을 명시했으며 사용자 설정 한도를 넘지 않았다. 서비스 자동 재시도/대체 생성 없음.
- job live-sfx-check-20261121-01: succeeded/request_attempted=true, character-cost 응답 헤더 **5**. ledger는 기존 보수 정책으로 예약 **200**을 계속 보유하며 5로 자동 환급/리셋하지 않는다. 서비스 usage 조회의 character_count는 직후 0이어서 이 값으로 최종 청구/비용 반영 완료를 주장하지 않는다.
- ready candidate MP3 8,820 bytes, SHA-256 `13ce7c3f028bd67bc3c517d3ddf1d7bc35c6bacea992a767e84db8f590cf0079`.
- 실제 FFmpeg/ffprobe 7.0.2 async 분석 성공: MP3/stereo/44100Hz, decoded duration 480ms, sample peak 약 -7.42dBFS/RMS 약 -31.99dBFS, 명시 clip_level=0.999 초과 sample 0. 실제 청감/장면 적합성은 평가하지 않았다.
- review pending, candidate/analysis 현재 일치 true, approval not_granted. 기술 검사/검토 대기는 게임 적용 승인이 아니다.
- 정상 종료→같은 설정의 새 엔진→동일 job_id/요청 재호출로 기존 succeeded 결과를 반환했다. generation_requests_held는 **1**을 유지했다. 추가 POST/음원 덮어쓰기 없음.

NAS의 Git 제외 user-data/live-check에 ui-wood-click.mp3, analysis.json, receipt.json을 보관했다. 원래 Docker managed candidate는 유지하며 게임 파일/사이트/운영 DB에 쓰지 않았다. 실제 음원은 개발 패키지/CI/Git에 포함하지 않는다. CI는 계속 가짜 키·합성 자료만 사용한다.

## 실행 중 발견한 호환성 보완

이 Docker에서 ~/.local/state를 생성할 권한이 없어 첫 engine launch가 session_start_failed로 종료됐다. 네트워크/생성 전의 로컬 경로 실패였다. 사용자의 NAS 설정은 수정하지 않았다.

빈 state_root의 기본 경로가 **존재하지 않고** 부모 mkdir가 PermissionError면 OS temporary directory의 dk-gamesound-<effective UID>로 선택하도록 수정했다. 정적 stderr 메시지만 출력하고 UID/0700/socket 확인은 기존 engine에 맡긴다. 기존 기본 root/ledger가 있거나 명시 state_root이면 경로를 조용히 바꾸지 않고 실패한다. 일시적 root의 상태/예산은 container/temp 정리로 소실될 수 있으므로 장기 운용은 쓰기 가능한 private 지속 volume의 절대 경로를 설정한다. 자동 예산 초기화/다른 ledger로 이동은 추가하지 않는다.

status.transport가 Linux에서도 Windows pipe 문자열을 출력하던 표시를 linux_unix_socket_owner_uid_local_only로 수정했다. Windows 문자열/공통 명령·schema는 유지한다.

## 증거 범위

작업 기반은 068563078d04a53f715d7db94aa1cac6452c17e4이며 이 세션에서 위 경로 보완을 적용한 launcher와 기존 v0.10 Linux HTTPS 엔진으로 실제 POST 1회를 실행했다. transport 표시 수정으로 다시 빌드한 engine에서 동일 ID 재조회/중복 무생성을 확인했다. 정확한 최종 코드 SHA/CI는 CURRENT_STATE.md를 따른다.

로컬 Python 23개(기존 22+기본 경로 실패/기존 ledger/명시 경로 보호 1개), 실제 engine NAS config/FD 세션 smoke, Rust 기본 36개/외부 도구 7개 ignore, native bins·Windows x64 타입 검사·fmt/diff를 확인했다. 실제 생성 결과의 디코딩·분석은 위 별도 라이브 증거다. TTS/다른 Voice/Windows 사용자 실제 인증·청감·최종 과금 정산·지원 AI 제품 MCP 등록/T05는 이번 증거에 포함하지 않는다.

기존 전용 temp root가 있으면 기본 home의 쓰기 권한이 회복돼도 그 root를 재사용한다. home/temp 기본 root가 둘 다 있으면 암묵적으로 하나를 선택하지 않고 명시 경로를 요구한다. 새 ledger로 조용히 갈아타 한도가 초기화되는 상황을 막는다.
