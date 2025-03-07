'use client'
import { faExternalLink, faXmarkCircle } from '@fortawesome/pro-solid-svg-icons'
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import { useGetAccountInfo } from '@multiversx/sdk-dapp/hooks/account/useGetAccountInfo'
import { Proposal } from '@peerme/core-ts'
import { clsx } from 'clsx'
import Link from 'next/link'
import { useEffect, useState } from 'react'
import { Config } from '../../../config'
import { usePendingTx } from '../../../hooks/usePendingTx'
import { Theme } from '../../../theme'
import { WalletAuthGuardOverlay } from '../../Auth/wallet/WalletAuthGuardOverlay'
import { Chain } from '../../Chain/types'
import { Button } from '../../Control/Button'
import { Dialog } from '../../Control/Dialog'
import { SwarmContracts } from '../contracts'
import { Swarm } from '../types'

type Props = {
  swarm: Swarm
  proposal: Proposal
  className?: string
}

export function ProposalApprover(props: Props) {
  const { address } = useGetAccountInfo()
  const [isHidden, setIsHidden] = useState(false)
  const [isProcessing, setIsProcessing] = useState(false)
  const [isConnecting, setIsConnecting] = useState(false)

  // hide connection dialog when user connects
  useEffect(() => {
    if (!address) return
    setIsConnecting(false)
  }, [address])

  const pendingTx = usePendingTx(props.proposal.entity.network as Chain, SwarmContracts.EntitySign(props.proposal.entity.address), {
    onSent: () => setIsProcessing(true),
    onSuccess: () => {
      setIsProcessing(false)
      setIsHidden(true)
    },
    onFailed: () => setIsProcessing(false),
  })

  const handleApprove = () => {
    if (!address) {
      setIsConnecting(true)
      return
    }
    pendingTx.call([props.proposal.chainId])
  }

  if (isHidden) {
    return null
  }

  return (
    <div className={clsx(props.className)}>
      <header className="mb-4">
        <h3 className={clsx(Theme.TextSize.Base)}>
          {props.proposal.title}
          <Link href={Config.Urls.PeermeWeb(Config.App.Env) + '/proposals/' + props.proposal.id} target="_blank">
            <span className="sr-only">Open proposal</span>
            <FontAwesomeIcon icon={faExternalLink} className="ml-1 opacity-60" style={{ fontSize: '90%' }} />
          </Link>
        </h3>
      </header>
      <div className="flex gap-2">
        <Button color="green" className="flex-grow" onClick={handleApprove} loading={isProcessing}>
          Approve
        </Button>
        <Button color="red" onClick={() => setIsHidden(true)} plain>
          <span className="sr-only">Reject</span>
          <FontAwesomeIcon icon={faXmarkCircle} className="opacity-90" />
        </Button>
      </div>
      <Dialog open={isConnecting} onClose={() => setIsConnecting(false)}>
        <WalletAuthGuardOverlay subject="approve" />
        <div className="h-48" />
      </Dialog>
    </div>
  )
}
