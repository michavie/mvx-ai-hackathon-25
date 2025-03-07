import { Config } from '../../../config'
import { useAppHttp } from '../../../hooks/useAppHttp'
import { useChainApi } from '../../../hooks/useChainApi'
import { useAppDispatch, useAppSelector } from '../../../store'
import { selectAgents } from '../../Agent/store/selectors'
import { addMessageToQueue } from '../../Agent/store/slice'
import { storeAgentExecuteRequest } from '../../Cortex/api'
import { selectSwarms } from '../../Swarm/store/selectors'
import { selectUser } from '../../User/store/selectors'
import { processAutonomousAction } from '../helpers/autonomous'
import { selectActiveRoom, selectAutonomousRooms } from '../store/selectors'
import { ChatMessage, ChatStreamData, ProcessingContext } from '../types'
import { useRoomStreams } from './useAgentStreams'

export function useAutonomousRoomStreams() {
  const http = useAppHttp()
  const httpPeerme = useAppHttp({ peerme: true })
  const dispatch = useAppDispatch()
  const chain = Config.App.DefaultNetwork() // TODO: support multiple networks
  const chainApi = useChainApi(chain)
  const user = useAppSelector(selectUser)
  const activeRoom = useAppSelector(selectActiveRoom)
  const autonomousRooms = useAppSelector(selectAutonomousRooms)
  const availableAgents = useAppSelector(selectAgents)
  const availableSwarms = useAppSelector(selectSwarms)

  const handleStreamEvent = async (data: ChatStreamData) => {
    if (!user) throw new Error('User not found')
    const agent = availableAgents.find((a) => a.uuid === data.agent)
    if (!agent) throw new Error('Agent not found')

    const handleEmitMessage: ProcessingContext['emitMessage'] = (text, meta, options) => {
      if (!activeRoom) throw new Error('Active room not found')
      const message: ChatMessage = {
        room: activeRoom,
        text,
        meta,
        byUser: false,
        error: options?.error,
        agent: options?.as?.uuid || data.agent,
      }
      const exec = () => dispatch(addMessageToQueue(message))
      if (options?.delay) {
        setTimeout(exec, options.delay)
      } else {
        exec()
      }
    }

    const handleEmitCommand: ProcessingContext['emitCommand'] = (agent, command, options) => {
      if (!activeRoom) throw new Error('Active room not found')
      if (!command.startsWith(Config.App.Chat.CommandPrefix)) throw new Error(`Command must start with "${Config.App.Chat.CommandPrefix}"`)
      const exec = async () => await storeAgentExecuteRequest(http, agent.uuid, agent.uuid, activeRoom, command)
      if (options?.delay) {
        setTimeout(exec, options.delay)
      } else {
        exec()
      }
    }

    const context: ProcessingContext = {
      http,
      httpPeerme,
      chainApi,
      user,
      agent,
      availableAgents,
      availableSwarms,
      emitMessage: handleEmitMessage,
      emitMessageAs: (agent, text, meta, options) => handleEmitMessage(text, meta, { ...options, as: agent }),
      emitCommand: handleEmitCommand,
    }

    if (data.type === 'action' && data.action) {
      await processAutonomousAction(context, data.action, data.text)
    }
  }

  useRoomStreams(
    autonomousRooms.map((auto) => auto.room),
    handleStreamEvent
  )
}
