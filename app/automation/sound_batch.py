"""Optional local-only batch client: preflight all recipes, execute selected items in order."""
from __future__ import annotations
import argparse
import hashlib
import json
import math
from pathlib import Path
import re
import sys

from sound_pipeline import Client, Failure, MAX_FRAME, absolute_file, compile_recipe, execute, fields

MAX_BATCH = 1024 * 1024
MAX_ITEMS = 32
# Only errors with a known terminal/local outcome may advance to another item.
# Transport failures and unknown future errors always stop, even with --keep-going.
CONTINUABLE = frozenset({
    'version_id_conflict', 'asset_metadata_conflict', 'import_integrity_mismatch',
    'review_not_pending', 'review_current_data_mismatch', 'review_id_conflict',
    'review_candidate_unavailable', 'review_candidate_hash_mismatch',
    'review_analysis_mismatch', 'review_analysis_not_found',
    'requirement_not_found', 'requirement_kind_mismatch',
    'audio_requires_attention_failed', 'audio_requires_attention_cancelled',
    'audio_requires_attention_interrupted', 'audio_requires_attention_output_ready',
})


def canonical(value):
    return json.dumps(value, ensure_ascii=True, sort_keys=True, separators=(',', ':'), allow_nan=False)


def identifier(value):
    if not isinstance(value, str) or not re.fullmatch(r'[a-z0-9_-]{1,64}', value):
        raise Failure('invalid_batch_identifier')
    return value


def load_manifest(path):
    with path.open('rb') as stream:
        data = stream.read(MAX_BATCH + 1)
    if len(data) > MAX_BATCH:
        raise Failure('batch_too_large')
    def unique(pairs):
        result = {}
        for key, value in pairs:
            if key in result:
                raise Failure('duplicate_batch_field')
            result[key] = value
        return result
    def finite(_):
        raise Failure('invalid_batch_number')
    try:
        return json.loads(data.decode('utf-8-sig'), object_pairs_hook=unique, parse_constant=finite)
    except (ValueError, UnicodeError):
        raise Failure('invalid_batch_json') from None


def compile_batch(manifest, context=None):
    fields(manifest, ('schema_version', 'batch_id', 'items'))
    if type(manifest['schema_version']) is not int or manifest['schema_version'] != 1:
        raise Failure('unsupported_batch_schema')
    identifier(manifest['batch_id'])
    items = manifest['items']
    if not isinstance(items, list) or not 1 <= len(items) <= MAX_ITEMS:
        raise Failure('batch_requires_1_to_32_items')
    ids, projects, sources, candidates, operations, reviews = set(), {}, {}, set(), set(), set()
    compiled, validations = [], {}
    for index, item in enumerate(items):
        if context is not None:
            context.update(item_index=index + 1)
            context.pop('item_id', None)
        fields(item, ('item_id', 'recipe'))
        iid = identifier(item['item_id'])
        if context is not None:
            context['item_id'] = iid
        if iid in ids:
            raise Failure('duplicate_batch_item_id')
        ids.add(iid)
        recipe = item['recipe']
        if len(canonical(recipe).encode('utf-8')) > MAX_FRAME:
            raise Failure('batch_recipe_too_large')
        plan, requests = compile_recipe(recipe)
        # Hash only validated JSON, not source bytes or any approval provenance.
        pid, aid = recipe['project']['project_id'], recipe['source']['asset_id']
        project_definition = canonical(recipe['project'])
        source_definition = canonical(recipe['source'])
        # Use canonical scalar keys after core field validation below. Invalid
        # scalar types are rejected here too, before any engine call.
        if not isinstance(pid, str) or not isinstance(aid, str):
            raise Failure('invalid_field_type')
        if pid in projects and projects[pid] != project_definition:
            raise Failure('batch_project_conflict')
        projects[pid] = project_definition
        key = (pid, aid)
        if key in sources and sources[key] != source_definition:
            raise Failure('batch_source_conflict')
        sources[key] = source_definition
        version = recipe['candidate']['version_id']
        if not isinstance(version, str) or not isinstance(recipe['source']['version_id'], str):
            raise Failure('invalid_field_type')
        target = (pid, aid, version)
        if target in candidates:
            raise Failure('batch_candidate_conflict')
        candidates.add(target)
        rid = recipe['review']['review_id']
        if not isinstance(rid, str):
            raise Failure('invalid_field_type')
        if rid in reviews:
            raise Failure('batch_review_id_conflict')
        reviews.add(rid)
        for command in requests:
            if command['op'] == 'audio.submit':
                oid = command['operation_id']
                if not isinstance(oid, str):
                    raise Failure('invalid_field_type')
                if oid in operations:
                    raise Failure('batch_operation_id_conflict')
                operations.add(oid)
            validations.setdefault(canonical(command), {'item_id': iid, 'command': command})
        compiled.append({'item_id': iid, 'recipe': recipe, 'plan': plan})
    # Items may share the exact imported source, but cannot consume a candidate
    # from another item. Keep-going therefore never implies dependency replay.
    for item in compiled:
        recipe = item['recipe']
        key = (recipe['project']['project_id'], recipe['source']['asset_id'], recipe['source']['version_id'])
        if key in candidates:
            raise Failure('batch_cross_item_dependency')
    return compiled, list(validations.values()), hashlib.sha256(canonical(manifest).encode('utf-8')).hexdigest()


def item_context(item):
    recipe = item['recipe']
    return {'item_id': item['item_id'], 'project_id': recipe['project']['project_id'],
            'asset_id': recipe['source']['asset_id'], 'version_id': recipe['candidate']['version_id'],
            'review_id': recipe['review']['review_id'], 'stage': 'starting'}


