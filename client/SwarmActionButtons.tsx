'use client'
import { IconProp } from '@fortawesome/fontawesome-svg-core'
import { faBolt, faFlaskGear, faGlobe, faHexagonNodes, faWallet } from '@fortawesome/pro-solid-svg-icons'
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import { Warp } from '@vleap/warps'
import { clsx } from 'clsx'
import Link from 'next/link'
import { useState } from 'react'
import { Config } from '../../config'
import { useAppDispatch } from '../../store'
import { Theme } from '../../theme'
import { ActionSelector } from '../Action/ActionSelector'
import { setActiveAction } from '../Agent/store/slice'
import { Dialog } from '../Control/Dialog'
import { Tooltip } from '../Feedback/Tooltip'
import { Swarm } from './types'

type Props = {
  swarm: Swarm
  className?: string
}

export function SwarmActionButtons(props: Props) {
  const dispatch = useAppDispatch()
  const [isOpenWallet, setIsOpenWallet] = useState(false)
  const [isOpenActions, setIsOpenActions] = useState(false)
  const entity = props.swarm.entity

  const handleActionSelect = (action: Warp) => {
    dispatch(setActiveAction(action))
    setIsOpenActions(false)
  }

  return (
    <>
      <div className="flex gap-2">
        <div className={clsx('px-4 flex items-center gap-1', Theme.Background.Subtle, Theme.BorderRadius.Subtle, props.className)}>
          <Tooltip
            tip={`${props.swarm.entity?.name} is a multi-agent swarm that can own funds and execute actions together.`}
            className={clsx('font-bold font-head', Theme.TextSize.Base, Theme.TextColor.Intense)}
          >
            <FontAwesomeIcon icon={faHexagonNodes} className="mr-2 opacity-75" />
            {props.swarm.entity?.name}
          </Tooltip>
        </div>
        <div
          className={clsx('p-1 flex gap-1', Theme.Background.Subtle, Theme.BorderRadius.Subtle, props.className)}
          style={{ width: 'fit-content' }}
        >
          <ActionButton icon={faFlaskGear} label="Swarm Settings" href={Config.App.Pages.SwarmsSettings} />
          <ActionButton icon={faWallet} label="Wallet" onClick={() => setIsOpenWallet(true)} />
          {!!entity?.address && (
            <ActionButton icon={faGlobe} label="View on PeerMe" href={`${Config.Urls.PeermeWeb(Config.App.Env)}/${entity.slug}`} />
          )}
          <ActionButton icon={faBolt} label="Action" onClick={() => setIsOpenActions(true)} />
        </div>
      </div>
      <Dialog open={isOpenWallet} onClose={() => setIsOpenWallet(false)} size="2xl">
        {/* <WalletManager agent={props.agent} /> */}
        <span>Coming soon.</span>
      </Dialog>
      <Dialog open={isOpenActions} onClose={() => setIsOpenActions(false)} size="5xl">
        <ActionSelector onSelect={handleActionSelect} />
      </Dialog>
    </>
  )
}

function ActionButton(props: { icon: IconProp; label: string; href?: string; onClick?: () => void }) {
  return props.href ? (
    <Link
      href={props.href}
      className={clsx('px-3 py-2', Theme.Background.SubtleWithHover, Theme.BorderRadius.Subtle)}
      target={props.href.startsWith('http') ? '_blank' : undefined}
    >
      <Tooltip tip={props.label} className="block">
        <div>
          <span className="sr-only">{props.label}</span>
          <FontAwesomeIcon icon={props.icon} className={clsx('text-lg', Theme.TextColor.Moderate)} />
        </div>
      </Tooltip>
    </Link>
  ) : (
    <button onClick={props.onClick} className={clsx('px-3 py-2', Theme.Background.SubtleWithHover, Theme.BorderRadius.Subtle)}>
      <Tooltip tip={props.label} className="block">
        <div>
          <span className="sr-only">{props.label}</span>
          <FontAwesomeIcon icon={props.icon} className={clsx('text-lg', Theme.TextColor.Moderate)} />
        </div>
      </Tooltip>
    </button>
  )
}
