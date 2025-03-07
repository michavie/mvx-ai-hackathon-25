import { Action, generateObject, ModelClass, stringToUuid } from '@elizaos/core'
import { z } from 'zod'
import { CreditUsageStore } from '../../credit-usage-store.js'
import { sendAgentMessage } from '../../helpers-eliza.js'
import { getCurrentAgentInSwarm, isCurrentAgentManagerInRoom } from './helpers/general.js'
import { getActionPlanForRoom, setActionPlanForRoom } from './helpers/plan.js'
import { createSwarmInfoTemplate } from './templates.js'
import { ActionPlan } from './types.js'

export const swarmPlanAction: Action = {
  name: 'SWARM_PLAN',
  similes: [],
  description: 'Create a step-by-step action plan for the swarm agents.',
  validate: async (runtime, message, state): Promise<boolean> => await isCurrentAgentManagerInRoom(runtime, message.roomId),
  handler: async (runtime, message, state, options, callback): Promise<unknown> => {
    const [currentAgent, swarm] = await getCurrentAgentInSwarm(runtime, message.roomId)
    if (!currentAgent || !swarm) return

    const existingPlan = await getActionPlanForRoom(message.roomId)
    if (existingPlan) {
      console.log('SwarmPlanAction: Skipping because action plan already exists')
      return false
    }

    let planTemplate = `Your job is to break down the user's request into minimal actionable steps that can be executed by the agents. `
    planTemplate += `Proposals, if any, should be done by the manager (ID: ${currentAgent.uuid}).\n`
    planTemplate += createSwarmInfoTemplate(swarm) + '\n'
    planTemplate += `User prompt: ${message.content.text}\n`
    planTemplate += 'Create a plan by returning a JSON object containing: '
    planTemplate += '{"steps": [{"action": "description of the action", "agent": "uuid of the agent to execute the action"}]}'

    const schema = z.object({
      steps: z.array(
        z.object({
          action: z.string(),
          agent: z.string(),
        })
      ),
    })

    const planResult = await generateObject({ runtime, context: planTemplate, modelClass: ModelClass.SMALL, schema })
    CreditUsageStore.getInstance().increaseCreditUsage(runtime.agentId, planResult.usage.totalTokens)
    const actionPlan = planResult.object as ActionPlan
    actionPlan.prompt = message.content.text
    actionPlan.steps.forEach((step) => {
      step.id = stringToUuid(step.action + '-' + step.agent)
    })

    if (actionPlan.steps.length > 0) {
      await setActionPlanForRoom(message.roomId, actionPlan)

      let response = `Created action plan with ${actionPlan.steps.length} steps 📋 \n`
      response += `${actionPlan.steps.map((step) => `- ${step.action}`).join('\n')}`
      sendAgentMessage(callback, response)
    } else {
      console.log('INVALID ACTION PLAN', actionPlan)
      sendAgentMessage(callback, 'Failed to create action plan ❌')
    }
  },
  examples: [],
}
