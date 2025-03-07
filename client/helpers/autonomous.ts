import { processAutoAgentDelegation } from '../../Agent/helpers/auto'
import { ChatClientAction } from '../../Cortex/types'
import { processAutoEntityPropose, processAutoEntitySign } from '../../Swarm/helpers/auto'
import { processAutoWarpExecute } from '../../Warp/helpers/auto'
import { ProcessingContext } from '../types'

export const processAutonomousAction = async (context: ProcessingContext, action: ChatClientAction, content: string) => {
  try {
    if (action === 'AGENT_DELEGATE') return await processAutoAgentDelegation(context, content)
    if (action === 'WARP_EXECUTE') return await processAutoWarpExecute(context, content)
    if (action === 'ENTITY_PROPOSE') return await processAutoEntityPropose(context, content)
    if (action === 'ENTITY_SIGN') return await processAutoEntitySign(context, content)
    throw new Error('Unhandled action: ' + action)
  } catch (error) {
    context.emitMessage(`I encountered an error: ${error}`, null, { error: true, delay: 1000 })
  }
}
