import type { AgentPubKey } from '@holochain/conductor-api'
import { _b } from '@ctx-core/object'
import { rpc_send_folk_lore_b, rpc_send_heartbeat_b } from '@syn-ui/zome-client'
import { am_i_scribe_b, folks_b, session_info_scribe_b } from '../session'
import { _scribe_signal_folk_pubKey_a1_b } from '../delta'
import { Timer } from './Timer'
// const outOfSessionTimout = 30 * 1000
const outOfSessionTimout = 8 * 1000 // testing code :)
// const heartbeatInterval = 15 * 1000 // 15 seconds
const heartbeat_interval = 30 * 1000 // for testing ;)
export const scribe_heartbeat_timer_b = _b<Timer>('scribe_heartbeat_timer', (ctx)=>{
    const am_i_scribe = am_i_scribe_b(ctx)
    const folks = folks_b(ctx)
    return new Timer(async ()=>{
        if (am_i_scribe.$ === true) {
            // examine folks last seen time and see if any have crossed the session out-of-session
            // timeout so we can tell everybody else about them having dropped.
            const gone:AgentPubKey[] = []
            const $folks = folks.$
            for (const [pubKeyStr, folk] of Object.entries($folks)) {
                if (folk.inSession && (Date.now() - ($folks[pubKeyStr].lastSeen || 0) > outOfSessionTimout)) {
                    folk.inSession = false
                    gone.push($folks[pubKeyStr].pubKey)
                }
            }
            if (gone.length > 0) {
                folks.$ = $folks
                const rpc_send_folk_lore = rpc_send_folk_lore_b(ctx)
                const _scribe_signal_folk_pubKey_a1 = _scribe_signal_folk_pubKey_a1_b(ctx)
                await rpc_send_folk_lore({
                    participants: _scribe_signal_folk_pubKey_a1(),
                    data: { gone }
                })
            }
        } else {
            // I'm not the scribe so send them a heartbeat
            const rpc_send_heartbeat = rpc_send_heartbeat_b(ctx)
            const session_info_scribe = session_info_scribe_b(ctx)
            await rpc_send_heartbeat({
                scribe: session_info_scribe.$!,
                data: 'Hello'
            })
        }
    }, heartbeat_interval)
})
