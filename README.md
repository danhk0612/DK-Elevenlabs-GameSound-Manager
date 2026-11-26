# DK GameSound Manager

게임 개발 중 필요한 BGM·효과음·캐릭터 음성을 프로젝트별로 관리하는 Windows 프로그램입니다. AI가 필요한 소리를 목록으로 만들고 원본에서 후보를 제작하면, 사람은 같은 라이브러리에서 버전을 듣고 비교할 수 있습니다.

**[Windows x64 포터블 ZIP 다운로드](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/releases/download/v0.10.0/DK-GameSound-Manager-v0.10.0-windows-x64-MIT.zip)** · [릴리스 및 SHA-256](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/releases/tag/v0.10.0)

## 사람용 · 첫 음원 등록하기

1. ZIP을 한 폴더에 모두 풀고 **dk-sound-gui.exe**를 실행합니다. Windows 11 x64가 대상이며 GUI에는 Microsoft Edge WebView2 Runtime이 필요합니다.
2. **라이브러리 → 새 프로젝트**에서 이름, 프로젝트 ID(예: `game-a`), 미리 만든 로컬 작업공간 폴더를 지정합니다. 음원 작업공간은 프로그램을 압축 해제한 폴더 밖에 둡니다.
3. **원본 가져오기**에서 음원의 전체 경로, 이름, 자산 ID, 종류(BGM·SFX·Voice), 출처를 입력합니다. 프로그램이 원본을 작업공간에 복사합니다. 로컬 파일 관리와 재생에는 API 키가 필요하지 않습니다.
4. 파일과 버전을 선택하고 **선택 버전 불러오기**를 누른 뒤 재생합니다. 두 버전을 들으려면 첫 음원을 **비교용으로 보관**하고 다음 버전을 엽니다.
5. AI가 제출한 음원은 **검토 기록**에서 찾습니다. 제출 당시 후보와 현재 파일의 일치 여부를 확인하고 재생할 수 있습니다. 이 버전에서 게임 파일로의 최종 적용은 사람이 직접 진행합니다.

**ElevenLabs로 생성할 때**는 GUI의 **연결 설정**에서 계정·키·생성 허용 여부·누적 요청 및 예약 크레딧 한도를 설정합니다. 생성 작업은 CLI 또는 MCP로 제출합니다. 키는 Windows Credential Manager에 저장됩니다. 실제 서비스에서 SFX와 짧은 음성 생성을 확인했습니다. 음악 생성 경로는 구현됐지만 실제 성공은 미확인입니다.

## AI용 · 게임 프로젝트 연결하기

로컬 명령을 실행할 수 있으면 CLI, stdio MCP를 지원하면 MCP를 사용합니다. 두 경로 모두 GUI를 닫은 상태에서 같은 엔진과 게임별 작업공간에 접근합니다.

### CLI

압축을 푼 폴더의 PowerShell에서 실행합니다.

```powershell
.\dk-sound-cli.exe schema
'{"protocol_version":1,"op":"project.list"}' | .\dk-sound-cli.exe request
```

`schema`는 도구와 입력 필드를 출력합니다. `validate`는 stdin으로 받은 요청의 구문과 필드를 오프라인에서 검사합니다. `request`는 같은 JSON을 실행하고 결과 JSON을 stdout에 반환합니다. 프로젝트를 선택한 후에는 자료 요청에 `project_id`를 지정합니다. 패키지의 `tools.json`에서도 도구 목록을 읽을 수 있습니다.

### MCP

로컬 stdio 서버를 등록하는 클라이언트의 설정 예시입니다. `command`를 실제 압축 해제 경로로 바꿉니다.

```json
{
  "mcpServers": {
    "dk-gamesound": {
      "command": "C:\\Tools\\DKGameSound\\dk-sound-mcp.exe",
      "args": []
    }
  }
}
```

도구 목록을 조회한 다음 `sound_project_list`를 호출해 GUI에서 만든 프로젝트가 보이는지 확인합니다. 클라이언트마다 서버 등록 형식이 다릅니다.

### AI에 붙여 넣을 작업 지시문

> 사운드 매니저 경로는 [압축 해제 경로], 대상 게임의 프로젝트 ID는 [ID]다. 스키마와 프로젝트를 조회한 뒤 게임에 필요한 소리를 requirement.put으로 등록해. 재사용할 음원이 있으면 asset.import로 복사하고 requirement.link로 연결해. 새로 제작할 소리는 별도 후보로 보관하고 검사 후 review.submit으로 제출해. 프로젝트·작업·버전 ID를 결과에 남겨. 유료 생성은 내가 허용한 범위와 한도에서만 실행하고, 응답이 유실되면 기존 작업 ID부터 조회해. 게임 파일 적용은 내 검토를 기다려.

## 패키지와 업데이트

| 파일 | 용도 |
| --- | --- |
| dk-sound-gui.exe | 프로젝트 등록, 음원 재생·비교, 검토, 연결 설정 |
| dk-sound-cli.exe | 셸·스크립트에서 JSON 요청 실행 |
| dk-sound-mcp.exe | AI 클라이언트용 stdio MCP 서버 |
| dk-sound-engine.exe | 세 진입점이 공유하는 엔진. 요청 시 자동 기동 |
| automation/ | Python 3.11+로 실행하는 선택형 단일·일괄 작업 예제 |
| LOCAL_LIBRARY.md · GENERATION.md · AUDIO.md · REVIEW.md | ZIP에 포함된 입력 예시와 작업별 계약 |

FFmpeg·ffprobe가 필요한 편집과 분석은 별도로 준비한 실행 파일의 경로를 지정합니다. GUI 미리듣기는 WebView2 지원 형식에 대해 버전당 최대 16 MiB입니다. 업데이트할 때는 작업이 끝난 뒤 이전 CLI에서 `shutdown`을 실행하고 새 ZIP을 별도 폴더에 풉니다. 네 실행 파일은 같은 버전을 함께 사용합니다.

Windows Server 2022 x64의 실제 WebView2에서 프로젝트 생성, WAV 가져오기·재생 종료, CLI에서 동일한 원본 해시 조회를 검증했습니다. Windows 11 사용자 PC의 청감과 개별 AI 제품의 연결 호환성은 추가 검증 대상입니다.

[상세 사용·빌드 문서](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/blob/v0.10.0/app/README.md) · [구현·검증 상태](https://github.com/danhk0612/DK-Elevenlabs-GameSound-Manager/blob/main/CURRENT_STATE.md)

## 라이선스

DK GameSound Manager의 자체 코드와 문서에는 MIT 라이선스가 적용됩니다. 전문은 저장소와 배포 ZIP의 `LICENSE`에 있습니다. 외부 구성 요소의 고지는 ZIP의 `THIRD-PARTY-NOTICES.txt`에서, 게임 음원과 ElevenLabs 생성물의 사용 조건은 각 출처에서 확인하세요.
