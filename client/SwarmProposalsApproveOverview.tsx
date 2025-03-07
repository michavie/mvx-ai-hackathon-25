'use client'
import { getProposalsRequest, Proposal } from '@peerme/core-ts'
import { clsx } from 'clsx'
import { useEffect, useState } from 'react'
import { executeMulti } from '../../helpers'
import { useAppHttp } from '../../hooks/useAppHttp'
import { handleAppResponse } from '../../http'
import { useAppSelector } from '../../store'
import { Theme } from '../../theme'
import { selectChatLatestAction } from '../Chat/store/selectors'
import { ProposalApprover } from './entity/ProposalApprover'
import { toSwarmEntityId } from './helpers/auto'
import { Swarm } from './types'

type Props = {
  swarm: Swarm
  className?: string
}

export function SwarmProposalsApproveOverview(props: Props) {
  const http = useAppHttp({ peerme: true })
  const latestChatAction = useAppSelector(selectChatLatestAction)
  const [nextProposal, setNextProposal] = useState<Proposal | null>(null)
  const [isLoading, setIsLoading] = useState(true)

  // initial load
  useEffect(() => {
    if (!props.swarm.entity) return
    loadProposals()
  }, [props.swarm.entity])

  // load triggered by chat actions
  useEffect(() => {
    if (latestChatAction !== 'ENTITY_PROPOSE') return
    executeMulti(loadProposals, 3, props.swarm.chain.blockTimeMs, props.swarm.chain.blockTimeMs)
  }, [latestChatAction])

  const loadProposals = async () => {
    handleAppResponse(getProposalsRequest(http, toSwarmEntityId(props.swarm)), (proposals) => {
      const active = proposals.filter((p) => p.status === 'active')
      setNextProposal(active[0] || null)
      setIsLoading(false)
    })
  }

  if (!props.swarm.entity) {
    return null
  }

  if (isLoading) {
    return <p className={clsx(Theme.TextSize.Small)}>Loading...</p>
  }

  if (!nextProposal) {
    return <p className={clsx(Theme.TextSize.Small)}>No proposals waiting for approval.</p>
  }

  return (
    <div className={props.className}>
      <h3 className="text-base font-bold mb-2 opacity-75">Awaiting approval</h3>
      <ProposalApprover swarm={props.swarm} proposal={nextProposal} />
    </div>
  )
}
