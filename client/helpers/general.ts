import {
  AbiRegistry,
  Address,
  ApiNetworkProvider,
  SmartContractTransactionsFactory,
  TokenTransfer,
  TransactionsFactoryConfig,
  U64Value,
} from '@multiversx/sdk-core/out'
import { ContractsConfig, IHttpService, Proposal } from '@peerme/core-ts'
import { handleAppResponse } from '../../../http'
import { getAgentWallet, signAndSendAgentTransaction } from '../../Agent/helpers'
import { Agent } from '../../Agent/types'
import { EntityProposable } from '../../Cortex/types'
import { storeProposalFinalizeRequest, storeSwarmProposalRequest } from '../api'
import { Swarm } from '../types'

export const createEntityProposal = async (
  http: IHttpService,
  chainApi: ApiNetworkProvider,
  agent: Agent,
  swarm: Swarm,
  proposable: EntityProposable,
  callbacks?: {
    onSuccess?: (proposal: Proposal) => void
    onFailed?: () => void
  }
) => {
  handleAppResponse(
    storeSwarmProposalRequest(http, agent.uuid, swarm.id, proposable.title, proposable.description, proposable.actions, {}, []),
    async (proposal, meta) => {
      const weightPayments: TokenTransfer[] = []
      const tx = await createProposeTransaction(proposal, weightPayments, meta, 0)
      const txHash = await signAndSendAgentTransaction(chainApi, tx, agent)
      handleAppResponse(
        storeProposalFinalizeRequest(http, swarm.id, txHash, proposal.id),
        () => callbacks?.onSuccess?.(proposal),
        () => callbacks?.onFailed?.()
      )
    },
    () => callbacks?.onFailed?.()
  )
}

const createProposeTransaction = async (
  proposal: Proposal,
  weightPayments: TokenTransfer[],
  meta: { contentSignature: string },
  pollOptionId: number
) => {
  const scInfo = ContractsConfig({} as any).EntityPropose(proposal.entity.address)
  const config = new TransactionsFactoryConfig({ chainID: proposal.entity.network.chainId })
  const abiRes = await fetch(scInfo.AbiUrl!)
  const abiContents = await abiRes.json()
  const abi = AbiRegistry.create(abiContents)
  const factory = new SmartContractTransactionsFactory({ config, abi })
  const args = [
    Buffer.from(proposal.id, 'ascii'),
    Buffer.from(proposal.contentHash, 'hex'),
    Buffer.from(meta.contentSignature || '', 'hex'),
    proposal.actionsHash ? Buffer.from(proposal.actionsHash, 'hex') : '',
    pollOptionId,
    ...proposal.permissions,
  ]

  return factory.createTransactionForExecute({
    sender: new Address(proposal.proposer.address),
    contract: new Address(proposal.entity.address),
    function: scInfo.Endpoint,
    gasLimit: BigInt(scInfo.GasLimit || 20_000_000 * proposal.actions.length),
    arguments: args,
    tokenTransfers: weightPayments,
  })
}

export const createSignTransaction = async (proposal: Proposal, agent: Agent) => {
  const scInfo = ContractsConfig({} as any).EntitySign(proposal.entity.address)
  const config = new TransactionsFactoryConfig({ chainID: proposal.entity.network.chainId })
  const factory = new SmartContractTransactionsFactory({ config })
  return factory.createTransactionForExecute({
    sender: new Address(getAgentWallet(agent, 'multiversx')!.address),
    contract: new Address(proposal.entity.address),
    function: scInfo.Endpoint,
    gasLimit: BigInt(scInfo.GasLimit || 20_000_000),
    arguments: [new U64Value(proposal.chainId)],
  })
}
