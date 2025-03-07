'use client'
import { useState } from 'react'
import { Button } from '../../Control/Button'
import { Dialog } from '../../Control/Dialog'
import { Swarm } from '../types'
import { AgentAdder } from './AgentAdder'

type Props = {
  swarm: Swarm
  className?: string
}

export function AgentAdderButton(props: Props) {
  const [isOpen, setIsOpen] = useState(false)

  return (
    <>
      <Button onClick={() => setIsOpen(true)} color="blue" className={props.className}>
        Add Agent
      </Button>
      <Dialog open={isOpen} onClose={() => setIsOpen(false)}>
        <AgentAdder swarm={props.swarm} />
      </Dialog>
    </>
  )
}
