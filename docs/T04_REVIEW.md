# T04 검토 제출·후보 이력 계약 — v0.6.0

이 기능은 검토할 후보와 기술 검사 근거를 모아 둔다. pending은 적용 승인이나 품질 합격이 아니다. 승인·게임 파일 쓰기·export는 제공하지 않는다. 주관 평가와 T05/D07은 별도다. 공통 명령 총 47개, MCP 도구 46개이며 기존 명령을 유지한다.

## 명령

모든 request에는 protocol_version=1, op를 넣는다. MCP는 sound_review_submit 등 도구 이름을 사용하고 arguments에서 두 필드를 뺀다. ID·페이지·문자열 byte 제한은 공통 카탈로그를 따른다.

| op | 필수 필드 | 선택 필드 | 결과/동작 |
| --- | --- | --- | --- |
| review.submit | project_id, review_id, asset_id, version_id, expected_sha256, summary | requirement_id, analysis_id, notes | 새 pending 기록 또는 동일 요청의 기존 기록 |
| review.get | project_id, review_id | 없음 | 불변 snapshot·현재 상태·생성/철회 UTC 시각 |
| review.list | project_id | after, limit, state, asset_id, version_id | ID 순 요약 페이지 |
| review.withdraw | project_id, review_id, reason | 없음 | pending → withdrawn, 파일/원본 보존 |
| review.check | project_id, review_id | 없음 | 현재 파일·연결 자료와 제출 기록의 일치 여부 |
| asset.history | project_id, asset_id, version_id | after, limit | 버전 metadata와 생성/편집/검사/검토 기록 요약 |

expected_sha256은 소문자 hex 64자다. summary는 200 byte, notes/reason은 1024 byte 이하이며 필수 summary/reason은 공백만 허용하지 않는다. 선택 필드는 없으면 연결하지 않는다. limit은 1~10, 기본 10이다. 반환한 next_after를 그대로 넘기고 null이면 끝낸다. 정확히 limit개인 마지막 페이지 다음에 빈 페이지가 올 수 있다. 목록은 일관된 시점의 고정 페이지 snapshot이 아니므로 동시 추가는 다음 탐색에서 대조한다.

## 제출·철회 경계

제출은 ready인 candidate만 허용한다. original 제출, 다른 hash, 파일 누락/변조는 거절한다. 실제 asset.file hash 검증을 수행하므로 대용량 파일은 engine의 일반 writer에서 대기/처리할 수 있다. review_id는 catalog 전체에서 고유하다. 동일 ID/동일 공통 JSON 요청은 기존 기록을 반환하고 다른 요청/프로젝트는 review_id_conflict다. 동일 재요청은 과거 결과를 확인하는 동작으로, 파일을 다시 검사하지 않는다. 철회한 ID를 다시 제출해도 withdrawn 그대로다. 새 후보/새 제출에는 새 review_id를 사용한다.

snapshot에 후보 ID/이름/종류/부모/역할/해시/크기/출처 메모, 선택적 requirement 당시 metadata, 선택적 분석 근거, summary/notes를 고정한다. requirement는 같은 프로젝트·같은 kind여야 한다. 제출은 requirement.link를 변경하지 않으며 기존 연결과 후보가 같다고 주장하지 않는다. 분석은 같은 프로젝트·같은 후보·같은 input_sha256의 성공한 audio.analyze여야 한다. 전체 저장 보고서의 SHA와 조건·도구·metadata·주요 수치만 복사하며 큰 waveform은 복사하지 않는다. 전체 보고서는 audio.get으로 조회한다. 분석 연결을 생략한 제출도 가능하며 검사 합격을 의미하지 않는다.

summary/notes는 클라이언트가 제공한 신뢰하지 않는 텍스트다. submitted_by는 automation_or_local_client이며 사람 인증/승인 출처가 아니다. 모든 기록의 approval은 not_granted다. 승인 필드/명령·임의 출력 경로를 받지 않는다.

철회는 SQLite IMMEDIATE transaction으로 상태·UTC 시각·사유를 한 번 기록한다. 같은 사유 재호출은 동일 기록, 다른 사유 재호출은 review_withdrawal_conflict다. 철회는 삭제나 작업 취소가 아니고 자료를 보존한다.

## 현재 일치 확인과 이력

