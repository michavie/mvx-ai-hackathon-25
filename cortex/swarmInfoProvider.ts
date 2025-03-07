import { Provider } from '@elizaos/core'
import { getSwarmFromRoom } from './helpers/general.js'
import { createSwarmInfoTemplate } from './templates.js'

export const swarmInfoProvider: Provider = {
  get: async (runtime, message) => {
    const swarm = await getSwarmFromRoom(message.roomId)

    if (!swarm) {
      return ''
    }

    return createSwarmInfoTemplate(swarm)
  },
}
