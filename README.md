# MultiversX AI Hackathon 2025 Submission by Micha Vie

Project: **Autonomous On-chain AI Agent Swarms** – a module within JoAi.ai.

JoAi introduces **Swarms**, groups of AI agents that operate as MultiSigs or Decentralized Autonomous Organizations (DAOs) using the [PeerMe](https://peerme.io) protocol.

Also supports a PeerMe backsync mechanims that allows PeerMe DAOs to hire AI agents by JoAi in the future.

- **What They Are**: Collaborative networks of agents making collective decisions or actions.

- **How They Work**: Swarms leverage PeerMe’s secure MultiSig and DAO features for consensus-driven tasks.

- **Use Case**: A Swarm could manage a community treasury, with agents voting to approve expenditures like “Fund this project with 100 EGLD.”

- **Impact**: Enables decentralized governance and teamwork, extending JoAi’s reach to complex, group-oriented goals.

## Cortex: The core intelligence of JoAi based on Eliza

Cortex powers JoAi agents’ core intelligence, enabling Warp creation and use, collecting user data via natural conversation, and managing task orchestration and delegation for Swarms.

## Client / Frontend

Contains the UI components to interact with cortext (and other services). It is written in React and uses MultiversX SDKs.

## Demo AdMarket Contract written in Swift

A demo smart contract was created using the new MultiversX Swift smart contract framework. The agents use it in the demo to showcase an automatic on-chain proposal to purchase ad space for advertising purposes.

A Warp (`warps/admarket-purchase.json`) is used to interat with the demo smart contract: [View on Devnet Explorer](https://devnet-explorer.multiversx.com/transactions/042af9d31f7882fa6cf0ff75a84a082d6f1676a12268e6bb39ca270b87ae0990)

## Entity v2: A new version of PeerMe Protocol that comes with advanced roles & permission system (in development)

The roles and permission system was improved to enable better management of agents within Swarms and hand out dedicated permissions through roles, permissions and dedicated policies.

## How to use

JoAi is currently in private development and will be released soon. A devnet version is avaiable, however, whitelisting you account is required. Please get in touch for whitelisting, then follow:

1. Create at least 2 agents using the "Create Agent" button
2. Create a Swarm and follow the process (this process is not fully optimized, if you experience issues, please wait a minute and refresh the page)
3. Add your two agents to the swarm
4. Enter the group chat of your swarm and give them a task. State that you would like them to propose it to the Swarm. (Note: we currently have limited Warps active on devnet; not all smart contract interactions will be possible)
5. After the agents proposed and signed, your signature is required in order to execute the action
6. You can always also navigate to PeerMe to manage your Swarm and propose actions to your agents
