'use client'
import { EntityTemplate } from '@peerme/core-ts'
import { useRouter } from 'next/navigation'
import { SyntheticEvent, useState } from 'react'
import { celebrate } from '../../celebrations'
import { Config } from '../../config'
import { denominateEgld } from '../../helpers-chain'
import { useAppHttp } from '../../hooks/useAppHttp'
import { usePendingTx } from '../../hooks/usePendingTx'
import { handleAppResponse } from '../../http'
import { useAppDispatch, useAppSelector } from '../../store'
import { setActiveRoom } from '../Chat/store/slice'
import { Button } from '../Control/Button'
import { Description, Field, FieldGroup, Fieldset, Label } from '../Control/Fieldset'
import { Input } from '../Control/Input'
import { Textarea } from '../Control/Textarea'
import { Steps } from '../Progress/Steps'
import { Team } from '../Team/types'
import { storeSwarmRequest } from './api'
import { SwarmContracts } from './contracts'
import { selectSwarms } from './store/selectors'
import { setActiveSwarm, setSwarms } from './store/slice'

type Props = {
  team: Team
}

export function SwarmCreator(props: Props) {
  const http = useAppHttp()
  const router = useRouter()
  const dispatch = useAppDispatch()
  const availableSwarms = useAppSelector(selectSwarms)
  const [name, setName] = useState('')
  const [description, setDescription] = useState('')
  const [isProcessing, setIsProcessing] = useState(false)
  const network = Config.App.DefaultNetwork()
  const isPrivate = false // TODO: make configurable
  const template: EntityTemplate = 'next'

  const pendingTx = usePendingTx(network, SwarmContracts.ManagerEntityCreate, {
    onSent: ({ txHash }) => {
      setIsProcessing(true)
      handleAppResponse(storeSwarmRequest(http, props.team, network.name, txHash, name, description, isPrivate, template), (swarm) =>
        setTimeout(() => {
          dispatch(setActiveSwarm(swarm))
          dispatch(setActiveRoom(swarm.room))
          dispatch(setSwarms(availableSwarms.concat(swarm)))
          celebrate()
          setIsProcessing(false)
          router.push(Config.App.Pages.SwarmsSettings)
        }, 10_000)
      )
    },
    onSuccess: () => {
      setIsProcessing(false)
    },
    onFailed: () => {
      setIsProcessing(false)
    },
  })

  const handleSubmit = (e: SyntheticEvent) => {
    e.preventDefault()
    pendingTx.callWithNative(denominateEgld(0.1), [])
  }

  return (
    <form onSubmit={handleSubmit}>
      <header className="mb-4">
        <div className="flex justify-center mb-2">
          <Steps total={3} active={1} />
        </div>
        <h1 className="text-lg sm:text-xl mb-1">Create a Swarm</h1>
        <p className="text-sm sm:text-base text-gray-500">
          Swarms are groups of agents that work together to achieve a goal. They can own assets collectively and perform actions on your
          behalf.
        </p>
      </header>
      <Fieldset>
        <FieldGroup>
          <Field>
            <Label>Name</Label>
            <Input
              name="name"
              placeholder="My Swarm"
              value={name}
              onChange={setName}
              maxLength={64}
              required
              autoFocus
              autoComplete="off"
            />
          </Field>
          <Field className="mb-4">
            <Label>Purpose</Label>
            <Description>Describe the purpose of the Swarm.</Description>
            <Textarea name="description" placeholder="Example: ..." value={description} onChange={setDescription} rows={4} required />
          </Field>
        </FieldGroup>
      </Fieldset>
      <Button color="blue" type="submit" className="mt-4 block w-full" loading={isProcessing}>
        Create now
      </Button>
    </form>
  )
}
