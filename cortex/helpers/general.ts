import { HandlerCallback, IAgentRuntime } from '@elizaos/core'
import { ProposalAction } from '@peerme/core-ts'
import config from '../../../config.js'
import { sendAgentAction } from '../../../helpers-eliza.js'
import { AppAgent } from '../../app/types.js'
import { SharedCache } from '../../cache.js'
import { CacheKey } from '../../constants.js'
import { AgentDelegatable, EntityProposable, EntitySignable, Swarm } from '../types.js'

export const sendAgentDelegate = (callback: HandlerCallback | undefined, data: AgentDelegatable, options?: { delay?: number; error?: boolean }) => {
  sendAgentAction(callback, 'AGENT_DELEGATE', data, options)
}

export const getSwarmFromRoom = async (roomId: string): Promise<Swarm | null> => {
  return (await SharedCache.get(CacheKey.Swarm(roomId))) as Swarm | null
}

export const isSwarmRoom = async (roomId: string) => {
  return (await getSwarmFromRoom(roomId)) !== null
}

export const setSwarmForRoom = async (roomId: string, swarm: Swarm) => {
  await SharedCache.set(CacheKey.Swarm(roomId), swarm)
}

export const getCurrentAgentInSwarm = async (runtime: IAgentRuntime, roomId: string): Promise<[AppAgent | null, Swarm | null]> => {
  const swarm = await getSwarmFromRoom(roomId)
  if (!swarm) return [null, null]
  const agent = swarm.agents.find((a) => a.uuid === runtime.agentId)
  if (!agent) return [null, swarm]
  return [agent, swarm]
}

export const isCurrentAgentManagerInRoom = async (runtime: IAgentRuntime, roomId: string) => {
  const swarmInfo = await getCurrentAgentInSwarm(runtime, roomId)
  const [currentAgent, _] = swarmInfo
  if (!swarmInfo) return false
  return currentAgent?.role === config().App.Swarm.ManagerRole
}

export const isValidDelegationObject = (obj: any): boolean => obj && !!obj.agent && !!obj.prompt

export const sendAgentEntityPropose = (
  callback: HandlerCallback | undefined,
  agent: string,
  swarm: string,
  title: string,
  description: string,
  actions: ProposalAction[]
) => {
  if (!agent) throw new Error('Agent is required')
  if (!swarm) throw new Error('Swarm is required')
  if (!title) throw new Error('Title is required')
  if (!description) throw new Error('Description is required')
  if (!actions || actions.length === 0) throw new Error('Actions are required')
  const executable: EntityProposable = { agent, swarm, title, description, actions }
  sendAgentAction(callback, 'ENTITY_PROPOSE', executable)
}

export const sendAgentEntitySign = (callback: HandlerCallback | undefined, agent: string, swarm: string, proposal: string, sign: boolean, reason: string) => {
  if (!agent) throw new Error('Agent is required')
  if (!swarm) throw new Error('Swarm is required')
  if (!proposal) throw new Error('Proposal is required')
  if (!reason) throw new Error('Reason is required')
  const executable: EntitySignable = { agent, swarm, proposal, sign, reason }
  sendAgentAction(callback, 'ENTITY_SIGN', executable)
}
