# DK GameSound Manager · Windows 11 x64

압축을 한 폴더에 모두 풀고 사용한다. 버전을 바꿀 때는 새 폴더에 풀고 이전 엔진을 종료한다. 게임별 작업공간은 프로그램 폴더 밖에 둔다.

## 사람용 라이브러리

1. WebView2 Runtime이 설치된 PC에서 `dk-sound-gui.exe`를 연다.
2. **라이브러리 → 새 프로젝트**에서 이름, 영문 소문자 ID, 미리 만든 로컬 작업공간 경로를 입력한다. 이미 AI가 만든 프로젝트는 목록에서 선택한다.
3. **원본 가져오기**에 파일 전체 경로, 이름, 자산 ID, 종류, 출처를 입력한다. 파일 목록에서 버전을 골라 **선택 버전 불러오기**로 재생한다. 비교할 때는 첫 버전을 **비교용으로 보관**한 뒤 두 번째 버전을 연다.
4. AI가 후보를 제출하면 **검토 기록 → 검토 목록**에서 제출 당시 자료, 현재 파일 일치 여부와 후보 음원을 확인한다. 게임 적용·승인 기능은 개발 중이다.

## 게임을 개발하는 AI

CLI를 실행할 수 있는 AI에는 압축을 푼 절대 경로와 아래 지시문을 전달한다.

> 이 폴더의 dk-sound-cli.exe schema로 현재 도구와 입력 필드를 읽어. project.list로 대상 프로젝트를 확인하고 필요한 사운드는 requirement.put으로 등록해. 로컬 파일은 asset.import로 복사하고 requirement.link로 연결해. 쓰기 요청은 먼저 validate에 같은 JSON을 stdin으로 전달하고 request로 실행해. 작업 ID와 원본·후보 버전 ID를 보관해. 생성·편집 결과는 review.submit으로 제출해. 유료 생성은 지정된 계정·허용 한도 안에서만 실행하고 게임 파일 적용은 사람에게 맡겨.

stdio MCP 클라이언트에는 같은 폴더의 `dk-sound-mcp.exe` 절대 경로를 command로 지정하고 args는 비워 둔다. 첫 호출은 `sound_project_list`다. GUI를 닫아도 사용할 수 있다. CLI에서 validate는 구문·필드만 검사한다. 실제 프로젝트 상태나 처리 결과는 request 응답으로 확인한다.

패키지의 `tools.json`은 동봉된 CLI가 출력한 도구 목록이다. CLI 호출은 `README.md`와 `LOCAL_LIBRARY.md`, 생성·검토는 `GENERATION.md`와 `REVIEW.md`를 참고한다.
