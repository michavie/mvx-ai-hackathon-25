import { SharedCache } from '../../../capabilities/cache.js'
import { CacheKey } from '../../../capabilities/constants.js'
import { ActionPlan, ActionPlanStep } from '../types.js'

export const getActionPlanForRoom = async (roomId: string): Promise<ActionPlan | null> => {
  const actionPlan = (await SharedCache.get(CacheKey.SwarmActionPlan(roomId))) as ActionPlan | undefined
  return actionPlan || null
}

export const hasActionPlanForRoom = async (roomId: string): Promise<boolean> => {
  const actionPlan = await getActionPlanForRoom(roomId)
  return !!actionPlan
}

export const setActionPlanForRoom = async (roomId: string, plan: ActionPlan) => {
  await SharedCache.set(CacheKey.SwarmActionPlan(roomId), plan)
}

export const pullNextStepFromActionPlan = async (roomId: string): Promise<[ActionPlanStep | null, ActionPlan | null]> => {
  const plan = await getActionPlanForRoom(roomId)
  if (!plan) return [null, null]
  const hasSteps = plan.steps.length > 0
  if (!hasSteps) return [null, plan]
  const step = plan.steps.find((step) => !step.done)
  if (!step) return [null, plan]
  const updatedPlan = { ...plan, steps: plan.steps.filter((s) => s.id !== step.id) }
  await markStepAsDoneInActionPlan(roomId, step.id)
  return [step, updatedPlan]
}

export const markStepAsDoneInActionPlan = async (roomId: string, stepId: string) => {
  const plan = await getActionPlanForRoom(roomId)
  if (!plan) return
  const updatedPlan = { ...plan, steps: plan.steps.map((s) => (s.id === stepId ? { ...s, done: true } : s)) }
  await setActionPlanForRoom(roomId, updatedPlan)
}

export const clearActionPlanForRoom = async (roomId: string): Promise<void> => {
  await SharedCache.delete(CacheKey.SwarmActionPlan(roomId))
}
