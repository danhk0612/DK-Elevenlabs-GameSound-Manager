"""Compile a small reusable music/voice pack to common commands; never submit jobs."""
from __future__ import annotations
import argparse
import json
from pathlib import Path
import re
import subprocess
import sys

VOICE_TYPES = {
    'energetic': {'name': '활기찬', 'attack': '이얍!', 'hit': '앗!'},
    'calm': {'name': '차분한', 'attack': '핫!', 'hit': '윽!'},
    'sturdy': {'name': '묵직한', 'attack': '으랏!', 'hit': '으윽!'},
}
MUSIC = {
    'adventure': 'Instrumental lighthearted fantasy game exploration. Warm acoustic plucks, soft woodwinds, gentle percussion, steady moderate tempo. Small memorable motif, restrained dynamics, no vocals, no spoken words. Repeating arrangement suitable for a loop, no dramatic intro or ending.',
    'battle': 'Instrumental tactical fantasy card battle. Playful tense pizzicato strings, crisp hand percussion, short brass accents, steady brisk pulse. Clear repeating motif, moderate intensity, no vocals, no spoken words. Repeating arrangement suitable for a loop, no dramatic intro or ending.',
}


def identifier(value, limit=64):
    if not isinstance(value, str) or not re.fullmatch(r'[a-z0-9_-]{1,' + str(limit) + r'}', value):
        raise ValueError('invalid_identifier')
    return value


def compile_starter(project, account, run, voices, max_reserved, only=None):
    """Explicit reservations are conservative planning amounts, not a billing cap."""
    identifier(project)
    identifier(account)
    identifier(run, 20)
    if type(max_reserved) is not int or not 1 <= max_reserved <= 10000:
        raise ValueError('invalid_starter_budget')
    known = [f'music-{key}' for key in MUSIC]
    known += [f'{kind}-{cue}' for kind in VOICE_TYPES for cue in ('attack', 'hit')]
    selected = known if only is None else list(only)
    if not selected or len(set(selected)) != len(selected) or any(key not in known for key in selected):
        raise ValueError('invalid_starter_selection')
    if any(kind not in VOICE_TYPES for kind in voices):
        raise ValueError('unknown_voice_type')
    total = sum(750 if key.startswith('music-') else 50 for key in selected)
    if total > max_reserved:
        raise ValueError('starter_budget_exceeded')
    commands, characters = [], set()
    for key in selected:
        job = {'op': 'job.submit', 'protocol_version': 1, 'project_id': project,
               'account_id': account, 'job_id': f'{run}-{key}', 'asset_id': f'starter-{key}',
               'version_id': 'take-1', 'name': f'Starter {key}',
               'output_format': 'mp3_44100_128'}
        if key.startswith('music-'):
            job.update(generation_kind='music', model_id='music_v1', text=MUSIC[key[6:]],
                       duration_ms=30000, force_instrumental=True, reserve_credits=750)
        else:
            kind, cue = key.split('-')
            voice = voices.get(kind)
            if not isinstance(voice, str) or not voice or len(voice) > 200 or '\x00' in voice:
                raise ValueError('selected_voice_type_requires_voice_id')
            character = f'voice-type-{kind}'
            dialogue = f'starter-{key}'
            if kind not in characters:
                commands.append({'op': 'character.put', 'protocol_version': 1,
                                 'project_id': project, 'character_id': character,
                                 'name': VOICE_TYPES[kind]['name'], 'voice_id': voice,
                                 'notes': 'Game voice archetype; no real-person trait or gender inference.'})
                characters.add(kind)
            commands.append({'op': 'dialogue.put', 'protocol_version': 1,
                             'project_id': project, 'dialogue_id': dialogue,
                             'character_id': character, 'text': VOICE_TYPES[kind][cue], 'language': 'ko'})
            job.update(generation_kind='tts', model_id='eleven_multilingual_v2',
                       voice_id=voice, text=VOICE_TYPES[kind][cue], language='ko',
                       dialogue_id=dialogue, reserve_credits=50)
        commands.append(job)
    return {'schema_version': 1, 'mode': 'plan_only', 'selected': selected,
            'generation_requests': len(selected), 'reserved_credits': total,
            'billing_cap_guaranteed': False, 'commands': commands}


def main(argv=None):
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--project', required=True)
    parser.add_argument('--account', required=True)
    parser.add_argument('--run', required=True)
    parser.add_argument('--max-reserved', type=int, required=True)
    parser.add_argument('--voice', action='append', default=[], metavar='TYPE=VOICE_ID')
    parser.add_argument('--only', action='append', choices=[f'music-{key}' for key in MUSIC] +
                        [f'{kind}-{cue}' for kind in VOICE_TYPES for cue in ('attack', 'hit')])
    parser.add_argument('--cli', type=Path, required=True, help='Native CLI used only for offline validate')
    args = parser.parse_args(argv)
    try:
        voices = {}
        for entry in args.voice:
            kind, separator, voice = entry.partition('=')
            if not separator or kind in voices:
                raise ValueError('invalid_or_duplicate_voice_mapping')
            voices[kind] = voice
        plan = compile_starter(args.project, args.account, args.run, voices, args.max_reserved, args.only)
        cli = args.cli.resolve(strict=True)
        for command in plan['commands']:
            checked = subprocess.run([str(cli), 'validate'], input=json.dumps(command).encode('utf-8'),
                                     stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL, timeout=15)
            if checked.returncode:
                raise ValueError('native_offline_validation_failed')
        print(json.dumps({'ok': True, 'data': plan}, ensure_ascii=False))
        return 0
    except (ValueError, OSError, subprocess.SubprocessError):
        print(json.dumps({'ok': False, 'error': {'code': 'starter_plan_invalid'}}))
        return 1


if __name__ == '__main__':
    sys.exit(main())
