import { Entity, ProposalAction } from '@peerme/core-ts'
import { Agent } from '../Agent/types'
import { Chain } from '../Chain/types'
import { User } from '../User/types'

export type Swarm = {
  id: string
  chain: Chain
  extId: string
  entity?: Entity
  room: string
  createdAt: string
  users: User[]
  agents: Agent[]
}

export type EntityDirectExecuteRequest = {
  title?: string
  entity: Entity
  actions: ProposalAction[]
  onClose?: () => void
  onSent?: (txHash: string) => void
  onExecuted?: () => void
}
