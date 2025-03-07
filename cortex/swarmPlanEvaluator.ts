import { Evaluator } from '@elizaos/core'
import { sendAgentMessage } from '../../helpers-eliza.js'
import { getCurrentAgentInSwarm, sendAgentDelegate } from './helpers/general.js'
import { clearActionPlanForRoom, hasActionPlanForRoom, pullNextStepFromActionPlan } from './helpers/plan.js'
import { AgentDelegatable } from './types.js'

export const swarmPlanEvaluator: Evaluator = {
  name: 'SWARM_PLAN',
  similes: [],
  description: 'Should always run when agent is conversing within a swarm.',
  validate: async (runtime, message) => await hasActionPlanForRoom(message.roomId),
  handler: async (runtime, message, state, options, callback) => {
    const [currentAgent, swarm] = await getCurrentAgentInSwarm(runtime, message.roomId)
    if (!currentAgent || !swarm) return

    const [nextStep, actionPlan] = await pullNextStepFromActionPlan(message.roomId)

    if (!actionPlan) {
      return
    }

    if (nextStep) {
      const delegatable: AgentDelegatable = {
        agent: nextStep.agent,
        swarm: swarm.id,
        room: swarm.room,
        prompt: nextStep.action,
      }
      sendAgentDelegate(callback, delegatable, { delay: 1000 })
    } else {
      await clearActionPlanForRoom(message.roomId)
      sendAgentMessage(callback, 'All actions completed ✅')
    }
  },
  examples: [],
}
