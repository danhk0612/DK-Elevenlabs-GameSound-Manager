# 현재 상태

기준일: 2026-11-20. **v0.5.0 T04 로컬 오디오 비동기 제출·조회·취소 보강. AI 자동화 우선·최소 UI, 실기/점검 대기 해제 유지.**

## 지시와 저장소

Windows 화면/청감 결과를 요청하며 개발을 멈추지 않는다. 미확인을 통과로 바꾸지 않는다. D01~D06 위임 채택 유지, D08 기본 출력/검사 기준·D09 기본 한도 수치는 미결정이다.

main `4c3537c55084b71feb5c42eb2340005bf922f197`, T03 기반 `4e0125b302dd203dd59ab52760deb9fa5277cf61`에서 기존 `task/t00-a-technical-spike`와 [draft PR #1](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/pull/1)을 재사용한다. 새 구현 SHA는 PR head/Git 기록과 artifact source-sha.txt를 따른다.

## 현재 구현

[실행/MCP](app/README.md), [로컬 관리](docs/T01_LOCAL_LIBRARY.md), [생성](docs/T02_GENERATION.md), [편집·분석](docs/T03_AUDIO.md), [비동기 큐/취소](docs/T04_ASYNC_AUDIO.md).

- 공통 명령 **41개 / CLI JSON / MCP 40개 도구**. 프로젝트 자료는 명시적 project_id로 격리한다. SID별 단일 엔진·owner-only local pipe·자동 기동·busy 종료 거절·5분 idle은 유지한다.
- 원본/후보·부모·SHA·출처·copying journal/no-replace 게시/복구, 필요한 사운드·캐릭터/대사를 관리한다. 생성/편집은 candidate만 만들고 원본/기존 버전/게임 파일을 보존한다.
- T02 Credential Manager·WinHTTP 계정/Voice/모델/TTS/SFX adapter와 합성 mock, workspace 공유 예산·미설정 차단·단일 POST 큐·미전송 취소/불확실 상태·수신 파일 등록 복구는 유지한다. 실제 청구액 상한은 보장하지 않는다.
- T03 명시적 FFmpeg 경로/hash/version과 출력 조건으로 trim/gain/fade·WAV/OGG/MP3·rate/channel 변환, decoded 첫 스트림의 peak/RMS/quiet·파형·선택 loop seam 수치를 제공한다. 조건/입력 hash·도구 버전을 보관하고 주관 적합/승인을 분리한다.
- T04 **audio.submit(action=edit/analyze)**은 외부 도구 실행/대용량 hash 없이 metadata/버전을 예약하고 queued로 반환한다. 대기는 전체 최대 32개다. 기존 audio.edit/analyze 동기 명령은 유지한다. 도구/입력 snapshot을 고정하고 실행 전 실제 hash를 대조한다.
- audio.get/list/cancel과 submit은 별도 짧은 catalog 연결에서 처리하여 긴 실행 중 조회·추가 제출·취소가 가능하다. 이 연결은 migration/재시작 정리를 하지 않는다. 실제 media 실행/최종 등록은 단일 worker이며 local/remote 대기가 모두 있으면 교대로 처리한다. 일반 자료 명령은 계속 직렬 대기할 수 있다.
- 단계와 cancel_requested를 영속 기록한다. 실행 중 외부 자식 kill/wait·통계 루프/단계 취소 대조, 취소와 output_ready/분석 완료 기록의 원자적 경계를 구현했다. output_ready 이후 취소는 거절하고 받은 결과 등록/복구를 보존한다. 재기동 자동 실행은 없다.
- catalog schema 3→4 migration, 프로젝트 DB schema 1 유지. 도구 snapshot/phase/cancel flag를 추가했다. 큐/실행 중 shutdown/idle 거절과 원격 생성·편집의 원자적 대상 예약을 보강했다.
- 기존 중립색 GUI에 작업 ID 상태/단계 조회와 취소 요청 버튼만 추가했다. 요청 접수와 최종 cancelled를 구분한다. 생성/편집/분석 제출은 CLI/MCP에 집중하며 기존 목록/재생/파형·비교를 유지한다.

## 확인한 결과

- T03 `4e0125b302dd203dd59ab52760deb9fa5277cf61`의 [Windows run 44741591337](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/actions/runs/44741591337)은 기본 bundled 회귀·engine/CLI/MCP/GUI 링크·패키징 성공이다. 새 v0.5 실행 증거는 아니다.
- Linux musl Rust 1.100.0·Zig CC/AR·기본 bundled SQLite와 FFmpeg/ffprobe 7.0.2 static으로 `cargo test --locked --manifest-path app/Cargo.toml --lib -- --include-ignored` **35개 통과**. DK_TEST_FFMPEG/DK_TEST_FFPROBE에 /tmp/dk-audio-tools 경로를 지정했다. 실제 외부 도구 통합은 6개이며 합성 신호만 사용했다. 도구 출처/hash는 T03 계약에 보존했다.
- 큐 ID 재사용/입력 충돌·프로젝트 경계·32개 제한·재기동 무재생, 두 연결의 동일 대상 예약 및 생성/편집 교차 예약, 실제 실행 중 별도 조회/새 제출/취소·snapshot 고정·후보 보존, 취소/등록 경계·자식 프로세스 kill/wait·shutdown/idle 보호를 확인했다. T01~T03의 편집/분석/보존·중단 회귀도 유지했다.
- 기본 cargo test --lib는 Linux 29개 통과/외부 도구 6개 ignore다. Windows 기본 CI에서 이 6개까지 실제 실행한 것으로 확대하지 않는다. Linux 자식 프로세스 취소와 Windows 실제 IPC/핸들 수명은 구분한다.
- native engine/CLI/MCP build·schema와 stdio initialize/list/call/EOF, action별 schema/공통 typed 오류·40개 도구·전송 크기를 확인했다. Windows x64 cargo check는 외부 SQLite link 선언 + no-default-features 타입 검사로 통과했다. Node 24 TypeScript/Vite 최소 GUI build 통과. fmt·JSON/문서 링크·whitespace도 통과했다.

Rust 도구는 /tmp/dk-cargo, /tmp/dk-rustup, /tmp/dk-rust-tools를 사용한다. Zig CC/AR와 CFLAGS=-fno-sanitize=all·rust-lld wrapper로 bundled SQLite를 빌드했다. 도구/fixture/실행 파일/음원은 Git에 넣지 않는다. Windows CI artifact는 README·LOCAL_LIBRARY.md·GENERATION.md·AUDIO.md·ASYNC_AUDIO.md·source SHA/manifest를 포함하며 해당 head 결과를 별도로 확인한다.

## 남은 범위와 다음

T04 전체를 완료로 닫지 않는다. 다음은 검토 대기 제출·후보 이력 조회·CLI/MCP 클라이언트 호환 보강이다. 실제 AI 클라이언트 연결 검증은 native stdio smoke와 다르다. T05 사람 승인/게임 적용(D07)은 별도 설계 범위이며 착수하지 않는다.

Windows 실제 IPC/GUI·실행 중 FFmpeg 취소/파일 핸들·사용자 청감·실제 Key/요금제/과금은 미확인이다. T02 원격 preview는 URL 반환만 제공하며 한도 증액/초기화·다른 PC 공유·불확실 비용 정산은 후속이다. D08 게임별 기본값·loop tag·LUFS/true peak·전문 편집 화면도 없다. 이 미확인을 개발 대기 조건으로 쓰지 않는다.

T00-A의 [과거 Windows 33개/별도 ACL·ZIP 증거](docs/T00_A_RESULTS.md)와 미수령 사용자 실기를 보존한다. 과거 [증거 CLI](spikes/t00-a/tools/README.md)는 현재 개발 승인 판정기가 아니다.

게임 개발은 일시 정지다. 요구는 사이트 games/02-deck-expedition/POST_AUDIO_BACKLOG.md에 보관했다. 병합·Release/tag·공개 배포·실제 Key/크레딧·사용자 음원/게임 소스/DB/사이트 빌드·배포 변경은 수행하지 않았다.
