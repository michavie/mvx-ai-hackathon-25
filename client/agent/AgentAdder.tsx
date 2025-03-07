'use client'
import { faAngleDown } from '@fortawesome/pro-solid-svg-icons'
import { FontAwesomeIcon } from '@fortawesome/react-fontawesome'
import { createAction, PermissionConfig } from '@peerme/core-ts'
import { address, string } from '@vleap/warps'
import { clsx } from 'clsx'
import { SyntheticEvent, useMemo, useState } from 'react'
import { useAppDispatch, useAppSelector } from '../../../store'
import { Theme } from '../../../theme'
import { AgentAvatar } from '../../Agent/AgentAvatar'
import { getAgentWallet } from '../../Agent/helpers'
import { selectAgents } from '../../Agent/store/selectors'
import { Agent } from '../../Agent/types'
import { Button } from '../../Control/Button'
import { Dropdown, DropdownButton, DropdownItem, DropdownMenu } from '../../Control/Dropdown'
import { SwarmChain } from '../config'
import { setDirectExecuteRequest } from '../store/slice'
import { EntityDirectExecuteRequest, Swarm } from '../types'

type Props = {
  swarm: Swarm
}

const AgentEntityRoleName = 'Agent'

export function AgentAdder(props: Props) {
  const dispatch = useAppDispatch()
  const allAgents = useAppSelector(selectAgents)
  const [isProcessing, setIsProcessing] = useState(false)

  const availableAgents = useMemo(() => {
    const agentInSwarmIds = props.swarm.agents.map((agent) => agent.uuid)
    return allAgents.filter((agent) => !agentInSwarmIds.includes(agent.uuid))
  }, [allAgents, props.swarm.agents])

  const [selected, setSelected] = useState<Agent | null>(availableAgents[0] || null)

  const handleSubmit = (e: SyntheticEvent) => {
    e.preventDefault()
    if (!props.swarm.entity || !selected) return
    const agentWallet = getAgentWallet(selected, SwarmChain)
    if (!agentWallet) throw new Error('Agent wallet not found')

    const action = createAction(
      props.swarm.entity.address,
      PermissionConfig.ContractFunctions.RoleAssign,
      0n,
      [string(AgentEntityRoleName), address(agentWallet.address)],
      []
    )

    const execRequest: EntityDirectExecuteRequest = {
      title: 'Add Agent',
      entity: props.swarm.entity,
      actions: [action],
      onClose: () => setIsProcessing(false),
      onSent: (txHash) => setIsProcessing(true),
      onExecuted: () => setIsProcessing(false),
    }

    dispatch(setDirectExecuteRequest(execRequest))
  }

  if (availableAgents.length === 0 || !selected) {
    return <p>No agents available.</p>
  }

  return (
    <form onSubmit={handleSubmit}>
      <header className="mb-4">
        <h2 className={clsx(Theme.TextSize.Large)}>Add Agent to {props.swarm.entity?.name}</h2>
        <p className="text-sm text-gray-500">
          {selected.name} will be added to the {props.swarm.entity?.name} group chat and can start to work on tasks.
        </p>
      </header>
      <SwarmAgentSelector available={availableAgents} selected={selected} onSelect={setSelected} />
      <Button type="submit" color="blue" disabled={!selected} className="mt-4 block w-full">
        {selected ? `Add ${selected.name}` : 'Add'}
      </Button>
    </form>
  )
}

function SwarmAgentSelector(props: { available: Agent[]; selected: Agent; onSelect?: (agent: Agent) => void }) {
  return (
    <Dropdown>
      <DropdownButton outline className="w-full border-none opacity-80">
        <AgentAvatar agent={props.selected} className="size-6 sm:size-8" />
        {props.selected?.name}
        <FontAwesomeIcon icon={faAngleDown} className="ml-px opacity-60" />
      </DropdownButton>
      <DropdownMenu>
        {props.available.map((agent) => (
          <DropdownItem key={agent.uuid} onClick={() => props.onSelect?.(agent)}>
            <AgentAvatar agent={agent} className="size-6 sm:size-8" />
            <span className="text-base ml-2">{agent.name}</span>
          </DropdownItem>
        ))}
      </DropdownMenu>
    </Dropdown>
  )
}
