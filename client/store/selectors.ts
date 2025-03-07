import { AppState } from '../../../store'

export const selectActiveSwarm = (state: AppState) => state.swarm.active

export const selectSwarms = (state: AppState) => state.swarm.swarms

export const selectDirectExecuteRequest = (state: AppState) => state.swarm.directExecuteRequest
