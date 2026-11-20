"""Optional Python 3.11+ local-only CLI/MCP recipe client; no paid calls or approval."""
from __future__ import annotations
import argparse
import json
import math
from pathlib import Path
import queue
import subprocess
import sys
import threading
import time

MAX_FRAME = 65536
PROTOCOL = '2025-11-25'


class Failure(Exception):
    pass


def encode(value):
    data = json.dumps(value, ensure_ascii=False, allow_nan=False, separators=(',', ':')).encode('utf-8')
    if len(data) + 1 > MAX_FRAME:
        raise Failure('request_too_large')
    return data + b'\n'


def decode(data):
    try:
        return json.loads(data.decode('utf-8-sig'), parse_constant=lambda _: (_ for _ in ()).throw(ValueError()))
    except (ValueError, UnicodeError):
        raise Failure('invalid_response_json') from None


class Process:
    """Bounded newline reader with a deadline, including Windows anonymous pipes."""
    def __init__(self, argv):
        self.process = subprocess.Popen(argv, stdin=subprocess.PIPE, stdout=subprocess.PIPE, stderr=subprocess.DEVNULL)
        self.messages = queue.Queue(maxsize=2)
        self.closed = threading.Event()
        self.write_done = threading.Event()
        self.write_done.set()
        self.reader = threading.Thread(target=self._read, daemon=True)
        self.reader.start()

    def _read(self):
        try:
            while not self.closed.is_set():
                data = self.process.stdout.readline(MAX_FRAME + 1)
                if not data:
                    item = Failure('transport_eof')
                elif len(data) > MAX_FRAME or not data.endswith(b'\n'):
                    item = Failure('response_too_large_or_unframed')
                else:
                    item = decode(data)
                while not self.closed.is_set():
                    try:
                        self.messages.put(item, timeout=.1)
                        break
                    except queue.Full:
                        pass
                if isinstance(item, Failure):
                    return
        except (OSError, ValueError, Failure):
            try:
                self.messages.put_nowait(Failure('transport_read_failed'))
            except queue.Full:
                pass

    def send(self, value, timeout=300):
        data = encode(value)
        result = queue.Queue(maxsize=1)
        self.write_done.clear()
        def write():
            try:
                self.process.stdin.write(data)
                self.process.stdin.flush()
                result.put(None)
            except (OSError, ValueError):
                result.put(Failure('transport_write_failed'))
            finally:
                self.write_done.set()
        writer = threading.Thread(target=write, daemon=True)
        writer.start()
        try:
            error = result.get(timeout=timeout)
        except queue.Empty:
            self.process.kill()
            self.process.wait()
            writer.join(timeout=2)
            raise Failure('transport_write_timeout_outcome_unknown') from None
        if error:
            raise error

    def receive(self, timeout):
        try:
            item = self.messages.get(timeout=timeout)
        except queue.Empty:
            raise Failure('transport_timeout_outcome_unknown') from None
        if isinstance(item, Failure):
            raise item
        return item

    def close(self):
        self.closed.set()
        if not self.write_done.is_set():
            self.process.kill()
            self.process.wait()
        try:
            self.process.stdin.close()
        except (OSError, ValueError):
            pass
        try:
            self.process.wait(timeout=2)
        except subprocess.TimeoutExpired:
            self.process.kill()
            self.process.wait()
        self.reader.join(timeout=2)
        self.process.stdout.close()


def envelope(value):
    if not isinstance(value, dict) or value.get('protocol_version') != 1 or type(value.get('ok')) is not bool:
        raise Failure('invalid_common_response')
    if not value['ok']:
        error = value.get('error')
        code = error.get('code', 'request_failed') if isinstance(error, dict) else 'request_failed'
        raise Failure(code if isinstance(code, str) else 'request_failed')
    if not isinstance(value.get('data'), dict):
        raise Failure('missing_response_data')
    return value['data']


