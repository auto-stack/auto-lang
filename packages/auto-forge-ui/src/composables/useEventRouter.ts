import { useGateInbox } from './useGateInbox'
import { useRelay } from './useRelay'

export type ViewType = 'chat' | 'specs' | 'relay'

export interface SSEEvent {
  type: 'run_started' | 'step_advanced' | 'spec_written'
       | 'gate_reached' | 'gate_resolved'
       | 'handoff_submitted' | 'run_completed'
  runId: string
  payload: Record<string, unknown>
}

// ─── Callback registry for view-specific reactions ──────────────────────────
interface EventCallbacks {
  onReport?: (report: Record<string, unknown>) => void
  onFlashSection?: (sectionId: string) => void
  onLiveLog?: (entry: Record<string, unknown>) => void
}

let _callbacks: EventCallbacks = {}

export function setEventCallbacks(cbs: EventCallbacks) {
  _callbacks = cbs
}

export function useEventRouter() {
  const gateInbox = useGateInbox()
  const relay = useRelay()

  function handleEvent(event: SSEEvent, _currentView: ViewType) {
    switch (event.type) {
      case 'run_started':
        // Highlight run in sidebar (ambient)
        break

      case 'step_advanced':
        // NO chat message — ambient only
        // Animate node in relay view (handled by reactive run state)
        break

      case 'spec_written': {
        const sectionId = event.payload.section_id as string | undefined
        if (sectionId && _callbacks.onFlashSection) {
          _callbacks.onFlashSection(sectionId)
        }
        break
      }

      case 'gate_reached': {
        const gate = {
          gateId: (event.payload.gate_id as string) || `${event.runId}-${event.payload.step_id}`,
          runId: event.runId,
          profession: (event.payload.profession as string) || 'unknown',
          title: (event.payload.title as string) || `${event.payload.profession_id} needs approval`,
          sectionId: event.payload.section_id as string | undefined,
          since: Date.now(),
        }
        gateInbox.registerGate(gate)
        break
      }

      case 'gate_resolved': {
        const gateId = (event.payload.gate_id as string) || `${event.runId}-${event.payload.step_id}`
        const decision = (event.payload.decision as 'approved' | 'rejected') || 'approved'
        gateInbox.resolveGate(gateId, decision)
        break
      }

      case 'handoff_submitted': {
        if (_callbacks.onLiveLog) {
          _callbacks.onLiveLog(event.payload)
        }
        break
      }

      case 'run_completed': {
        if (_callbacks.onReport) {
          _callbacks.onReport(event.payload)
        }
        // Mark run done in relay state
        if (relay.currentRun.value?.run_id === event.runId) {
          relay.loadRun(event.runId)
        }
        break
      }
    }
  }

  return { handleEvent, setEventCallbacks }
}
