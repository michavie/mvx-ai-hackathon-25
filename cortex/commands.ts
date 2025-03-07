import { elizaLogger } from '@elizaos/core'
import { getProposalRequest } from '@peerme/core-ts'
import { getHttpService } from '../../http.js'
import { ChatClientAction } from '../../types-shared.js'
import { CommandExecutor } from '../commands.js'
import { getSwarmFromRoom } from './helpers/general.js'
import { setActiveProposalForRoom } from './helpers/proposal.js'
import { swarmProposalSignEvaluator } from './swarmProposalSignEvaluator.js'

export const executeActivateProposal: CommandExecutor = async (context, args) => {
  const proposalId = args[0]
  const httpPeerme = getHttpService(context.env, 'peerme')
  const proposalRes = await getProposalRequest(httpPeerme, proposalId)
  if (!proposalRes.ok || !proposalRes.data) throw new Error(`Failed to get proposal ${proposalId}`)
  const proposal = proposalRes.data
  const swarm = await getSwarmFromRoom(context.room)
  if (!swarm) throw new Error(`Swarm not found for room ${context.room}`)
  if (swarm.extId !== proposal.entity.id) throw new Error(`Proposal ${proposalId} does not belong to swarm ${swarm.name}`)
  await setActiveProposalForRoom(context.room, proposal)
  elizaLogger.info(`Swarm proposal ${proposalId} set as active for room ${context.room}`)
  swarmProposalSignEvaluator.handler(context.runtime, context.message, undefined, undefined, async (response) => {
    context.handlers.onAction(response.action as ChatClientAction, response.text)
    return [context.message]
  })
}
