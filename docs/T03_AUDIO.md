# T03 — 편집·분석 자동화 v0.4.0

v0.5.0의 catalog schema 4·41개 명령/MCP 40개 도구, 비동기 제출/조회·취소는 [T04 보강 계약](T04_ASYNC_AUDIO.md)을 함께 따른다. 이 문서의 동기 명령은 유지한다.

39개 공통 명령 / MCP 38개 도구다. 모든 편집·검사는 CLI JSON/MCP에서 실행하며 GUI는 저장된 파형/수치 조회와 비교용 재생만 추가했다. 게임 파일 적용·사람 승인·음질 자동 합격 판정은 없다.

## 출력·판정의 선택

D08의 게임별 출력·음량/무음 임계값 기본값은 확정하지 않았다. 요청자가 출력 프로필/표본율/채널, 편집 구간/음량/페이드와 검사 조건을 명시한다. 생략하면 자동 추천값으로 실행하지 않고 거절한다. 아래 수치는 예시이며 제품의 기본 기준이 아니다.

외부 도구는 사용자가 준비한 FFmpeg/ffprobe 실행 파일의 절대 경로를 명시한다. PATH 자동 검색·다운로드·자동 설치·바이너리 동봉은 없다. 도구 파일의 SHA256과 version 첫 줄을 설정에 보관하고 각 작업 전 hash를 다시 확인한다. 업데이트/이동했다면 명시적으로 재설정한다. 임의 실행 파일을 안전하게 만드는 검증/샌드박스가 아니므로 신뢰하는 도구 경로만 설정한다.

```json
{"protocol_version":1,"op":"audio.tools.set","ffmpeg_path":"C:\\Tools\\ffmpeg\\bin\\ffmpeg.exe","ffprobe_path":"C:\\Tools\\ffmpeg\\bin\\ffprobe.exe"}
```

각 JSON은 UTF-8 파일로 저장하여 `dk-sound-cli request < request.json` 또는 PowerShell `Get-Content -Raw -Encoding utf8 request.json | .\dk-sound-cli.exe request`로 실행한다. MCP는 sound_audio_tools_set 등의 공통 schema를 사용한다. 기본 로컬 등록·mock 생성에는 이 도구 설정이 필요하지 않다.

## 명령

| 명령 | 필수 입력 | 동작 |
| --- | --- | --- |
| audio.tools.set | ffmpeg_path, ffprobe_path | 실행/version 확인 후 로컬 설정 저장 |
| audio.tools.get | 없음 | 설정과 현재 도구 hash 대조 |
| audio.analyze | project_id, asset_id, version_id, operation_id, waveform_bins, silence_db, silence_min_ms, clip_level | 지정 버전을 분석하고 report 저장 |
| audio.edit | project_id, asset_id, parent_version_id, version_id, operation_id, start_ms, end_ms, gain_db, fade_in_ms, fade_out_ms, output_format, sample_rate, channels | 부모를 읽어 같은 asset에 새 후보 생성 |
| audio.get | project_id, operation_id | 저장된 요청·입력 hash·상태·결과/오류 조회 |
| audio.list | project_id | after/limit으로 이력 요약, 최대 10개 |
| audio.recover | project_id, operation_id | output_ready 파일의 후보 등록만 복구 |

operation_id는 전역 고유하다. 같은 프로젝트/정확히 같은 입력의 반복 요청은 기존 상태를 돌려주고 다시 FFmpeg를 실행하지 않는다. 다른 입력에는 audio_operation_id_conflict다. 완료된 보고서는 해당 입력 hash의 과거 측정이며 현재 파일의 재검사가 아니다. 새 측정은 새로운 ID로 명시한다.

## 편집

```json
{"protocol_version":1,"op":"audio.edit","project_id":"demo","asset_id":"click","parent_version_id":"take-001","version_id":"edited-001","operation_id":"edit-click-001","start_ms":0,"end_ms":450,"gain_db":-3,"fade_in_ms":5,"fade_out_ms":25,"output_format":"wav_pcm_s16le","sample_rate":48000,"channels":1}
```

