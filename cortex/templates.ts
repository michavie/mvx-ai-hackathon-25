import { getAgentDescription, getAgentName } from '../agent/helpers.js'
import { Swarm } from './types.js'

export const createSwarmInfoTemplate = (swarm: Swarm) => `
=== Swarm Info ===
Situation: You are in a group chat with ${swarm.agents.length} agents with the goal to complete tasks efficiently for the owner.
Every agent, including yourself, is part of a MultiSig wallet on MultiversX.
Swarm Name: ${swarm.name}
Swarm Description: ${swarm.description}
MultiSig Contract address: ${swarm.address}
Agents (Name, Role, Description, UUID):
${swarm.agents.map((agent) => `- ${getAgentName(agent)}, ${agent.role}, ${getAgentDescription(agent) || 'none'}, ${agent.uuid}`).join('\n')}
`
