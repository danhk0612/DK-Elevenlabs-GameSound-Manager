# 음악·짧은 게임 음성과 최소 제작 예산

2026-11-21 사용자 확정: 음악과 짧은 기합/반응을 추가하며 월 10,000크레딧에서 필수분부터 만든다. 게임 개발/적용은 재개하지 않는다. AI 우선 CLI/MCP·최소 GUI 원칙을 유지한다.

## 공통 계약

기존 `job.submit`에 generation_kind=music을 추가한다. 공통 47개 명령/MCP 46개 도구·DB schema 5/1·큐/예산·후보 보존을 유지한다.

| 필드 | 음악 계약 |
| --- | --- |
| text | 프롬프트, 1~4,100 Unicode 문자/16,400 bytes 이하 |
| model_id | 초기 music_v1만 지원 |
| output_format | mp3_44100_128 |
| duration_ms | 명시 정수 3,000~600,000; SFX는 기존 500~30,000 유지 |
| force_instrumental | 명시 boolean; 초기 제작 true |
| 기타 | project/account/job/asset/version/name/reserve_credits는 기존 필수 필드 |

음악에 voice_id/voice_settings/language/dialogue_id/loop/prompt_influence는 허용하지 않는다. `/v1/music?output_format=mp3_44100_128`에 `{prompt,model_id,music_length_ms,force_instrumental}`을 **한 번** POST한다. 성공 후보 kind는 bgm, TTS는 voice, SFX는 sfx다. 원본/부모·타깃 예약/해시·복구/사람 승인 계약을 유지한다.

실패 HTTP의 status 숫자만 job.get.result.provider_http_status에 보존한다. 원격 오류 본문은 보존/출력하지 않는다. 변경 전 실패에는 소급하지 못한다. 기존 timeout 120초/응답 32 MiB를 유지하며 긴 음악의 완료/루프·음질을 보장하지 않는다. 실패/timeout은 자동 재전송하지 않는다. Music v2/v2.5·composition plan·구간 편집은 미지원이다.

[공식 Music endpoint](https://elevenlabs.io/docs/api-reference/music/compose), [TTS endpoint](https://elevenlabs.io/docs/api-reference/text-to-speech/convert)를 기준으로 구현했다. mock 계약 검증과 실제 Music 성공을 구분한다.

## 짧은 음성 타입

| ID | 연출 의도 | 공격 | 피격 |
| --- | --- | --- | --- |
| energetic | 활기찬 | 이얍! | 앗! |
| calm | 차분한 | 핫! | 윽! |
| sturdy | 묵직한 | 으랏! | 으윽! |

게임용 타입을 명시 Voice ID에 연결하고 재사용한다. 실제 버튜버의 성별·성격·음성을 추정/복제하지 않는다. 기존 character.put/dialogue.put과 TTS job.submit, model=eleven_multilingual_v2/language=ko를 사용한다. 타입명은 연출 의도이며 선택 Voice의 청감 합격이 아니다. 발음/감정/길이는 청감 pending이며 자동 반복 후보를 만들지 않는다. Voice Design/원격 Voice 변경·삭제는 없다.

후속 게임 연동 시 주요 공격/피격에 선택적으로 사용한다. 재생 빈도/겹침 제한·음소거는 게임 후속 작업이다. 이번에는 게임 파일을 수정하지 않는다.

## 오프라인 최소 제작 계획기

`app/automation/sound_starter.py`는 **작업을 제출하지 않는다**. 선택 항목 예약량을 합산하고 전체 명령을 native CLI validate로 검사한 후 JSON을 출력한다. 엔진 기동/HTTP/과금 없이 동작한다. 기존 프로젝트가 필요하다.

```sh
python app/automation/sound_starter.py --project demo --account elevenlabs-main \
  --run starter-first --max-reserved 2000 \
  --voice energetic=VOICE_ID_A --voice calm=VOICE_ID_B --voice sturdy=VOICE_ID_C \
  --cli /absolute/path/to/dk-sound-cli
```

Windows에서는 Python/CLI 로컬 경로를 사용한다. --only music-battle 또는 --only energetic-attack을 반복해 선택할 수 있다. 미선택 타입 Voice는 없어도 된다. 중복/알 수 없는 항목·Voice 타입·선택 Voice 누락·예약 초과·native 검증 실패는 제출 전에 거절한다.

- 모험/일반 전투 BGM 30초씩, reserve 각각 750.
- 음성 3타입 × 공격/피격, reserve 각각 50.
- 전체 8개 예약 **1,800**, 초기 계획 상한 **2,000**. 보스곡·승리곡·긴 대사·추가 take는 후속 제작.
- 예약은 계획치이며 실제 청구 상한이 아니다. [공식 요금표](https://elevenlabs.io/pricing)의 근사값도 실제 계정·모델/API 조건·응답과 구분한다. 정확한 비용 사전 견적 기능은 없다.
- 월 10,000 사용자 예산과 기존 설정을 유지한다. ledger/설정 한도를 증액하거나 매월 자동 초기화하지 않는다.
- 출력 순서의 character/dialogue 등록 후 job.submit을 CLI request/MCP sound_job_submit에 전달한다. 호출자는 매번 결과를 확인하고 다음 요청을 결정한다. 자동 일괄 제출기는 아니다.
- 같은 run/입력은 같은 ID다. 동일 ID는 기존 결과를 반환하며 실패를 다시 생성하지 않는다. 새 run도 기존 asset/take-1을 덮어쓸 수 없다. 추가 take는 기존 명시 parent/version 계약으로 별도 후보를 만든다.
- Key는 출력/CLI/MCP에 넣지 않는다. 실제 Voice ID·음원·계획은 Git 제외 user-data에 둔다.

## 실제 결과/한계

생성 전 service_usage count=5/limit=10,000, 기존 ledger request 1/예약 200이었다.

- 음악 starter-20261121-music-adventure POST **1회 실패**, provider_http_error/확인 비용 null. 당시 status를 저장하지 않아 원인은 미확인이다. 요금제/권한/요청 문제를 단정하지 않는다. **재시도·대체 모델·두 번째 곡 호출 없음**, 예약 750 유지.
- TTS **6회 성공**, header 비용 3+2+2+2+3+3=**15**, 길이 약 0.601~0.914초. MP3 전부 ffprobe/FFmpeg 전체 디코딩 성공. 직후 /v1/user는 count=5여서 월 잔액/최종 청구 반영 완료를 주장하지 않는다.
- 누적 ledger: 기존 SFX 1+음악 실패 1+TTS 6=**8회**, credits_held=**1,250**. 실제 청구액/잔액과 다르며 실패 예약은 자동 반환하지 않는다.
- Git 제외 user-data/starter-pack에 MP3 6개·hash/metadata·receipt·summary·listen.html을 보관한다. managed 후보도 보존한다. Key/계정 identity/원격 응답 전문/음원은 Git·CI에 없다.
- 6개 review pending/check matches=true/approval=not_granted. 청감/타입 적합성·게임 적용은 미확인/미승인이다.

이후 개발은 과금 없는 mock/오프라인 검증을 우선한다. 음악의 기존 실패 요청을 재시도하지 않는다. 추가 제작 시 HTTP status 진단이 가능한 별도 후보 요청 여부를 판단한다. Windows 실제 생성·AI 제품 MCP 연결·T05/게임 적용은 별도다.