- 시간 입력은 정수 ms, 0..86,400,000 범위다. end > start, 실제 디코딩 길이 안이어야 한다. 페이드 합은 선택 길이를 넘지 못한다. 부모/대상 version_id가 같거나 기존/예약된 대상이면 거절한다.
- gain_db는 -60..24. volume 뒤 선형(tri) fade-in/out을 적용한다. 0 ms 페이드는 적용하지 않는다. 자동 정규화/리미터·LUFS 목표는 없다. 큰 gain은 clipping을 만들 수 있으며 분석으로 별도 확인한다.
- output_format은 명시적 프로필 wav_pcm_s16le, ogg_vorbis_q4, mp3_128k 중 하나다. 각 이름이 PCM 16-bit LE / libvorbis q4 / libmp3lame 128 kbps를 고정한다. 기본 프로필을 자동 선택하지 않는다. 도구에 encoder가 없으면 실패하고 다른 형식으로 대체하지 않는다.
- sample_rate는 8000/16000/22050/24000/32000/44100/48000/96000, channels는 1/2. 요청한 출력의 rate/channel을 ffprobe로 대조한다. 분석은 원래 스트림의 rate/channel을 유지한다.
- 첫 오디오 스트림만 사용하며 비디오/자막/데이터·원래 metadata는 출력에 복사하지 않는다. 선택 스트림을 시작 시간 0으로 정렬하고 trim → 시간 재정렬 → gain → fade → 출력 변환한다. lossy 형식은 encoder delay/padding과 codec 특성이 있어 샘플 단위 동일성/일반 무손실 루프를 보장하지 않는다.
- 선택 loop_start_ms/loop_end_ms는 둘 다 필요하며 편집 **출력** 좌표의 구간이다. 요청 길이 안의 metadata로 보관한다. 오디오를 반복하거나 파일에 loop tag를 삽입하지 않는다. 실제 출력의 loop/seam은 별도 분석 요청에서 확인한다.

원본/부모는 읽고 전후 hash를 대조한다. LOCALAPPDATA/audio-operations/{operation_id}/output.ext에 create_new·sync로 결과를 만들고 hash/output_ready를 기록한 다음 T01 import journal로 candidate를 등록한다. 기존 원본·후보/승인본·게임 경로를 덮어쓰지 않는다. 편집 조건·부모·입력 hash·도구 버전/hash·결과 후보를 이력으로 묶는다.

## 분석·파형

```json
{"protocol_version":1,"op":"audio.analyze","project_id":"demo","asset_id":"click","version_id":"edited-001","operation_id":"inspect-click-001","waveform_bins":128,"silence_db":-60,"silence_min_ms":20,"clip_level":0.999,"loop_start_ms":0,"loop_end_ms":400}
```

- waveform_bins 16..256. 전체 디코딩 프레임을 균등 구간으로 나누고 모든 채널의 min/max를 함께 기록한다. 짧은 파일은 실제 프레임 수까지 줄인다. 한 채널이 다른 채널에 상쇄되지 않으며 원래 채널을 mono로 섞어서 검사하지 않는다.
- silence_db -120..0, silence_min_ms 1..86,400,000. 각 프레임의 모든 채널 절댓값이 10^(silence_db/20) 이하이면 quiet다. 전체 quiet 시간/가장 긴 연속 quiet와 최소 길이 이상인 무음 구간 수를 기록한다. 구간 목록은 32개까지이며 더 많으면 truncated=true다. 전체 시간은 최소 구간 길이와 관계없이 합산한다.
- clip_level 0.01..1. 각 디코딩 샘플의 절댓값이 기준 **이상**인 개수를 threshold_exceeding_samples로 기록한다. 이 숫자가 원본 녹음의 clipping을 확정하거나 lossy 복원의 overshoot와 구분하지는 않는다.
- sample_peak, RMS, 각각의 dBFS, decoded frames/duration, codec/container/sample_rate/channels를 저장한다. RMS는 모든 채널 샘플 제곱 평균의 제곱근이다. 완전 무음의 dBFS는 JSON null(−∞)이다. true peak·integrated LUFS·청감/장면 적합은 측정하지 않는다.
- 분석의 선택 loop 구간은 **분석 대상 파일** 좌표다. ms를 가장 가까운 프레임으로 변환한 [start,end) 안에서 첫/마지막 샘플의 채널별 차이 중 최대를 loop_seam_sample_delta로 반환한다. 이는 단일 경계 수치이며 매끄러운 루프의 자동 인증이 아니다.

보고서의 conditions/input_sha256/tools는 동일 측정에 묶여 영속 저장된다. subjective_quality=not_evaluated, approval=not_granted이며 사용자 승인으로 바꾸는 입력은 없다. GUI는 선택한 project/asset/version과 일치하는 완료 analysis ID를 조회해 파형과 수치를 보여 준다. 표시 범위는 ±1이며 실제 초과 peak는 수치로 확인한다. 비교용으로 보관한 재생기와 현재 재생기를 따로 조작한다. 전문 타임라인/구간 선택·동기화 A/B는 없다.

## 실행·중단·오류

이 단계의 audio 명령은 엔진 writer에서 순서대로 실행하는 **동기 명령**이다. 별도 async queue로 제출하는 T02 job.submit과 구별한다. 실행 동안 status와 원격 job.get/list는 응답하지만 일반 자료/audio.get/list는 writer 뒤에서 기다릴 수 있다. 클라이언트를 닫아도 엔진의 작업은 이어지고 정상 busy shutdown/idle은 거절된다. 클라이언트 응답 timeout 300초 이후 자동 재제출하지 않는다. audio.list/get으로 상태를 대조한다.

