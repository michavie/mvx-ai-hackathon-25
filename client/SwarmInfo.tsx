'use client'
import { faDiamond, faPlay } from '@fortawesome/pro-solid-svg-icons'
import { TabGroup, TabPanel, TabPanels } from '@headlessui/react'
import { clsx } from 'clsx'
import { useAppSelector } from '../../store'
import { Theme } from '../../theme'
import { TabBar } from '../Tabs/TabBar'
import { PortfolioOverview } from '../Wallet/multiversx/PortfolioOverview'
import { selectActiveSwarm } from './store/selectors'
import { SwarmProposalsApproveOverview } from './SwarmProposalsApproveOverview'

type Props = {}

export function SwarmInfo(props: Props) {
  const activeSwarm = useAppSelector(selectActiveSwarm)

  if (!activeSwarm) {
    return null
  }

  return (
    <div className={clsx('w-64 p-4', Theme.Background.Subtle, Theme.BorderRadius.BrandedSubtle)}>
      <TabGroup>
        <TabBar
          items={[
            { icon: faPlay, label: 'Actions' },
            { icon: faDiamond, label: 'Assets' },
          ]}
          className="mb-4"
        />
        <TabPanels>
          <TabPanel>
            <SwarmProposalsApproveOverview swarm={activeSwarm} />
          </TabPanel>
          <TabPanel>
            {!!activeSwarm.entity && <PortfolioOverview address={activeSwarm.entity.address} owner={activeSwarm.entity.name} />}
          </TabPanel>
        </TabPanels>
      </TabGroup>
    </div>
  )
}
