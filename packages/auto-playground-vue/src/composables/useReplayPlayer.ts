import { ref, computed } from 'vue';
import type { DebugRecording, DebugState, BytecodeLine } from '../types';

export function useReplayPlayer() {
  const isActive = ref(false);
  const recording = ref<DebugRecording | null>(null);
  const currentIndex = ref(0);
  const isPlaying = ref(false);
  let playTimer: ReturnType<typeof setInterval> | null = null;

  const events = computed(() => recording.value?.events ?? []);

  // Only state events are "frames" (pause points)
  const stateEvents = computed(() =>
    events.value.filter((e) => e.type === 'state').map((e, idx) => ({ ...e, frameIndex: idx }))
  );

  const totalFrames = computed(() => stateEvents.value.length);

  const currentState = computed<DebugState | null>(() => {
    if (!isActive.value || !recording.value) return null;
    const ev = events.value[currentIndex.value];
    if (ev?.type === 'state') return ev.state;
    // If current event is a command, look backward for the nearest state
    for (let i = currentIndex.value; i >= 0; i--) {
      const e = events.value[i];
      if (e.type === 'state') return e.state;
    }
    return null;
  });

  const bytecode = computed<BytecodeLine[]>(() => recording.value?.bytecode ?? []);

  const lineToOffsets = computed(() => {
    const map: Record<number, number[]> = {};
    for (const line of bytecode.value) {
      if (line.line !== undefined) {
        if (!map[line.line]) map[line.line] = [];
        map[line.line].push(line.offset);
      }
    }
    return map;
  });

  const offsetToLine = computed(() => {
    const map: Record<number, number> = {};
    for (const line of bytecode.value) {
      if (line.line !== undefined) {
        map[line.offset] = line.line;
      }
    }
    return map;
  });

  function load(data: DebugRecording) {
    stop();
    recording.value = data;
    isActive.value = true;
    currentIndex.value = 0;
  }

  function stop() {
    pause();
    isActive.value = false;
    recording.value = null;
    currentIndex.value = 0;
  }

  function play() {
    if (isPlaying.value) return;
    isPlaying.value = true;
    playTimer = setInterval(() => {
      if (currentIndex.value >= events.value.length - 1) {
        pause();
        return;
      }
      currentIndex.value++;
    }, 800);
  }

  function pause() {
    isPlaying.value = false;
    if (playTimer) {
      clearInterval(playTimer);
      playTimer = null;
    }
  }

  function stepForward() {
    pause();
    if (currentIndex.value < events.value.length - 1) {
      currentIndex.value++;
    }
  }

  function stepBackward() {
    pause();
    if (currentIndex.value > 0) {
      currentIndex.value--;
    }
  }

  function seek(targetIndex: number) {
    pause();
    currentIndex.value = Math.max(0, Math.min(events.value.length - 1, targetIndex));
  }

  return {
    isActive,
    recording,
    currentIndex,
    isPlaying,
    currentState,
    bytecode,
    lineToOffsets,
    offsetToLine,
    totalFrames,
    load,
    stop,
    play,
    pause,
    stepForward,
    stepBackward,
    seek,
  };
}