class Client:
    def __init__(self, cli, mcp=None, timeout=300):
        self.cli, self.timeout, self.session, self.sequence = cli, timeout, None, 0
        if mcp:
            self.session = Process([str(mcp)])
            try:
                result = self.rpc('initialize', {'protocolVersion': PROTOCOL, 'capabilities': {},
                                               'clientInfo': {'name': 'dk-local-recipe', 'version': '1'}})
                if result.get('protocolVersion') != PROTOCOL:
                    raise Failure('mcp_protocol_mismatch')
                self.session.send({'jsonrpc': '2.0', 'method': 'notifications/initialized'}, self.timeout)
            except BaseException:
                self.close()
                raise

    def rpc(self, method, params):
        self.sequence += 1
        self.session.send({'jsonrpc': '2.0', 'id': self.sequence, 'method': method, 'params': params}, self.timeout)
        reply = self.session.receive(self.timeout)
        if not isinstance(reply, dict) or reply.get('jsonrpc') != '2.0' or type(reply.get('id')) is not int or reply.get('id') != self.sequence:
            raise Failure('mcp_response_id_mismatch')
        if 'error' in reply:
            raise Failure('mcp_protocol_error')
        if not isinstance(reply.get('result'), dict):
            raise Failure('mcp_result_missing')
        return reply['result']

    def cli_call(self, mode, command):
        process = Process([str(self.cli), mode])
        try:
            process.send(command, self.timeout)
            process.process.stdin.close()
            response = process.receive(self.timeout)
            try:
                code = process.process.wait(timeout=2)
            except subprocess.TimeoutExpired:
                raise Failure('cli_exit_timeout_outcome_unknown') from None
            if not isinstance(response, dict):
                raise Failure('invalid_common_response')
            if code != (0 if response.get('ok') is True else 1):
                raise Failure('cli_exit_mismatch')
            return envelope(response)
        finally:
            process.close()

    def validate(self, command):
        return self.cli_call('validate', command)

    def call(self, command):
        if not self.session:
            return self.cli_call('request', command)
        args = {k: v for k, v in command.items() if k not in ('op', 'protocol_version')}
        result = self.rpc('tools/call', {'name': 'sound_' + command['op'].replace('.', '_'), 'arguments': args})
        common = result.get('structuredContent')
        if type(result.get('isError')) is not bool or not isinstance(common, dict) or result['isError'] != (common.get('ok') is not True):
            raise Failure('mcp_envelope_mismatch')
        return envelope(common)

    def discover(self, operations):
        result = self.rpc('tools/list', {})
        tools = result.get('tools')
        if not isinstance(tools, list):
            raise Failure('mcp_tools_missing')
        names = {tool.get('name') for tool in tools if isinstance(tool, dict)}
        if any('sound_' + op.replace('.', '_') not in names for op in operations):
            raise Failure('mcp_required_tool_missing')

    def close(self):
        if self.session:
            self.session.close()
            self.session = None


def fields(value, required, optional=()):
    if not isinstance(value, dict) or set(value) - set(required) - set(optional) or set(required) - set(value):
        raise Failure('invalid_recipe_fields')
    return value


def absolute_file(value):
    if not isinstance(value, str) or not Path(value).is_absolute() or not Path(value).is_file():
        raise Failure('recipe_file_requires_existing_absolute_path')


def compile_recipe(recipe):
    """Return every planned command for offline core validation before any engine request."""
    fields(recipe, ('schema_version', 'project', 'source', 'candidate', 'review'), ('tools', 'analysis'))
    if type(recipe['schema_version']) is not int or recipe['schema_version'] != 1:
        raise Failure('unsupported_recipe_schema')
    project = fields(recipe['project'], ('project_id', 'name', 'workspace'))
    if not isinstance(project['workspace'], str) or not Path(project['workspace']).is_absolute() or not Path(project['workspace']).is_dir():
        raise Failure('workspace_requires_existing_absolute_directory')
    source = fields(recipe['source'], ('asset_id', 'version_id', 'name', 'kind', 'source_path', 'source_note'))
    absolute_file(source['source_path'])
    candidate = recipe['candidate']
    if not isinstance(candidate, dict):
        raise Failure('invalid_recipe_fields')
    edited = 'operation_id' in candidate
    if edited:
        fields(candidate, ('version_id', 'operation_id', 'start_ms', 'end_ms', 'gain_db', 'fade_in_ms', 'fade_out_ms', 'output_format', 'sample_rate', 'channels'))
    else:
        fields(candidate, ('version_id', 'source_path', 'source_note'))
        absolute_file(candidate['source_path'])
    if source['version_id'] == candidate['version_id']:
        raise Failure('candidate_requires_new_version_id')
    review = fields(recipe['review'], ('review_id', 'summary'), ('notes', 'requirement_id'))
    pid = project['project_id']
    aid = source['asset_id']
    def command(op, args):
        return {'protocol_version': 1, 'op': op, **args}
    plan = [('project', command('project.create', project)),
            ('source', command('asset.import', {'project_id': pid, **source}))]
    if edited or 'analysis' in recipe:
        tools = fields(recipe.get('tools'), ('ffmpeg_path', 'ffprobe_path'))
        for path in tools.values():
            absolute_file(path)
        plan.append(('tools', command('audio.tools.set', tools)))
    elif 'tools' in recipe:
        raise Failure('unused_tools_configuration')
    shared = {'project_id': pid, 'asset_id': aid}
    if edited:
        plan.append(('candidate', command('audio.submit', {**shared, 'action': 'edit',
                    'parent_version_id': source['version_id'], **candidate})))
    else:
        plan.append(('candidate', command('asset.import', {**shared, 'name': source['name'],
                    'kind': source['kind'], 'parent_version_id': source['version_id'], **candidate})))
    target = {**shared, 'version_id': candidate['version_id']}
    if 'analysis' in recipe:
        analysis = fields(recipe['analysis'], ('operation_id', 'waveform_bins', 'silence_db', 'silence_min_ms', 'clip_level'), ('loop_start_ms', 'loop_end_ms'))
        if edited and analysis['operation_id'] == candidate['operation_id']:
            raise Failure('analysis_requires_distinct_operation_id')
        plan.append(('analysis', command('audio.submit', {**target, 'action': 'analyze', **analysis})))
    plan.append(('file', command('asset.file', target)))
    plan.append(('review', command('review.submit', {**target, **review, 'expected_sha256': '0' * 64,
                **({'analysis_id': recipe['analysis']['operation_id']} if 'analysis' in recipe else {})})))
    plan.append(('check', command('review.check', {'project_id': pid, 'review_id': review['review_id']})))
    plan.append(('history', command('asset.history', target)))
    # Include polling requests in the same validation phase.
    validations = [c for _, c in plan]
    validations += [command('audio.get', {'project_id': pid, 'operation_id': c['operation_id']})
                    for _, c in plan if c['op'] == 'audio.submit']
    return plan, validations


