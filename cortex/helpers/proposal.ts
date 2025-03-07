import { Proposal } from '@peerme/core-ts'
import { SharedCache } from '../../../capabilities/cache.js'
import { CacheKey } from '../../../capabilities/constants.js'

export const getActiveProposalForRoom = async (roomId: string): Promise<Proposal | null> => {
  const proposal = (await SharedCache.get(CacheKey.SwarmProposal(roomId))) as Proposal | undefined
  return proposal || null
}

export const hasActiveProposalForRoom = async (roomId: string): Promise<boolean> => {
  const proposal = await getActiveProposalForRoom(roomId)
  return !!proposal
}

export const setActiveProposalForRoom = async (roomId: string, proposal: Proposal) => {
  await SharedCache.set(CacheKey.SwarmProposal(roomId), proposal)
}

export const clearActiveProposalForRoom = async (roomId: string): Promise<void> => {
  await SharedCache.delete(CacheKey.SwarmProposal(roomId))
}