외부 프로세스는 shell 없이 인자 배열로 실행하며 stdin null/stderr 비공개, version 최대 5초·64 KiB, probe 15초·128 KiB, decode/encode 각각 90초·출력 512 MiB 제한이다. 첫 스트림만 처리하고 file/pipe protocol 및 지원 demuxer(wav/mp3/ogg/flac/mov/aac/aiff/asf)만 허용한다. decoded f32 파일은 별도 새 임시 파일로 받으며 성공 검사 후 제거한다. 분석 rate는 8..192 kHz/최대 8채널이다. decoded 512 MiB 한도는 압축 파일 허용 크기와 다르며 rate/channel에 따라 처리 가능한 길이가 달라진다. 원본 보관 한도는 기존 512 MiB다.

상태는 processing → succeeded / failed, 등록 직전은 output_ready다. 엔진 재기동 시 processing은 interrupted이며 자동 반복하지 않는다. output_ready는 실제 출력 hash·부모 hash를 대조하여 audio.recover로 후보 등록만 복구한다. 도구 설정 없이도 이 복구는 가능하다. partial/failed/interrupted 출력은 자동 채택하지 않는다. 실패 ID/대상 버전도 보존하며 다른 요청에 재사용하지 않는다. 새 작업은 새 ID/버전으로 명시한다. 파생 spool/실패 임시 파일의 자동 정리나 모든 전원 장애 복구는 제공하지 않는다.

주요 오류: audio_tools_not_configured, audio_tool_changed, audio_tool_start_failed, audio_tool_failed, audio_tool_timeout, audio_output_limit, audio_source_unavailable/changed, audio_target_reserved, audio_operation_id_conflict, audio_trim_out_of_range, audio_loop_out_of_range, audio_output_integrity_mismatch. 원본 hash 불일치는 덮어쓰기/자동 수정으로 처리하지 않는다.

catalog schema 2→3을 transaction으로 migration한다. 프로젝트 DB schema 1은 유지한다. 이전 v0.3.0은 schema 3을 읽지 못하므로 업데이트 전 정상 엔진 종료·일관된 catalog/작업공간 백업이 필요하다. 자동 다운그레이드로 새 자료를 삭제하지 않는다. 모든 바이너리는 같은 버전 세트를 사용한다.

## 검증과 남은 범위

실제 FFmpeg/ffprobe 7.0.2 static을 /tmp의 개발 전용 경로에서 사용했다. 합성 신호만 처리했으며 실행 파일/음원은 Git/artifact에 동봉하지 않는다. 도구 출처는 https://johnvansickle.com/ffmpeg/releases/ffmpeg-release-amd64-static.tar.xz, SHA256은 ffmpeg e7e7fb30477f717e6f55f9180a70386c62677ef8a4d4d1a5d948f4098aa3eb99, ffprobe 4f231a1960d83e403d08f7971e271707bec278a9ae18e21b8b5b03186668450d다. 제품의 배포/권장 바이너리를 확정한 것은 아니다.

기본 `cargo test --locked --lib`는 외부 도구 없이 실행한다. 실제 처리 통합 4개는 도구 경로가 필요하여 ignore로 분리했다. 이번 작업에서는 다음 명령으로 포함 실행했다.

```sh
DK_TEST_FFMPEG=/tmp/dk-audio-tools/ffmpeg \
DK_TEST_FFPROBE=/tmp/dk-audio-tools/ffprobe \
cargo test --locked --manifest-path app/Cargo.toml --lib -- --include-ignored
```

알려진 1초 PCM 신호의 peak/RMS/무음/파형·채널/loop 수치·NaN/부분 PCM 거절, 실제 trim/gain/fade/resample/stereo·WAV/OGG/MP3, 원본 hash·새 후보, 중복/충돌·다른 프로젝트 접근·등록 복구/변조·도구 변경·중단·잘못된 오디오, 프로세스 timeout/출력 제한을 검증한다. Windows CI 기본 테스트는 실제 FFmpeg 통합 4개를 실행했다고 주장하지 않는다. 실제 Windows 외부 도구·GUI 재생/비교·사용자 청감은 미확인이며 개발 대기 조건으로 삼지 않는다.

T03 기본 자동화 경로를 구현했으며 게임별 출력/검사 프로필(D08), 파일별 loop tag, LUFS/true-peak, 긴 작업 async 전환·취소, 주관 검토 제출/승인 흐름은 별도 후속이다.

공식 근거: [FFmpeg atrim/asetpts/volume/afade](https://ffmpeg.org/ffmpeg-filters.html), [ffprobe JSON과 stream 선택](https://ffmpeg.org/ffprobe.html). 문서 지원과 이번 실측을 구분한다.
