import { ProposalAction } from '@peerme/core-ts'
import { AppAgent } from '../app/types.js'

export type AppSwarm = {
  id: string
  chain: {
    name: string
    urls: {
      api: string
    }
    chainId: string
    blockTimeMs: number
  }
  extId: string
  room: string
  createdAt: string
  agents: AppAgent[]
}

export type Swarm = {
  id: string
  extId: string
  name: string
  description: string
  address: string
  room: string
  agents: AppAgent[]
}

// Action payload sent from cortex to client
export type AgentDelegatable = {
  agent: string
  swarm?: string
  room: string
  prompt: string
}

// Action payload sent from cortex to client
export type EntityProposable = {
  agent: string
  swarm: string
  title: string
  description: string
  actions: ProposalAction[]
}

// Action payload sent from cortex to client
export type EntitySignable = {
  agent: string
  swarm: string
  proposal: string
  sign: boolean
  reason: string
}

export type ActionPlan = {
  prompt: string
  steps: ActionPlanStep[]
}

export type ActionPlanStep = {
  id: string
  action: string
  agent: string
  done?: boolean
}
