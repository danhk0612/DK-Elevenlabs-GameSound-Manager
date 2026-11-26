"""Exercise the packaged GUI and CLI together on an isolated Windows CI runner."""
import hashlib
import json
import os
from pathlib import Path
import socket
import subprocess
import tempfile
import time
import urllib.error
import urllib.request
import wave

from playwright.sync_api import expect, sync_playwright


def main():
    if os.name != "nt" or os.environ.get("GITHUB_ACTIONS") != "true":
        raise RuntimeError("Use the isolated Windows runner")
    bins = Path("target/release").resolve()
    with tempfile.TemporaryDirectory(prefix="dk-gui-", ignore_cleanup_errors=True) as temporary:
        root = Path(temporary)
        source = root / "게임 효과음.wav"
        with wave.open(str(source), "wb") as sound:
            sound.setparams((1, 2, 8000, 0, "NONE", "not compressed"))
            sound.writeframes(bytes(16000))
        digest = hashlib.sha256(source.read_bytes()).hexdigest()
        with socket.socket() as sock:
            sock.bind(("127.0.0.1", 0))
            port = sock.getsockname()[1]
        endpoint = f"http://127.0.0.1:{port}"
        settings = root / "settings"
        settings.mkdir()
        env = dict(os.environ, LOCALAPPDATA=str(settings), DK_SOUND_ROOT=str(settings),
                   WEBVIEW2_USER_DATA_FOLDER=str(root / "webview"),
                   WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=f"--remote-debugging-port={port}")
        gui = subprocess.Popen([bins / "dk-sound-gui.exe"], env=env)
        cli = bins / "dk-sound-cli.exe"
        try:
            with sync_playwright() as playwright:
                deadline = time.monotonic() + 45
                while time.monotonic() < deadline and gui.poll() is None:
                    try:
                        with urllib.request.urlopen(endpoint + "/json/list", timeout=2) as response:
                            if any(t.get("type") == "page" for t in json.load(response)):
                                break
                    except (urllib.error.URLError, TimeoutError):
                        pass
                    time.sleep(0.3)
                browser = playwright.chromium.connect_over_cdp(endpoint, timeout=10000)
                page = browser.contexts[0].pages[0]
                page.get_by_text("새 프로젝트", exact=True).click()
                project_form = page.locator("form").filter(has=page.get_by_role("button", name="프로젝트 만들기"))
                project_form.get_by_label("프로젝트 이름").fill("게임 테스트")
                project_form.get_by_label("프로젝트 ID").fill("gui-ci")
                project_form.get_by_label("작업공간 경로").fill(str(root))
                project_form.get_by_role("button").click()
                page.get_by_text("원본 가져오기", exact=True).click()
                asset_form = page.locator("form").filter(has=page.get_by_role("button", name="원본 등록"))
                for label, value in [("파일 경로", str(source)), ("표시 이름", "효과음"), ("자산 ID", "hit"), ("출처", "CI fixture")]:
                    asset_form.get_by_label(label).fill(value)
                asset_form.get_by_role("button").click()
                page.get_by_role("button", name="효과음").click()
                page.locator('section[aria-label="사운드"] select').filter(has=page.locator('option[value="original"]')).select_option("original")
                page.get_by_role("button", name="선택 버전 불러오기").click()
                audio = page.locator("audio")
                expect(audio).to_be_visible()
                expect(audio).to_have_js_property("duration", 1)
                audio.focus()
                audio.press("Space")
                expect(audio).to_have_js_property("ended", True, timeout=10000)
                command = {"protocol_version": 1, "op": "asset.file", "project_id": "gui-ci",
                           "asset_id": "hit", "version_id": "original"}
                result = subprocess.run([cli, "request"], input=json.dumps(command).encode(), env=env,
                                        capture_output=True, check=True, timeout=20)
                record = json.loads(result.stdout)["data"]
                assert record["sha256"] == digest == hashlib.sha256(source.read_bytes()).hexdigest()
                for screen in ["검토 기록", "연결 설정", "라이브러리"]:
                    page.locator("nav button").filter(has_text=screen).click()
                    expect(page.get_by_role("heading", level=1)).to_have_text(screen)
                print("Passed real WebView2 project creation, import, shared CLI hash, WAV decode and navigation")
        finally:
            if gui.poll() is None:
                gui.terminate()
            gui.wait(timeout=10)
            subprocess.run([cli, "shutdown"], env=env, timeout=20, capture_output=True)


if __name__ == "__main__":
    main()
