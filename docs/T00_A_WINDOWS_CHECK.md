# T00-A Windows 결과 수집 경로

이 절차는 제품 설치가 아닌 격리 실험이다. 실제 게임·음원·API Key는 사용하지 않는다.

1. PR의 `T00-A Windows spike` run에서 `t00-a-windows-<40자리 source SHA>` artifact를 받는다. run head SHA와 artifact 내부 `source-sha.txt`가 같은지 확인한다. PR merge SHA가 아닌 브랜치 source SHA를 사용한다.
2. 새 한글/공백 경로에 압축을 풀고 Windows PowerShell에서 `powershell -NoProfile -ExecutionPolicy Bypass -File .\Collect-UserResult.ps1 -ExpectedSha <40자리-SHA>`을 실행한다. Python 3와 FFmpeg/ffprobe가 PATH에 있어야 한다. 스크립트는 ExpectedSha 및 artifact 파일 해시를 검사한다. 없으면 결과에 실패를 남기며 사용자 설치를 자동 변경하지 않는다. WebView2가 필요하다.
3. 자동 검사 후 GUI를 직접 열어 상태 조회 → 작업 제출 → 창 닫기 → CLI 조회 → GUI 재연결을 확인한다. 합성 WAV 로드 후 합성 WAV 재생 버튼으로 소리가 실제로 들리는지 확인한다.
4. 결과 JSON의 미확인 항목을 화면/청감 관찰로 작성하고 PR Conversation에 JSON과 필요 시 화면 캡처를 첨부한다. 실행자·OS 빌드·SHA·도구 버전·명령·각 결과와 실패 메시지를 포함한다. 비밀정보/개인 경로/실제 음원을 첨부하지 않는다.

CI의 자동 검사, GUI 프로세스 기동, WebView 명령 실행, 실제 사용자 화면·청감은 별도 판정한다. 빈 수동 결과는 미검증이다. 검토자는 제출 SHA가 PR 최신 코드와 일치하는지 확인하고 CURRENT_STATE.md와 docs/T00_A_RESULTS.md에 반영한다. 다른 SHA의 성공을 최신 코드에 전용하지 않는다.

현재 확보된 자동 실행 기준은 `f8aa463bfd02fa84a10c39f32b353cf83d2c7872`의 [run/artifact](T00_A_RESULTS.md)다. 이후 HEAD를 사용하면 해당 run과 source-sha를 다시 대조한다. `Collect-UserResult.ps1`은 PowerShell 5.1용 JSON 인자 helper를 사용하고 수동 검사 동안 opt-in 자동 기동/idle 설정을 해제한다. 스크립트가 작성한 자동 결과와 사용자가 채울 observation을 구분한다.

현재 Windows Server CI에서 실제 GUI 버튼·닫기·재연결·재생 종료는 자동 검증됐다. 이 수집은 사용자 Windows 10/11 화면과 실제 들리는 소리를 확인하기 위한 것이다. 별도 native pipe의 다른 신원 ACL 검사는 CI에서 통과했으며 이 사용자 수집에는 계정 생성이나 pipe 검사를 포함하지 않는다. 엔진 IPC 통합·원격 차단을 통과 처리하지 않는다.
