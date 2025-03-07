import { EntityTemplate, IHttpService, MediaItems, Proposal, ProposalAction, ProposalPollOption } from '@peerme/core-ts'
import { Agent } from '../Agent/types'
import { ChainName } from '../Chain/types'
import { Team } from '../Team/types'
import { Swarm } from './types'

export const getSwarmsRequest = async (http: IHttpService, team: Team) => await http.get<Swarm[]>(`swarms?team=${team.slug}`)

export const storeSwarmRequest = async (
  http: IHttpService,
  team: Team,
  chain: ChainName,
  txHash: string,
  name: string,
  description: string,
  isPrivate: boolean,
  template?: EntityTemplate
) => await http.post<Swarm>('swarms', { team: team.slug, tx: txHash, chain, name, description, private: isPrivate, template })

export const getSwarmAgentsRequest = async (http: IHttpService, swarm: string) => await http.get<Agent[]>(`swarms/${swarm}/agents`)

export const storeSwarmProposalRequest = async (
  http: IHttpService,
  agent: string,
  swarm: string,
  title: string,
  description: string,
  actions: ProposalAction[],
  attachments: MediaItems,
  poll: ProposalPollOption[]
) => await http.post<Proposal>(`swarms/${swarm}/entities/proposals`, { agent, title, description, actions, attachments, poll })

export const storeProposalFinalizeRequest = async (http: IHttpService, swarm: string, txHash: string, proposal: string) =>
  await http.post(`swarms/${swarm}/entities/finalize-proposal`, { tx: txHash, proposal })

export const storeSwarmSignatureFinalizeRequest = async (http: IHttpService, swarm: string, txHash: string, proposal: string) =>
  await http.post(`swarms/${swarm}/entities/finalize-signature`, { tx: txHash, proposal })

export const storeSwarmExecuteFinalizeRequest = async (http: IHttpService, chain: ChainName, txHash: string, actions?: ProposalAction[]) =>
  await http.post('swarms/finalize-exec', { chain, tx: txHash, actions: actions || [] })
