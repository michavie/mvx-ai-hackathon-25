import { createSlice, PayloadAction } from '@reduxjs/toolkit'
import { EntityDirectExecuteRequest, Swarm } from '../types'

type SwarmState = {
  active: Swarm | null
  swarms: Swarm[]
  directExecuteRequest: EntityDirectExecuteRequest | null
}

const initialState: SwarmState = {
  active: null,
  swarms: [],
  directExecuteRequest: null,
}

export const slice = createSlice({
  name: 'swarm',
  initialState,
  reducers: {
    setActiveSwarm: (state, action: PayloadAction<Swarm | null>) => {
      state.active = action.payload
    },
    setSwarms: (state, action: PayloadAction<Swarm[]>) => {
      state.swarms = action.payload
    },
    setDirectExecuteRequest: (state, action: PayloadAction<EntityDirectExecuteRequest>) => {
      state.directExecuteRequest = action.payload
    },
    resetDirectExecuteRequest: (state) => {
      state.directExecuteRequest = null
    },
  },
})

export const { setActiveSwarm, setSwarms, setDirectExecuteRequest, resetDirectExecuteRequest } = slice.actions

export const swarmReducer = slice.reducer
