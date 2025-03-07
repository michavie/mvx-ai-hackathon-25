'use client'
import { faHexagonNodes, faSquarePlus } from '@fortawesome/pro-solid-svg-icons'
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import { useGetAccountInfo } from '@multiversx/sdk-dapp/hooks/account/useGetAccountInfo'
import { clsx } from 'clsx'
import { useState } from 'react'
import { useAppDispatch, useAppSelector } from '../../store'
import { Theme } from '../../theme'
import { setActiveAgent } from '../Agent/store/slice'
import { WalletAuthGuardOverlay } from '../Auth/wallet/WalletAuthGuardOverlay'
import { setActiveRoom } from '../Chat/store/slice'
import { Dialog } from '../Control/Dialog'
import { Team } from '../Team/types'
import { selectSwarms } from './store/selectors'
import { setActiveSwarm } from './store/slice'
import { SwarmCreator } from './SwarmCreator'
import { Swarm } from './types'

type Props = {
  team: Team
  className?: string
}

export function SwarmWidget(props: Props) {
  const { address } = useGetAccountInfo()
  const dispatch = useAppDispatch()
  const availableSwarms = useAppSelector(selectSwarms)
  const [isCreating, setIsCreating] = useState(false)

  const buttonClasses = clsx(
    'size-10 flex items-center justify-center',
    Theme.TextColor.ModerateWithHover,
    Theme.Background.ModerateWithHover,
    Theme.BorderRadius.Subtle
  )

  const handleSwitch = (swarm: Swarm) => {
    dispatch(setActiveAgent(null))
    dispatch(setActiveSwarm(swarm))
    dispatch(setActiveRoom(swarm.room))
  }

  return (
    <div className={props.className}>
      <h2 className={clsx('mb-1', Theme.TextSize.Small, Theme.TextColor.Subtle)}>
        Swarms
        <FontAwesomeIcon icon={faHexagonNodes} className="ml-1 opacity-75" />
      </h2>
      <ul className="flex flex-wrap gap-2">
        {availableSwarms.map((swarm) => (
          <li key={swarm.id}>
            <button key={swarm.id} onClick={() => handleSwitch(swarm)} className={buttonClasses}>
              {swarm.entity?.name[0] || '.'}
            </button>
          </li>
        ))}
        <li>
          <button onClick={() => setIsCreating(true)} className={buttonClasses}>
            <span className="sr-only">Create Swarm</span>
            <FontAwesomeIcon icon={faSquarePlus} className="ml-px opacity-75" />
          </button>
        </li>
      </ul>
      <Dialog open={isCreating} onClose={() => setIsCreating(false)}>
        {!address && <WalletAuthGuardOverlay />}
        <SwarmCreator team={props.team} />
      </Dialog>
    </div>
  )
}
