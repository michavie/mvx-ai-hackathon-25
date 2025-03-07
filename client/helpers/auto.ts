import { getProposalRequest, Proposal } from '@peerme/core-ts'
import { withRetry } from '../../../helpers'
import { signAndSendAgentTransaction } from '../../Agent/helpers'
import { ProcessingContext } from '../../Chat/types'
import { parseEntityProposable, parseEntitySignable } from '../../Cortex/helpers'
import { storeSwarmSignatureFinalizeRequest } from '../api'
import { createEntityProposal, createSignTransaction } from './general'

export const processAutoEntityPropose = async (context: ProcessingContext, content: string) => {
  const proposable = parseEntityProposable(content)
  if (!proposable) throw new Error('Invalid proposable: ' + content)
  const targetAgent = context.availableAgents.find((a) => a.uuid === proposable.agent)
  if (!targetAgent) throw new Error('Target agent not found')
  const targetSwarm = context.availableSwarms.find((s) => s.id === proposable.swarm)
  if (!targetSwarm) throw new Error('Target swarm not found')
  createEntityProposal(context.http, context.chainApi, targetAgent, targetSwarm, proposable, {
    onSuccess: (proposal) => {
      context.emitMessage(`I created a proposal: ${proposal.title}.\n\nPlease everyone review and approve it ✅`, null, {
        delay: 2000,
      })
    },
    onFailed: () => context.emitMessage(`Failed to create proposal`, null, { error: true }),
  })
}

export const processAutoEntitySign = async (context: ProcessingContext, content: string) => {
  const signable = parseEntitySignable(content)
  if (!signable) throw new Error('Invalid signable: ' + content)
  const targetAgent = context.availableAgents.find((a) => a.uuid === signable.agent)
  if (!targetAgent) throw new Error('Target agent not found')
  const targetSwarm = context.availableSwarms.find((s) => s.id === signable.swarm)
  if (!targetSwarm) throw new Error('Target swarm not found')
  if (!signable.sign) {
    context.emitMessageAs(targetAgent, `I am not signing the proposal because: ${signable.reason}`)
    return
  }

  try {
    const proposal = await withRetry(
      async () => {
        const response = await getProposalRequest(context.httpPeerme, signable.proposal)
        return response.data as Proposal
      },
      (proposal) => Boolean(proposal.chainId),
      { maxRetries: 5, retryDelay: targetSwarm.chain.blockTimeMs }
    )
    const signTx = await createSignTransaction(proposal, targetAgent)
    const signTxHash = await signAndSendAgentTransaction(context.chainApi, signTx, targetAgent)
    await storeSwarmSignatureFinalizeRequest(context.http, targetSwarm.id, signTxHash, proposal.id)
    context.emitMessageAs(targetAgent, `✅ I signed the proposal because: ${signable.reason}`)
  } catch (error) {
    const errorMessage = error instanceof Error ? error.message : 'Unknown error occurred'
    context.emitMessageAs(targetAgent, `Failed to get proposal: ${errorMessage}`, { error: true })
  }
}
