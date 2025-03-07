'use client'
import { clsx } from 'clsx'
import { useMemo } from 'react'
import { useAppDispatch, useAppSelector } from '../../store'
import { Theme } from '../../theme'
import { setActiveAgent } from '../Agent/store/slice'
import { Agent } from '../Agent/types'
import { selectSwarms } from './store/selectors'
import { setActiveSwarm } from './store/slice'
import { Swarm } from './types'

type Props = {
  agent?: Agent
  className?: string
}

export function SwarmOverview(props: Props) {
  const dispatch = useAppDispatch()
  const availableSwarms = useAppSelector(selectSwarms)

  const swarms = useMemo(
    () => (props.agent ? availableSwarms.filter((s) => s.agents.some((a) => a.uuid === props.agent?.uuid)) : availableSwarms),
    [props.agent, availableSwarms]
  )

  const handleClick = (swarm: Swarm) => {
    dispatch(setActiveAgent(null))
    dispatch(setActiveSwarm(swarm))
  }

  return (
    <ul className={clsx(props.className)}>
      {swarms.map((swarm) => (
        <li key={swarm.id}>
          <button
            onClick={() => handleClick(swarm)}
            type="button"
            className={clsx('px-4 py-2 block w-full text-left', Theme.Background.ModerateWithHover, Theme.BorderRadius.Subtle)}
          >
            <span className={clsx(Theme.TextSize.Base, Theme.TextColor.Intense)}>{swarm.entity?.name}</span>
          </button>
        </li>
      ))}
    </ul>
  )
}