get은 제출 당시 기록이며 current_integrity=not_checked다. check는 실제 후보 hash를 재검사하고 연결한 requirement 전체 metadata와 분석 보고서 SHA를 대조한다. matches/candidate_matches/requirement_matches/analysis_matches를 반환한다. 연결하지 않은 항목은 null이다. 파일 확인 실패·metadata 변경은 false이며 제출 기록/상태를 자동 변경하지 않는다. withdrawn 기록도 검사 가능하지만 다시 pending/approved가 되지 않는다. matches=true는 측정 시점 자료 일치만 뜻한다. 검사 이후 외부 변경, 주관 음질, 게임 적용 적합성은 보장하지 않는다.

asset.history는 지정 버전의 generation_jobs, audio_operations, reviews를 합친다. category_then_identifier 순서이며 시간순이 아니다. 과거 작업에는 생성 시각이 없어 시간순을 만들어 내지 않는다. cursor는 audio:<id>, generation:<id>, review:<id>다. 결과 항목의 kind/id로 각각 audio.get/job.get/review.get 상세를 조회한다. 버전의 parent_version_id로 부모 이력을 별도 조회할 수 있다. import 자체는 버전 metadata로 표시되고 작업 행을 소급 생성하지 않는다. file_verified=false이므로 현재 파일 검증에는 asset.file/review.check를 사용한다.

review.get/list와 asset.history는 migration/재시작 큐 정리를 하지 않는 별도 읽기 연결에서 처리한다. review.submit/check/withdraw는 일반 writer를 사용한다. catalog schema 4→5 migration은 reviews 테이블/인덱스만 추가하며 프로젝트 DB schema 1과 음원 파일을 변경하지 않는다. 이전 실행 파일은 schema 5를 거절한다. 업데이트 전 엔진을 종료하고 DB/작업공간을 함께 백업한다. downgrade를 위해 user_version을 수동으로 낮추지 않는다.

## CLI 예시

asset.file로 확인한 실제 후보 hash를 expected_sha256에 넣는다. 아래 HASH는 대체 표기이며 그대로 호출하면 검증 오류다.

```json
{"protocol_version":1,"op":"review.submit","project_id":"demo","review_id":"hit-review-1","asset_id":"hit","version_id":"edited-1","expected_sha256":"HASH","summary":"타격음 후보","analysis_id":"hit-analysis-1","notes":"기술 수치는 확인했고 청감 검토는 대기 중"}
{"protocol_version":1,"op":"review.list","project_id":"demo","state":"pending"}
{"protocol_version":1,"op":"review.check","project_id":"demo","review_id":"hit-review-1"}
{"protocol_version":1,"op":"asset.history","project_id":"demo","asset_id":"hit","version_id":"edited-1"}
{"protocol_version":1,"op":"review.withdraw","project_id":"demo","review_id":"hit-review-1","reason":"새 후보로 비교 예정"}
```

GUI는 목록/제출 snapshot 상세·현재 자료 일치 확인·해시가 일치하는 후보 재생 준비와 선택 버전 이력 조회만 제공한다. 생성/편집/분석/검토 제출·철회는 CLI/MCP에서 수행한다. 기존 중립색 목록 UI를 유지한다.

## 확인과 한계

2026-11-20 Linux musl/Rust 1.100.0/Zig CC·AR/bundled SQLite, 개발용 FFmpeg/ffprobe 7.0.2로 합성 입력만 사용했다. 전체 회귀 39개 통과(실제 외부 도구 통합 7개 포함). 새 회귀는 불변 제출/철회/재기동, 승인 필드 거절, 원본/변조/해시 거절, 프로젝트 격리·requirement 변경 감지·페이지·읽기 연결, 실제 async 편집→검사→제출/이력·검사 보고서 변조 감지를 다룬다.

```sh
# 신뢰하는 개발 도구 경로를 명시한다. 바이너리/음원은 Git에 넣지 않는다.
DK_TEST_FFMPEG=/path/to/ffmpeg DK_TEST_FFPROBE=/path/to/ffprobe cargo test --locked --manifest-path app/Cargo.toml --lib -- --include-ignored
```

일반 cargo test --lib는 외부 도구 통합 7개를 ignore한다. native CLI/MCP initialize/list/call/EOF·46개 도구·64 KiB frame 이내·불량 hash/승인 필드 거절을 확인했다. TypeScript/Vite build와 Windows x64 no-default-features 타입 검사도 통과했다. 새 Windows native 링크/GUI CI는 해당 PR head 실행을 확인한다. Linux의 stdio 검증은 실제 지원 AI 클라이언트 연결·Windows IPC/GUI 실행 증거가 아니다. 실제 Key/요금/청감·사용자 음원·게임 적용은 수행하지 않았으며 T04 전체 수용을 완료로 닫지 않는다.
