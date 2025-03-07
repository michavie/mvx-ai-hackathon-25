import { Evaluator, generateObject, IAgentRuntime, ModelClass } from '@elizaos/core'
import { Proposal } from '@peerme/core-ts'
import { z } from 'zod'
import { AgentsStore } from '../../agent/agents-store.js'
import config from '../../config.js'
import { CreditUsageStore } from '../../credit-usage-store.js'
import { getSwarmFromRoom, sendAgentEntitySign } from './helpers/general.js'
import { clearActiveProposalForRoom, getActiveProposalForRoom, hasActiveProposalForRoom } from './helpers/proposal.js'

export const swarmProposalSignEvaluator: Evaluator = {
  name: 'SWARM_PROPOSAL_SIGN',
  similes: [],
  description: 'Should always run when a proposal is active.',
  validate: async (runtime, message) => await hasActiveProposalForRoom(message.roomId),
  handler: async (runtime, message, state, options, callback) => {
    const proposal = await getActiveProposalForRoom(message.roomId)
    const swarm = await getSwarmFromRoom(message.roomId)
    if (!proposal || !swarm) return

    const potentialSigners = swarm.agents.filter((a) => a.role === config().App.Swarm.ManagerRole)
    potentialSigners.forEach(async (agent) => {
      const agentRuntime = AgentsStore.getInstance().getAgent(agent.uuid)
      if (!agentRuntime) return
      const { sign, reason } = await generateAgentSigningDecision(agentRuntime, proposal)
      sendAgentEntitySign(callback, agent.uuid, swarm.id, proposal.id, sign, reason)
    })

    await clearActiveProposalForRoom(message.roomId)
  },
  examples: [],
}

const generateAgentSigningDecision = async (runtime: IAgentRuntime, proposal: Proposal): Promise<{ sign: boolean; reason: string }> => {
  let actionTemplate = `Make your best decision if you as the agent should sign this proposal based on everything you know.\n`
  actionTemplate += `Proposal: ${JSON.stringify(proposal)}\n`
  actionTemplate += 'Return a JSON object containing only the fields: '
  actionTemplate += '{"sign": "true or false", "reason": "very short reason for the decision to show the user"}'

  const schema = z.object({ sign: z.boolean(), reason: z.string() })
  const signResult = await generateObject({ runtime, context: actionTemplate, modelClass: ModelClass.SMALL, schema })
  CreditUsageStore.getInstance().increaseCreditUsage(runtime.agentId, signResult.usage.totalTokens)
  const sign = signResult.object as { sign: boolean; reason: string }

  return sign
}