def run_batch(compiled, selected, client_factory, wait_seconds, poll_seconds, keep_going=False,
              on_progress=None, on_item=None):
    results, stopped, interrupted = [], False, False
    for item in compiled:
        context = item_context(item)
        row = {**context, 'status': 'not_run'}
        client = None
        if item['item_id'] not in selected:
            row['status'] = 'not_selected'
        elif not stopped:
            def progress(event):
                if on_progress:
                    on_progress({**context, **event})
            try:
                client = client_factory(item)
                output = execute(client, item['plan'], wait_seconds, poll_seconds, context,
                                 on_progress=progress)
                candidate = output['review']['snapshot']['candidate']
                row.update(status='succeeded', stage='completed', candidate_sha256=candidate['sha256'],
                           review_state=output['review']['state'], matches=output['check']['matches'],
                           approval='not_granted')
            except KeyboardInterrupt:
                row.update(context, status='interrupted', error={'code': 'client_interrupted_work_may_continue'})
                interrupted, stopped = True, True
            except (Failure, OSError, ValueError, TypeError, KeyError) as error:
                code = str(error) if isinstance(error, Failure) else 'client_input_or_transport_error'
                row.update(context, status='failed', error={'code': code})
                stopped = not (keep_going and code in CONTINUABLE)
            finally:
                if client:
                    client.close()
        results.append(row)
        if on_item:
            on_item(row)
    counts = {status: sum(row['status'] == status for row in results)
              for status in ('succeeded', 'failed', 'interrupted', 'not_run', 'not_selected')}
    return {'items': results, 'counts': counts, 'approval': 'not_granted'}, 130 if interrupted else (1 if counts['failed'] else 0)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('manifest', type=Path)
    parser.add_argument('--cli', required=True, type=Path)
    parser.add_argument('--mcp', type=Path)
    parser.add_argument('--validate-only', action='store_true')
    parser.add_argument('--only', action='append', default=[], metavar='ITEM_ID')
    parser.add_argument('--keep-going', action='store_true')
    parser.add_argument('--jsonl', action='store_true', help='Emit item records as completed and one final summary')
    parser.add_argument('--request-seconds', type=float, default=300)
    parser.add_argument('--wait-seconds', type=float, default=300)
    parser.add_argument('--poll-seconds', type=float, default=.5)
    args = parser.parse_args()
    context, validator = {'stage': 'preflight'}, None
    def emit(value):
        print(json.dumps(value, ensure_ascii=True, separators=(',', ':'), allow_nan=False), flush=True)
    try:
        for value, low, high in [(args.request_seconds, 1, 300), (args.wait_seconds, 1, 86400), (args.poll_seconds, .1, 60)]:
            if not math.isfinite(value) or not low <= value <= high:
                raise Failure('invalid_client_timing')
        manifest = load_manifest(args.manifest)
        compiled, requests, digest = compile_batch(manifest, context)
        context.update(batch_id=manifest['batch_id'], manifest_sha256=digest)
        ids = {item['item_id'] for item in compiled}
        selected = set(args.only) if args.only else ids
        if not selected <= ids:
            raise Failure('batch_unknown_selected_item')
        absolute_file(str(args.cli))
        validator = Client(args.cli, timeout=args.request_seconds)
        context.pop('item_index', None)
        for request in requests:
            context['item_id'] = request['item_id']
            validator.validate(request['command'])
        context.pop('item_id', None)
        context.pop('item_index', None)
        # Even selecting one item validates the entire manifest, without engine calls.
        if args.validate_only:
            emit({'type': 'summary', 'ok': True, 'data': {**context, 'items': len(compiled),
                  'selected': len(selected), 'validated_commands': len(requests), 'engine_started': False}})
            return 0
        if args.mcp:
            absolute_file(str(args.mcp))
        context['stage'] = 'running'
        def factory(item):
            client = Client(args.cli, args.mcp, timeout=args.request_seconds)
            try:
                if args.mcp:
                    required = {c['op'] for _, c in item['plan']} | {'audio.get'}
                    client.discover(required)
                return client
            except BaseException:
                client.close()
                raise
        def progress(event):
            print(json.dumps({**context, **event}, ensure_ascii=True, separators=(',', ':')), file=sys.stderr, flush=True)
        def completed(row):
            if args.jsonl:
                emit({'type': 'item', **context, 'item': row})
        data, code = run_batch(compiled, selected, factory, args.wait_seconds, args.poll_seconds,
                               args.keep_going, progress, completed)
        data.update(batch_id=manifest['batch_id'], manifest_sha256=digest,
                    transport='mcp' if args.mcp else 'cli',
                    policy='keep_going_terminal' if args.keep_going else 'fail_stop')
        result = {'type': 'summary', 'ok': code == 0, 'data': data}
        if code:
            result['error'] = {'code': 'client_interrupted_work_may_continue' if code == 130 else 'batch_incomplete'}
        if args.jsonl:
            result['data'] = {k: v for k, v in data.items() if k != 'items'}
        emit(result)
        return code
    except KeyboardInterrupt:
        emit({'type': 'summary', 'ok': False, 'error': {'code': 'client_interrupted_work_may_continue'}, 'context': context})
        return 130
    except (Failure, OSError, ValueError, TypeError, KeyError) as error:
        code = str(error) if isinstance(error, Failure) else 'client_input_or_transport_error'
        emit({'type': 'summary', 'ok': False, 'error': {'code': code}, 'context': context})
        return 1
    finally:
        if validator:
            validator.close()


if __name__ == '__main__':
    raise SystemExit(main())
