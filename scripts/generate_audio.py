import math
import os
import struct
import subprocess
import wave
from pathlib import Path

SAMPLE_RATE = 44100
ROOT_DIR = Path(__file__).resolve().parents[1]
SOUNDS_DIR = ROOT_DIR / "assets" / "sounds"

NOTE_OFFSETS = {
    "C": 0,
    "C#": 1,
    "DB": 1,
    "D": 2,
    "D#": 3,
    "EB": 3,
    "E": 4,
    "F": 5,
    "F#": 6,
    "GB": 6,
    "G": 7,
    "G#": 8,
    "AB": 8,
    "A": 9,
    "A#": 10,
    "BB": 10,
    "B": 11,
}


def oscillator(phase, waveform):
    if waveform == "sine":
        return math.sin(phase)
    if waveform == "triangle":
        return (2.0 / math.pi) * math.asin(math.sin(phase))
    if waveform == "soft_square":
        return math.tanh(1.8 * math.sin(phase))
    raise ValueError(f"Unsupported waveform: {waveform}")


def envelope(sample_index, num_samples, attack_samples, release_samples):
    value = 1.0
    if attack_samples > 0 and sample_index < attack_samples:
        progress = (sample_index + 1) / attack_samples
        value *= 0.5 - 0.5 * math.cos(progress * math.pi)

    if release_samples > 0 and sample_index >= num_samples - release_samples:
        remaining = max(0.0, (num_samples - sample_index - 1) / release_samples)
        value *= 0.5 - 0.5 * math.cos(remaining * math.pi)

    return value


def note_to_frequency(note):
    if note in (None, "", "R"):
        return 0.0

    normalized = note.strip().upper()
    octave = int(normalized[-1])
    pitch_class = normalized[:-1]
    midi_note = 12 * (octave + 1) + NOTE_OFFSETS[pitch_class]
    return 440.0 * (2.0 ** ((midi_note - 69) / 12.0))


