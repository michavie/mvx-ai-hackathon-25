# Autonomous On-chain AI Agent Swarms [MultiversX AI Hackathon 2025]

![MultiversX](https://img.shields.io/badge/MultiversX-Hackathon%202025-blue) ![Status](https://img.shields.io/badge/Status-In%20Development-orange)

**Demo Video**: [Watch on YouTube](https://www.youtube.com/watch?v=03eHJlX_jBA)
**Pitch Deck**: [View on GitHub](https://github.com/michavie/mvx-ai-hackathon-25/blob/main/Pitch%20Deck%20Draft.pdf)
**JoAi Summary**: [View on GitHub](https://github.com/michavie/mvx-ai-hackathon-25/blob/main/About%20JoAi.pdf)

**JoAi** introduces _Swarms_ – decentralized groups of AI agents functioning as MultiSigs or DAOs via the PeerMe protocol on MultiversX. Launch a Swarm to act on your behalf, generating proposals and actionable outcomes for you and your team to approve. Each Swarm has a coordinator that creates a plan, breaks it into tasks, and delegates them to agents.

Powered by the **Warp Protocol**, Swarms can interact with _any_ smart contract or application in the MultiversX ecosystem.

---

## Core Features

- 🤖 **Create Agents**: Each agent has its own MultiversX wallet.
- 🧠 **Train Agents**: Agents use semantic search (RAG) for on-demand knowledge retrieval.
- 👥 **Create Swarms**: Collaborative agent networks making on-chain decisions via MultiSig/DAO.
- ⚡ **Universal On-chain Actions**: Agents and Swarms use Warps to interact with ecosystem smart contracts.
- 🔑 **On-chain Roles & Permissions**: Agents propose actions for approval unless granted specific permissions.
- 🤝 **PeerMe Integration**: Agents evaluate and sign PeerMe proposals in real-time.
- 📊 **Portfolio Overview**: View agent and Swarm-owned assets in the dApp.
  👛 **Full Wallet Abstraction**: Perform any wallet operation (e.g., 'send 1 EGLD to Alice') through natural conversation.

---

## Components

### Cortex: Core Intelligence

Based on Eliza, Cortex drives agent intelligence, enabling Warp creation, natural language data collection, and Swarm task orchestration.

### Client / Frontend

Built with React (Next.js) and MultiversX SDKs, offering an intuitive UI to interact with Cortex and other services.

### Demo Smart Contract: AdMarket

A demo contract written in MultiversX Swift, showcasing an on-chain proposal for ad space purchases.
**Warp Example**: [AdMarket Purchase Warp](https://devnet-explorer.multiversx.com/transactions/042af9d31f7882fa6cf0ff75a84a082d6f1676a12268e6bb39ca270b87ae0990)

### Smart Contract: Entity v2 (In Development)

An upgraded PeerMe Protocol with an advanced roles & permissions system for better agent and Swarm management.

---

## How to Use

JoAi is in private development with a Devnet version available (whitelisting required). Contact [michavie on Telegram](https://t.me/michavie) for access, then follow these steps:

1. Click **"Create Agent"** to generate at least 2 agents.
2. Create a Swarm (note: process optimization in progress; refresh if issues occur).
3. Add your agents to the Swarm.
4. Enter the Swarm’s group chat, assign a task, and request a proposal.
   _Limited Warps are active on Devnet; not all interactions are supported yet._
5. Review and sign the proposal to execute the action.
6. Manage your Swarm and propose actions via PeerMe.

---

## Roadmap

- **Q2**

  - Payments
  - User contacts
  - Warp Core Intelligence
  - Progressive user onboarding

- **Q3**

  - Agent Swarms Full Release
  - Recurring tasks
  - Agent Goals
  - AI Warp Creation
  - Mobile App
  - Product Hunt Launch

- **Q4**
  - Further integrations
  - AI smart contract auditing/recommendations
  - Desktop app and more

---

_Built for the MultiversX AI Hackathon 2025 by [michavie](https://github.com/michavie). Contributions welcome!_
