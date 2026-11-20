# 현재 상태

기준일: 2026-11-20. **v0.8.0 T04 여러 사운드 작업 정의/결과 추적 보강. AI 자동화 우선·최소 UI, 실기/점검 대기 해제 유지.**

## 지시와 저장소

사용자의 Windows 화면/청감 결과를 요청하며 개발을 멈추지 않는다. 미확인을 통과로 기록하지 않는다. D01~D06 위임 채택 유지, D08 게임별 기본 출력/검사 기준·D09 기본 한도 수치는 미결정이다.

main `4c3537c55084b71feb5c42eb2340005bf922f197`, v0.7 상태 문서 head `10f379f48989257078d3fbdf2f42194b45ad1951`에서 기존 `task/t00-a-technical-spike`와 [PR #1](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/pull/1)을 재사용한다. 정확한 새 코드 SHA/빌드 결과는 Git/PR head와 artifact의 source-sha.txt를 따른다. PR은 open이며 사용자 쪽에서 draft가 해제된 상태를 유지한다. 직접 병합하지 않는다.

## 현재 구현

[실행/MCP](app/README.md), [로컬 관리](docs/T01_LOCAL_LIBRARY.md), [생성](docs/T02_GENERATION.md), [편집·분석](docs/T03_AUDIO.md), [비동기 큐/취소](docs/T04_ASYNC_AUDIO.md), [검토/이력](docs/T04_REVIEW.md), [단일 자동화](docs/T04_AUTOMATION.md), [다건 자동화](docs/T04_BATCH.md).

- 공통 **47개 명령/CLI JSON/MCP 46개 도구**, SID별 단일 Windows 엔진·owner-only local pipe·자동 기동·busy 종료 거절·5분 idle, catalog schema 5/프로젝트 DB schema 1을 유지한다.
- 프로젝트별 원본/후보·부모·SHA·출처, copying journal/no-replace 게시/중단 대조를 제공한다. 생성/편집은 candidate만 만들고 기존 버전/게임 파일을 보존한다.
- ElevenLabs Credential Manager/WinHTTP adapter와 합성 mock, workspace 공유 예산·미설정 차단·단일 POST/불확실 상태·받은 파일 등록 복구를 제공한다. 실제 청구액 상한을 보장하지 않는다.
- 명시한 FFmpeg 경로/hash/version·출력/검사 조건으로 trim/gain/fade·형식/rate/channel 변환과 peak/RMS/quiet/파형/loop 수치를 기록한다. 주관 품질/승인과 분리한다.
- audio.submit/get/list/cancel: 도구/입력 snapshot·32개 큐·단일 실행기·진행 단계·협조 취소·output_ready 등록 경계, 별도 연결의 빠른 조회/취소와 재기동 무재생을 유지한다. 일반 자료 명령은 실행기 뒤에서 대기할 수 있다.
- review.submit/get/list/withdraw/check와 asset.history: ready 후보 SHA·선택 requirement/분석 근거의 불변 snapshot, pending/withdrawn·현재 일치 확인·종류/ID 순 이력 페이지다. 검토 제출/일치는 적용 승인이 아니다.
- 최소 GUI는 설정/목록/재생·비교/저장 검사·작업 조회/취소·검토 조회만 제공한다. 이번 UI 변경은 없다.
- v0.7의 CLI offline validate·선택형 Python 3.11+ 단일 작업 정의 클라이언트를 유지한다. 프레임/timeout·전용 reader 정리·부분 쓰기·CLI 단일 응답/Windows 동기 읽기 해제로 상속 pipe 정리 멈춤을 방지한다.
- **v0.8 sound_batch.py**는 최대 32개/1 MiB manifest의 모든 입력·core 명령·프로젝트/source/후보/작업/검토 ID 충돌을 사전 검사한다. 정확히 같은 원본 공유는 허용하고 항목 간 후보 의존은 거절한다. 순차 실행·항목별 성공/실패/중단/미실행/미선택 요약·JSONL·--only 선택 재호출·결과 확정 오류에만 --keep-going을 제공한다. timeout/미지 오류는 항상 중단한다. 새 ID/복구/승인/게임 적용·checkpoint 건너뛰기를 자동 수행하지 않는다.

## 확인한 결과

- v0.7 코드 `3f47e5852942dd28d951354484ecd8d37107e582`의 [Windows run 44751508856](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/actions/runs/44751508856) 전체 성공: Windows Server 2022/Rust 1.100.0/Python 3.12.10에서 Rust 29개/외부 FFmpeg 7개 ignore·Python 9개, engine/CLI/MCP/GUI build·실제 CLI/MCP 독립 자동 기동·같은 검토 기록 재호출/재연결·원본 보존/변경 거절·패키징을 확인했다. 초기 client cleanup timeout은 상속 stdout/reader 정리 수정으로 해소됐다. Git에 과거 실패 증거를 보존한다.
- 해당 [개발 artifact](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/actions/runs/44751508856/artifacts/14635831632), ZIP SHA-256 `72397ae32d9335ead240ad4b184b7f64792e21ba9fb5755f41e813577c728810`. 이는 새 v0.8 실행 증거가 아니다.
- v0.8 Linux에서 Python **18개**(단일 9+batch 9) 통과. 실제 native CLI offline batch preflight/뒤 항목 core 입력 오류·선택 항목/전역 ID 충돌·엔진 미기동을 확인했다. batch의 입력 한도/중복 key/NaN·canonical manifest hash·공유 원본·프로젝트/작업/검토 충돌·선택/상태/진행 식별자·fail-stop/keep-going·불확실 결과 중단·Ctrl+C/클라이언트 정리를 검사한다.
- core 변경은 없고 기존 기본 Rust 회귀는 Linux 32개 통과/외부 FFmpeg 7개 ignore다. 기존 합성 음원·실제 FFmpeg/ffprobe 7.0.2 통합 7개 포함 39개 결과와 도구 출처/hash는 T03/각 계약에 남긴다. Linux 자식 취소·Windows 실제 FFmpeg 동작을 혼동하지 않는다.
- native engine/CLI/MCP build·Windows x64 타입 검사·Node 24 TypeScript/Vite 최소 GUI build·fmt/JSON/문서 링크/diff를 확인한다. Windows 타입 검사는 외부 SQLite link 선언/no-default-features이며 기본 bundled native 링크 증거와 다르다.
- 새 Windows CI에는 단일 recipe에 더해 실제 engine batch의 shared source·선택 실행·CLI/MCP parity/JSONL·철회 항목의 중단/keep-going·원본 보존을 추가했다. 해당 새 head 실행 결과는 별도 확인한다. FFmpeg를 설치하지 않는 CI다.

Rust는 /tmp/dk-cargo, /tmp/dk-rustup, /tmp/dk-rust-tools의 Zig CC/AR·CFLAGS=-fno-sanitize=all·rust-lld wrapper로 bundled SQLite를 빌드한다. Python은 /tmp/msv-groups-tools/python3, 개발 FFmpeg는 /tmp/dk-audio-tools다. 도구/실행 파일/합성 음원은 Git에 넣지 않는다. 개발 artifact에는 명령 문서·BATCH.md·automation 소스/예제·source SHA/재귀 SHA256SUMS를 포함한다.

## 남은 범위

T04 전체 수용을 완료로 닫지 않는다. 자체 stdio 클라이언트·Windows CI 성공은 실제 지원 AI 제품 연결이나 사용자 Windows 11 화면/청감 성공이 아니다. 다음은 새 batch CI 결과 확인·지원 AI 제품 클라이언트 호환과 필요한 사운드/작업 이력 자동화 보강이다. T05 사람 승인/게임 적용(D07)은 별도 설계 범위이며 착수하지 않는다.

Windows 실제 FFmpeg 실행 중 취소/파일 핸들·실제 Key/요금제/과금, 원격 preview 재생, 한도 증액/초기화·다른 PC 공유·불확실 비용 정산은 후속이다. D08 기본값·loop tag·LUFS/true peak·전문 편집 화면도 없다. 미확인을 개발 대기 조건으로 쓰지 않는다.

T00-A 과거 Windows 33개/ACL·ZIP 증거와 미수령 사용자 실기는 docs/T00_A_RESULTS.md에 보존한다. 게임 개발은 일시 정지이며 사이트 games/02-deck-expedition/POST_AUDIO_BACKLOG.md를 유지한다. 병합·Release/tag·공개 배포·실제 Key/크레딧·사용자 음원/게임 소스/DB/사이트 빌드·배포는 수행하지 않았다.