def execute(client, plan, wait_seconds, poll_seconds, context):
    outputs = {}
    for stage, original in plan:
        context.update(stage=stage)
        command = dict(original)
        if stage == 'review':
            command['expected_sha256'] = outputs['file']['sha256']
        if command['op'] == 'audio.submit':
            context['operation_id'] = command['operation_id']
        print(json.dumps({'stage': stage, 'op': command['op']}, separators=(',', ':')), file=sys.stderr, flush=True)
        data = client.call(command)
        if command['op'] == 'audio.submit':
            deadline = time.monotonic() + wait_seconds
            while data.get('state') in ('queued', 'processing'):
                remaining = deadline - time.monotonic()
                if remaining <= 0:
                    raise Failure('wait_deadline_work_may_continue')
                time.sleep(min(poll_seconds, remaining))
                data = client.call({'protocol_version': 1, 'op': 'audio.get', 'project_id': command['project_id'], 'operation_id': command['operation_id']})
            if data.get('state') != 'succeeded':
                raise Failure('audio_requires_attention_' + str(data.get('state')))
        outputs[stage] = data
        if stage == 'file' and data.get('role') != 'candidate':
            raise Failure('candidate_role_required')
        if stage == 'review' and data.get('state') != 'pending':
            raise Failure('review_not_pending')
        if stage == 'check' and data.get('matches') is not True:
            raise Failure('review_current_data_mismatch')
    return {'project_id': context['project_id'], 'review': outputs['review'], 'check': outputs['check'],
            'history_page': outputs['history'], 'approval': 'not_granted'}


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('recipe', type=Path)
    parser.add_argument('--cli', required=True, type=Path)
    parser.add_argument('--mcp', type=Path, help='Use MCP transport after CLI offline preflight')
    parser.add_argument('--validate-only', action='store_true')
    parser.add_argument('--wait-seconds', type=float, default=300)
    parser.add_argument('--poll-seconds', type=float, default=.5)
    args = parser.parse_args()
    context, client = {'stage': 'preflight'}, None
    try:
        for value, low, high in [(args.wait_seconds, 1, 86400), (args.poll_seconds, .1, 60)]:
            if not math.isfinite(value) or not low <= value <= high:
                raise Failure('invalid_client_timing')
        with args.recipe.open('rb') as stream:
            raw = stream.read(MAX_FRAME + 1)
        if len(raw) > MAX_FRAME:
            raise Failure('recipe_too_large')
        recipe = decode(raw)
        plan, validations = compile_recipe(recipe)
        context.update(project_id=recipe['project']['project_id'], review_id=recipe['review']['review_id'])
        absolute_file(str(args.cli))
        client = Client(args.cli)
        for command in validations:
            client.validate(command)
        if args.validate_only:
            result = {'validated_commands': len(validations), 'engine_started': False}
        else:
            if args.mcp:
                absolute_file(str(args.mcp))
                client = Client(args.cli, args.mcp)
                client.discover({command['op'] for command in validations})
            result = execute(client, plan, args.wait_seconds, args.poll_seconds, context)
        print(json.dumps({'ok': True, 'data': result}, ensure_ascii=True, allow_nan=False))
        return 0
    except KeyboardInterrupt:
        print(json.dumps({'ok': False, 'error': {'code': 'client_interrupted_work_may_continue'}, 'context': context}))
        return 130
    except (Failure, OSError, ValueError, TypeError, KeyError):
        error = sys.exception()
        code = str(error) if isinstance(error, Failure) else 'client_input_or_transport_error'
        print(json.dumps({'ok': False, 'error': {'code': code}, 'context': context}, ensure_ascii=True))
        return 1
    finally:
        if client:
            client.close()


if __name__ == '__main__':
    raise SystemExit(main())
