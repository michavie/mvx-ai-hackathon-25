import { Action, elizaLogger, generateObject, HandlerCallback, ModelClass } from '@elizaos/core'
import { TokenTransfer } from '@multiversx/sdk-core/out/tokens.js'
import { ProposalAction } from '@peerme/core-ts'
import { Warp, WarpBuilder, WarpContractAction, WarpIndex, WarpSearchHit } from '@vleap/warps'
import { z } from 'zod'
import { CreditUsageStore } from '../../credit-usage-store.js'
import { getWarpConfig } from '../../helpers.js'
import { AppEnv } from '../../types.js'
import { getAppEnvFromMessage } from '../helpers.js'
import { generateWarpSearchQuery } from '../warps/helpers.js'
import { getSwarmFromRoom, isCurrentAgentManagerInRoom, sendAgentEntityPropose } from './helpers/general.js'

export const swarmProposalCreateAction: Action = {
  name: 'SWARM_PROPOSAL_CREATE',
  similes: [],
  description: 'Used for any kind of blockchain action like asset transfer, smart contract call, or other blockchain app interactions.',
  validate: async (runtime, message, state): Promise<boolean> => await isCurrentAgentManagerInRoom(runtime, message.roomId),
  handler: async (runtime, message, state, options, callback): Promise<unknown> => {
    const env = getAppEnvFromMessage(message)
    elizaLogger.log('SWARM_PROPOSE', env)
    const swarm = await getSwarmFromRoom(message.roomId)
    if (!swarm) return false

    const searchQuery = await generateWarpSearchQuery(runtime, message.content)

    const warpIndex = new WarpIndex(getWarpConfig(env))
    const searchHits = await warpIndex.search(searchQuery)
    console.log('SWARM_PROPOSE: searchHits', searchHits)

    const warpAction = await findWarpAction(env, searchHits, callback)
    if (!warpAction) return false

    let actionTemplate = `Your job is to propose an action to the swarm. The action should be a smart contract action.\n`
    actionTemplate += `User prompt: ${message.content.text}\n`
    actionTemplate += 'Delegate by returning a JSON object containing only the fields: '
    actionTemplate += '{"title": "the title of the action", "description": "the description of the action and interesting content for the user"}'

    const schema = z.object({ title: z.string(), description: z.string() })
    const proposalResult = await generateObject({ runtime, context: actionTemplate, modelClass: ModelClass.SMALL, schema })
    CreditUsageStore.getInstance().increaseCreditUsage(runtime.agentId, proposalResult.usage.totalTokens)
    const proposal = proposalResult.object as { title: string; description: string }

    const actions = [toProposalAction(warpAction, [], [])]

    sendAgentEntityPropose(callback, runtime.agentId, swarm.id, proposal.title, proposal.description, actions)

    return true
  },
  examples: [],
}

const findWarpAction = async (env: AppEnv, searchHits: WarpSearchHit[], callback: HandlerCallback | undefined): Promise<WarpContractAction | null> => {
  const warpBuilder = new WarpBuilder(getWarpConfig(env))

  let firstSearchHitWithContractAction: Warp | null = null

  for (const hit of searchHits) {
    try {
      const warp = await warpBuilder.createFromTransactionHash(hit.hash)
      if (warp?.actions.some((action) => action.type === 'contract')) {
        firstSearchHitWithContractAction = warp
        break
      }
    } catch (error) {
      return null
    }
  }

  if (!firstSearchHitWithContractAction) {
    return null
  }

  const warpAction = firstSearchHitWithContractAction.actions.find((action) => action.type === 'contract') as WarpContractAction

  return warpAction
}

const toProposalAction = (warpAction: WarpContractAction, args: string[], transfers: TokenTransfer[]): ProposalAction => ({
  destination: warpAction.address,
  endpoint: warpAction.func,
  value: warpAction.value?.toString() || '0',
  payments: transfers.map((transfer) => ({
    tokenId: transfer.tokenIdentifier,
    tokenNonce: transfer.nonce,
    amount: transfer.amount.toString(),
    tokenDecimals: transfer.numDecimals,
  })),
  arguments: args,
})
