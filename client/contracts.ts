import { ContractsConfig } from '@peerme/core-ts'
import { Config } from '../../config'
import { AppEnv } from '../../types'

export const getManagerContractAddress = (env: AppEnv): string => {
  if (env === 'local') return 'erd1qqqqqqqqqqqqqpgqxtccplxgycv25kjfude7mgwdm7znp9u4l3ts46euux'
  if (env === 'devnet') return 'erd1qqqqqqqqqqqqqpgqxtccplxgycv25kjfude7mgwdm7znp9u4l3ts46euux'
  if (env === 'testnet') return 'erd1qqqqqqqqqqqqqpgql8k3x9tvnp9w5jc802eh6793asvlhu7pl3tsz5tzk3'
  return 'erd1qqqqqqqqqqqqqpgqtatmxjhlxkehl37u5kz9tz7sm450xd7f27rsppynzj'
}

export const SwarmContracts = ContractsConfig({
  Organization: '',
  Identity: '',
  Manager: getManagerContractAddress(Config.App.Env),
  Earn: '',
  Payout: '',
  Bounty: '',
  ContractStage: '',
  DexRouter: '',
})
