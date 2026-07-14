import { ref, computed } from 'vue';
import type { BytecodeLine, DebugState, DebugCommand, DebugRecording } from '../types';

export function useDebugger() {
  const ws = ref<WebSocket | null>(null);
  const isConnected = ref(false);
  const isDebugging = ref(false);
  const bytecode = ref<BytecodeLine[]>([]);
  const state = ref<DebugState | null>(null);
  const error = ref<string | null>(null);

  // Recording state
  const isRecording = ref(false);
  const recording = ref<DebugRecording | null>(null);

  // Maps derived from bytecode
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

  function connect(source: string, initialBreakpoints: number[] = []) {
    if (ws.value) return;

    const protocol = window.location.protocol === 'https:' ? 'wss:' : 'ws:';
    const socket = new WebSocket(`${protocol}//${window.location.host}/api/debug/ws`);

    socket.onopen = () => {
      isConnected.value = true;
      isDebugging.value = true;
      // Send source to start debug session
      socket.send(JSON.stringify({ type: 'debug.start', source }));
      // Send initial breakpoints if any
      if (initialBreakpoints.length > 0) {
        socket.send(JSON.stringify({ type: 'breakpoints.set', lines: initialBreakpoints }));
      }
    };

    socket.onmessage = (event) => {
      const msg = JSON.parse(event.data);
      handleMessage(msg);
    };

    socket.onerror = (e) => {
      error.value = 'WebSocket error';
      console.error('Debug WS error:', e);
    };

    socket.onclose = () => {
      isConnected.value = false;
      isDebugging.value = false;
      ws.value = null;
    };

    ws.value = socket;
  }

  function handleMessage(msg: any) {
    switch (msg.type) {
      case 'bytecode':
        bytecode.value = msg.lines || [];
        if (isRecording.value && recording.value) {
          recording.value.bytecode = msg.lines || [];
        }
        break;
      case 'state':
        state.value = msg.data;
        if (isRecording.value && recording.value) {
          recording.value.events.push({ type: 'state', state: msg.data });
        }
        if (msg.data.status === 'finished' || msg.data.status === 'error') {
          isDebugging.value = false;
        }
        break;
      case 'error':
        error.value = msg.message;
        isDebugging.value = false;
        break;
    }
  }

  function sendCommand(cmd: DebugCommand) {
    if (ws.value?.readyState === WebSocket.OPEN) {
      ws.value.send(JSON.stringify({ type: 'command', cmd }));
    }
    if (isRecording.value && recording.value) {
      recording.value.events.push({ type: 'command', cmd });
    }
  }

  function setBreakpoints(lines: number[]) {
    if (ws.value?.readyState === WebSocket.OPEN) {
      ws.value.send(JSON.stringify({ type: 'breakpoints.set', lines }));
    }
    if (isRecording.value && recording.value) {
      recording.value.events.push({ type: 'breakpoints', lines });
    }
  }

  function stop() {
    sendCommand('stop');
    ws.value?.close();
    ws.value = null;
    isDebugging.value = false;
    state.value = null;
    bytecode.value = [];
    error.value = null;
  }

  // Recording controls
  function startRecording(source: string, initialBreakpoints: number[]) {
    recording.value = {
      version: 1,
      createdAt: new Date().toISOString(),
      source,
      initialBreakpoints: [...initialBreakpoints],
      bytecode: [],
      events: [],
    };
    isRecording.value = true;
  }

  function stopRecording() {
    isRecording.value = false;
    return recording.value;
  }

  function exportRecording() {
    if (!recording.value) return;
    const blob = new Blob([JSON.stringify(recording.value, null, 2)], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `replay_${Date.now()}.autoreplay`;
    a.click();
    URL.revokeObjectURL(url);
  }

  return {
    isConnected,
    isDebugging,
    bytecode,
    state,
    error,
    lineToOffsets,
    offsetToLine,
    connect,
    sendCommand,
    setBreakpoints,
    stop,
    // Recording
    isRecording,
    recording,
    startRecording,
    stopRecording,
    exportRecording,
  };
}
