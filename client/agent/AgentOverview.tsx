'use client'
import { faCopy } from '@fortawesome/pro-solid-svg-icons'
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import { clsx } from 'clsx'
import Image from 'next/image'
import { useEffect, useState } from 'react'
import { useClipboard } from 'use-clipboard-copy'
import { trimHash } from '../../../helpers'
import { useAppHttp } from '../../../hooks/useAppHttp'
import { handleAppResponse } from '../../../http'
import { Theme } from '../../../theme'
import { AgentAvatar } from '../../Agent/AgentAvatar'
import { getAgentDescription, getAgentName } from '../../Agent/helpers'
import { Agent } from '../../Agent/types'
import { getChainLogo } from '../../Chain/helpers'
import { showToast } from '../../Feedback/Toast'
import { Tooltip } from '../../Feedback/Tooltip'
import { capitalize } from '../../helpers'
import { getSwarmAgentsRequest } from '../api'
import { Swarm } from '../types'
import { AgentAdderButton } from './AgentAdderButton'

type Props = {
  swarm: Swarm
}

export function AgentOverview(props: Props) {
  const http = useAppHttp()
  const [agents, setAgents] = useState<Agent[]>([])

  useEffect(() => {
    if (!props.swarm.entity) return
    handleAppResponse(getSwarmAgentsRequest(http, props.swarm.id), (data) => setAgents(data))
  }, [])

  return (
    <div>
      <header className="mb-4">
        <AgentAdderButton swarm={props.swarm} />
      </header>
      <ul className="space-y-2">
        {agents.map((agent) => (
          <li key={agent.uuid} className="w-full sm:w-auto">
            <AgentCard agent={agent} />
          </li>
        ))}
      </ul>
    </div>
  )
}

function AgentCard({ agent }: { agent: Agent }) {
  const clipboard = useClipboard({
    onSuccess: () => showToast('Copied', 'info'),
  })

  return (
    <div
      className={clsx(
        'px-4 sm:px-6 py-4 flex gap-4 sm:gap-6 items-center max max-w-2xl',
        Theme.Background.Subtle,
        Theme.BorderRadius.BrandedSubtle
      )}
    >
      <AgentAvatar agent={agent} className="size-8 sm:size-12" />
      <div>
        <h3 className={clsx(Theme.TextSize.Base, Theme.TextColor.Intense)}>{getAgentName(agent)}</h3>
        <p className={clsx(Theme.TextSize.Small, Theme.TextColor.Subtle)}>{capitalize(agent.role ?? 'unknown')}</p>
        <p className={clsx(Theme.TextSize.Small, Theme.TextColor.Subtle)}>{getAgentDescription(agent)}</p>
        <ul className="mt-2 space-y-2 px-2">
          {agent.wallets.map((wallet) => (
            <li key={wallet.chain} className="flex items-center gap-2">
              <Tooltip tip={capitalize(wallet.chain)}>
                <Image src={getChainLogo(wallet.chain)} alt={wallet.chain} width={25} height={25} className="size-6" />
              </Tooltip>
              <span className={clsx(Theme.TextSize.Small, Theme.TextColor.Subtle)}>
                {trimHash(wallet.address, 8)}
                <button onClick={() => clipboard.copy(wallet.address)} className="ml-1">
                  <span className="sr-only">Copy</span>
                  <FontAwesomeIcon icon={faCopy} className="opacity-80" />
                </button>
              </span>
            </li>
          ))}
        </ul>
      </div>
    </div>
  )
}