def transpose(note, semitones):
    if note in (None, "", "R"):
        return note

    normalized = note.strip().upper()
    octave = int(normalized[-1])
    pitch_class = normalized[:-1]
    midi_note = 12 * (octave + 1) + NOTE_OFFSETS[pitch_class] + semitones

    octave = (midi_note // 12) - 1
    offset = midi_note % 12
    reverse_offsets = {
        0: "C",
        1: "C#",
        2: "D",
        3: "D#",
        4: "E",
        5: "F",
        6: "F#",
        7: "G",
        8: "G#",
        9: "A",
        10: "A#",
        11: "B",
    }
    return f"{reverse_offsets[offset]}{octave}"


def build_tone(
    frequency,
    duration,
    *,
    volume=0.5,
    partials=None,
    attack=0.01,
    release=0.08,
    vibrato_depth=0.0,
    vibrato_rate=5.0,
):
    num_samples = max(1, int(duration * SAMPLE_RATE))
    if frequency <= 0:
        return [0.0] * num_samples

    partials = partials or [(1.0, 1.0, "sine")]
    attack_samples = min(int(attack * SAMPLE_RATE), max(1, num_samples // 3))
    release_samples = min(int(release * SAMPLE_RATE), max(1, num_samples // 2))
    phase = 0.0
    samples = []

    for sample_index in range(num_samples):
        time_seconds = sample_index / SAMPLE_RATE
        modulated_frequency = frequency * (
            1.0 + vibrato_depth * math.sin(2.0 * math.pi * vibrato_rate * time_seconds)
        )
        phase += (2.0 * math.pi * modulated_frequency) / SAMPLE_RATE

        sample = 0.0
        for multiple, amplitude, waveform in partials:
            sample += amplitude * oscillator(phase * multiple, waveform)

        sample *= volume * envelope(sample_index, num_samples, attack_samples, release_samples)
        samples.append(sample)

    return samples


def build_sweep(
    start_frequency,
    end_frequency,
    duration,
    *,
    volume=0.5,
    partials=None,
    attack=0.002,
    release=0.05,
):
    num_samples = max(1, int(duration * SAMPLE_RATE))
    partials = partials or [(1.0, 1.0, "sine")]
    attack_samples = min(int(attack * SAMPLE_RATE), max(1, num_samples // 4))
    release_samples = min(int(release * SAMPLE_RATE), max(1, num_samples // 2))
    phase = 0.0
    samples = []

    for sample_index in range(num_samples):
        progress = sample_index / max(1, num_samples - 1)
        if start_frequency > 0 and end_frequency > 0:
            frequency = start_frequency * ((end_frequency / start_frequency) ** progress)
        else:
            frequency = start_frequency + (end_frequency - start_frequency) * progress
        phase += (2.0 * math.pi * frequency) / SAMPLE_RATE

        sample = 0.0
        for multiple, amplitude, waveform in partials:
            sample += amplitude * oscillator(phase * multiple, waveform)

        sample *= volume * envelope(sample_index, num_samples, attack_samples, release_samples)
        samples.append(sample)

    return samples


def mix_in(destination, samples, start_index=0):
    limit = min(len(destination), start_index + len(samples))
    for destination_index in range(start_index, limit):
        destination[destination_index] += samples[destination_index - start_index]


def master(samples, drive=1.1, peak=0.85):
    driven = [math.tanh(sample * drive) for sample in samples]
    max_sample = max((abs(sample) for sample in driven), default=1.0) or 1.0
    scale = peak / max_sample
    return [sample * scale for sample in driven]


def save_wav(file_path, samples):
    file_path = Path(file_path)
    int_samples = [max(-32767, min(32767, int(sample * 32767))) for sample in samples]
    with wave.open(str(file_path), "wb") as wav_file:
        wav_file.setnchannels(1)
        wav_file.setsampwidth(2)
        wav_file.setframerate(SAMPLE_RATE)
        frames = b"".join(struct.pack("<h", sample) for sample in int_samples)
        wav_file.writeframes(frames)


def render_sequence(total_beats, bpm, events):
    total_samples = int(total_beats * 60.0 / bpm * SAMPLE_RATE)
    track = [0.0] * total_samples

    for event in events:
        start_beats = event["start"]
        duration_beats = event["duration"]
        note = event["note"]
        if note in (None, "", "R"):
            continue

        samples = build_tone(
            note_to_frequency(note),
            duration_beats * 60.0 / bpm,
            volume=event.get("volume", 0.2),
            partials=event.get("partials"),
            attack=event.get("attack", 0.01),
            release=event.get("release", 0.08),
            vibrato_depth=event.get("vibrato_depth", 0.0),
            vibrato_rate=event.get("vibrato_rate", 5.0),
        )
        start_index = int(start_beats * 60.0 / bpm * SAMPLE_RATE)
        mix_in(track, samples, start_index)

    return track


def mix_tracks(*tracks):
    length = max((len(track) for track in tracks), default=0)
    mixed = [0.0] * length
    for track in tracks:
        for index, sample in enumerate(track):
            mixed[index] += sample
    return mixed


def create_bgm():
    bpm = 96
    bars = [
        {
            "chord": ["C4", "E4", "G4", "B4"],
            "bass": ["C2", "G2", "C3", "G2"],
            "melody": ["E5", "G5", "A5", "G5"],
        },
        {
            "chord": ["A3", "C4", "E4", "G4"],
            "bass": ["A2", "E3", "A2", "E3"],
            "melody": ["E5", "C5", "D5", "E5"],
        },
        {
            "chord": ["F3", "A3", "C4", "E4"],
            "bass": ["F2", "C3", "F2", "C3"],
            "melody": ["A4", "C5", "D5", "C5"],
        },
        {
            "chord": ["G3", "A3", "D4", "G4"],
            "bass": ["G2", "D3", "G2", "D3"],
            "melody": ["D5", "B4", "G4", "A4"],
        },
        {
            "chord": ["E3", "G3", "B3", "D4"],
            "bass": ["E2", "B2", "E3", "B2"],
            "melody": ["G4", "B4", "D5", "B4"],
        },
        {
            "chord": ["A3", "C4", "E4", "G4"],
            "bass": ["A2", "E3", "A2", "E3"],
            "melody": ["A4", "C5", "E5", "C5"],
        },
        {
            "chord": ["F3", "A3", "C4", "E4"],
            "bass": ["F2", "C3", "F2", "C3"],
            "melody": ["G4", "A4", "C5", "A4"],
        },
        {
            "chord": ["G3", "A3", "D4", "G4"],
            "bass": ["G2", "D3", "G2", "D3"],
            "melody": ["D5", "B4", "A4", "G4"],
        },
    ]

    total_beats = len(bars) * 4
    pad_events = []
    bass_events = []
    melody_events = []
    sparkle_events = []

    for bar_index, bar in enumerate(bars):
        bar_start = bar_index * 4.0

        for note in bar["chord"]:
            pad_events.append(
                {
                    "start": bar_start,
                    "duration": 4.0,
                    "note": note,
                    "volume": 0.05,
                    "partials": [(1.0, 0.8, "sine"), (2.0, 0.16, "triangle")],
                    "attack": 0.08,
                    "release": 0.22,
                }
            )

        for step, note in enumerate(bar["bass"]):
            bass_events.append(
                {
                    "start": bar_start + step,
                    "duration": 0.9,
                    "note": note,
                    "volume": 0.16,
                    "partials": [(1.0, 0.92, "sine"), (2.0, 0.15, "triangle")],
                    "attack": 0.01,
                    "release": 0.14,
                }
            )

        for step, note in enumerate(bar["melody"]):
            melody_events.append(
                {
                    "start": bar_start + step,
                    "duration": 0.86,
                    "note": note,
                    "volume": 0.11,
                    "partials": [
                        (1.0, 0.7, "triangle"),
                        (2.0, 0.2, "sine"),
                        (3.0, 0.05, "soft_square"),
                    ],
                    "attack": 0.015,
                    "release": 0.11,
                    "vibrato_depth": 0.0025,
                    "vibrato_rate": 5.2,
                }
            )

        arp_pattern = [0, 1, 2, 1, 3, 1, 2, 1]
        arp_notes = [transpose(bar["chord"][index], 12) for index in arp_pattern]
        for step, note in enumerate(arp_notes):
            sparkle_events.append(
                {
                    "start": bar_start + (step * 0.5),
                    "duration": 0.42,
                    "note": note,
                    "volume": 0.05,
                    "partials": [(1.0, 0.65, "triangle"), (2.0, 0.12, "sine")],
                    "attack": 0.005,
                    "release": 0.08,
                }
            )

    pad = render_sequence(total_beats, bpm, pad_events)
    bass = render_sequence(total_beats, bpm, bass_events)
    melody = render_sequence(total_beats, bpm, melody_events)
    sparkle = render_sequence(total_beats, bpm, sparkle_events)
    return master(mix_tracks(pad, bass, melody, sparkle), drive=1.08, peak=0.82)


def create_move_sfx():
    return master(
        build_sweep(
            520.0,
            640.0,
            0.05,
            volume=0.28,
            partials=[(1.0, 0.9, "sine"), (2.0, 0.12, "triangle")],
            attack=0.002,
            release=0.03,
        ),
        drive=1.0,
        peak=0.75,
    )


def create_rotate_sfx():
    return master(
        build_sweep(
            540.0,
            840.0,
            0.08,
            volume=0.24,
            partials=[(1.0, 0.75, "triangle"), (2.0, 0.2, "sine")],
            attack=0.002,
            release=0.05,
        ),
        drive=1.0,
        peak=0.74,
    )


def create_drop_sfx():
    return master(
        build_sweep(
            210.0,
            96.0,
            0.12,
            volume=0.34,
            partials=[(1.0, 0.92, "sine"), (2.0, 0.18, "triangle")],
            attack=0.002,
            release=0.08,
        ),
        drive=1.02,
        peak=0.78,
    )


def create_clear_sfx():
    samples = [0.0] * int(0.42 * SAMPLE_RATE)
    notes = ["E5", "G5", "C6", "E6"]
    for index, note in enumerate(notes):
        tone = build_tone(
            note_to_frequency(note),
            0.12,
            volume=0.18,
            partials=[(1.0, 0.8, "triangle"), (2.0, 0.16, "sine")],
            attack=0.004,
            release=0.08,
        )
        mix_in(samples, tone, int(index * 0.07 * SAMPLE_RATE))
    return master(samples, drive=1.0, peak=0.8)


def convert_to_ogg(wav_path):
    ogg_path = wav_path.with_suffix(".ogg")
    subprocess.run(
        ["ffmpeg", "-y", "-i", str(wav_path), str(ogg_path)],
        check=True,
        capture_output=True,
    )
    os.remove(wav_path)


def generate_assets():
    SOUNDS_DIR.mkdir(parents=True, exist_ok=True)

    assets = {
        "bgm": create_bgm(),
        "move": create_move_sfx(),
        "rotate": create_rotate_sfx(),
        "drop": create_drop_sfx(),
        "clear": create_clear_sfx(),
    }

    for name, samples in assets.items():
        wav_path = SOUNDS_DIR / f"{name}.wav"
        save_wav(wav_path, samples)
        convert_to_ogg(wav_path)

    print(f"Generated audio assets in {SOUNDS_DIR}")


if __name__ == "__main__":
    generate_assets()
