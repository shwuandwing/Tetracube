import wave
import struct
import math
import os

def generate_tone(frequency, duration, volume=0.5, type='square', sample_rate=44100):
    num_samples = int(duration * sample_rate)
    samples = []
    for i in range(num_samples):
        if type == 'square':
            val = math.sin(2 * math.pi * frequency * i / sample_rate)
            sample = 32767 * volume * (1 if val > 0 else -1)
        elif type == 'sine':
            sample = 32767 * volume * math.sin(2 * math.pi * frequency * i / sample_rate)
        else: # noise
            import random
            sample = 32767 * volume * random.uniform(-1, 1)
        samples.append(int(sample))
    return samples

def save_wav(file_path, samples, sample_rate=44100):
    with wave.open(file_path, 'w') as f:
        f.setnchannels(1)
        f.setsampwidth(2)
        f.setframerate(sample_rate)
        for s in samples:
            f.writeframesraw(struct.pack('<h', s))

def generate_assets():
    sample_rate = 44100
    if not os.path.exists('assets/sounds'):
        os.makedirs('assets/sounds')

    # BGM - Original catchy-ish chiptune loop
    bgm_notes = [
        (440.00, 0.3), (523.25, 0.3), (659.25, 0.3), (783.99, 0.3),
        (698.46, 0.3), (659.25, 0.3), (587.33, 0.3), (523.25, 0.3),
        (493.88, 0.3), (392.00, 0.3), (440.00, 0.6),
        (0.0, 0.3),
        (440.00, 0.15), (440.00, 0.15), (523.25, 0.3), (440.00, 0.3), (392.00, 0.3),
        (440.00, 0.6)
    ]
    bgm_samples = []
    for freq, dur in bgm_notes:
        if freq > 0:
            bgm_samples.extend(generate_tone(freq, dur, volume=0.3, type='square'))
        else:
            bgm_samples.extend([0] * int(sample_rate * dur))
        bgm_samples.extend([0] * int(sample_rate * 0.02))
    
    save_wav('assets/sounds/bgm.wav', bgm_samples)
    
    # Sound Effects
    save_wav('assets/sounds/move.wav', generate_tone(440, 0.05, volume=0.2, type='sine'))
    save_wav('assets/sounds/rotate.wav', generate_tone(660, 0.08, volume=0.2, type='sine'))
    save_wav('assets/sounds/drop.wav', generate_tone(220, 0.1, volume=0.3, type='square'))
    save_wav('assets/sounds/clear.wav', generate_tone(880, 0.3, volume=0.3, type='sine'))

    print("Generated .wav assets. Converting to .ogg...")
    import subprocess
    for f in ['bgm', 'move', 'rotate', 'drop', 'clear']:
        subprocess.run(['ffmpeg', '-y', '-i', f'assets/sounds/{f}.wav', f'assets/sounds/{f}.ogg'], capture_output=True)
        os.remove(f'assets/sounds/{f}.wav')
    
    print("Done. Assets ready in assets/sounds/")

if __name__ == "__main__":
    generate_assets()
