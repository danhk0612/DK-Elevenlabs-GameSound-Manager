# T00-A Windows 결과 수집 경로

제품 설치가 아닌 격리 실험이다. 실제 게임·음원·API Key를 사용하지 않는다. 이번 안내는 **내부 검사까지 마친 아래 ZIP**에만 해당하며 이후 문서 HEAD/다른 날짜의 ZIP과 SHA를 섞지 않는다.

## 검증된 실기 ZIP

- [다운로드: artifact 14618906066](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/actions/runs/44713166683/artifacts/14618906066) / [성공 run 44713166683](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/actions/runs/44713166683)
- 이름: `t00-a-windows-828e74540ae0aa0013d6ec3425389cb0631b0b1a`
- run head / source SHA: `828e74540ae0aa0013d6ec3425389cb0631b0b1a`
- ZIP SHA256: `368d9d873265edade0fa7203d947353e032795144d26e0bac2c313cc98b8a631`
- 크기: 4,271,017바이트. 만료 예정: **2026-12-04 18:49:13 KST**.
- [내부 검증 영수증](t00-a-evidence/refresh-20261120/artifact-inspection.json): CRC·manifest 24개·보고서 source SHA·실행 파일 4개 x64 일치. Windows CI 33개와 별도 pipe ACL 성공. 사람 실기/청감은 미확인.

다운로드한 ZIP은 압축을 풀기 전에 `Get-FileHash -LiteralPath '<다운로드 ZIP 경로>' -Algorithm SHA256`으로 위 digest와 대조한다. 만료/불일치면 실행하지 않고 새 run의 SHA/manifest를 다시 검증한다. 과거 ZIP의 hash를 새 ZIP에 재사용하지 않는다.

## 준비와 실행

1. ZIP을 새 한글/공백 경로(예: `C:\T00 실기\20261120`)에 푼다. 기존 결과 위에 풀지 않는다. 일반 사용자 권한으로 확인한다. Python 3, FFmpeg/ffprobe의 PATH, WebView2가 필요하며 설치를 자동으로 바꾸지 않는다. 이 ZIP은 FFmpeg를 동봉하지 않는다.
2. 압축을 푼 폴더에서 Windows PowerShell로 아래 명령을 실행한다. 수집 스크립트가 ExpectedSha와 파일 manifest를 재확인한다.

```powershell
powershell -NoProfile -ExecutionPolicy Bypass -File .\Collect-UserResult.ps1 -ExpectedSha 828e74540ae0aa0013d6ec3425389cb0631b0b1a
```

3. 자동 검사가 끝나고 열린 첫 GUI에서 다음을 직접 확인한다.
   - `공통 상태 조회 / 재연결`: 오류 없이 상태가 보이는지 확인하고 엔진 `pid`를 기록한다.
   - `10초 모의 작업 제출`: 제출 응답에 작업이 생기는지 확인한다. 약 10초 안에 창을 닫아 작업 중 닫기를 검사한다.
   - 창이 닫히면 스크립트가 CLI 상태를 출력하고 두 번째 GUI를 연다. GUI 없이도 작업이 유지됐는지, 다시 상태를 조회하면 같은 엔진 PID와 해당 작업 결과가 보이는지 기록한다. 첫 창을 늦게 닫아 이미 완료됐다면 '작업 중 닫기'는 미검증으로 남긴다.
   - `합성 WAV 불러오기` → `합성 WAV 재생`: 재생 이벤트와 별도로 **스피커/헤드폰에서 소리가 실제로 들렸는지** 적는다. 소리가 없으면 오디오 출력 장치·음소거 상태와 함께 실패/미검증을 기록한다.
4. 두 번째 창을 닫으면 `user-observation.json` 템플릿을 만든다. `source_sha`는 수정하지 않고 실행자 별칭과 각 화면/청감 결과를 직접 채운다. 빈 값·'미검증'은 통과가 아니다. 오류가 나서 템플릿이 없으면 실패 메시지와 `user-automatic.json`만 먼저 보존한다.

## 제출할 결과와 대조

- `user-automatic.json`: 실제 실행한 자동 검사 결과 원문.
- `user-observation.json`: 사용자가 직접 작성한 화면/청감 관찰. `source_sha`, Windows 버전/에디션/빌드·x64 여부·일반/관리자 권한, 실행자 별칭, GUI 상태/제출/닫기/재연결/실제 청감 결과를 포함한다. OS 정보는 `Get-CimInstance Win32_OperatingSystem | Select-Object Caption, Version, BuildNumber, OSArchitecture`로 확인해 notes에 적을 수 있다.
- 두 JSON의 SHA256: 작성 완료 후 `Get-FileHash -LiteralPath .\user-automatic.json,.\user-observation.json -Algorithm SHA256`으로 구한다.
- 필요 시 GUI 캡처와 실패 메시지. 비밀정보·개인 전체 경로·실제 음원은 첨부하지 않는다.

PR #1 Conversation에 결과를 전달한다. 검토자는 run head → ZIP digest → source-sha/manifest → 사용자 JSON의 source_sha와 실제 파일 hash를 연결한다. 다른 SHA 결과나 비어 있는 관찰은 현재 코드의 성공으로 처리하지 않는다. 결과를 받기 전에는 user JSON의 hash도 **미수령**으로 남긴다.

CI는 Windows Server 2022 x64에서 실제 GUI 버튼·닫기·재연결·WAV ended를 자동 검사했다. Windows 11 사용자 성공은 Windows 10 지원 증거가 아니며, MP3/OGG·실제 음원 청감과도 구분한다. 별도 native pipe ACL과 계정 삭제는 CI에서 통과했지만 사용자 수집에 계정 생성/pipe 검사는 포함하지 않는다. 제품 엔진 IPC 통합·원격 차단은 미검증이다.

D01~D06은 후속 사용자가 기술 선택을 명시적으로 위임해 [현재 결정](T00_A_DECISIONS.md)에 반영했다. 이번 실기의 우선 대상은 Windows 11 x64다. 판단 위임이나 이 절차의 실행 자체를 화면/청감 통과로 해석하지 않는다. T00-A 완료 조건이 충족되기 전 T00-B·병합·Release/tag·제품 배포를 하지 않는다.
