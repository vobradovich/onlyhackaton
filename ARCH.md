# Nak3dCrypto

Nak3dCrypto is a decentralized content platform — think OnlyFans, but on-chain. Creators publish exclusive content behind token-gated paywalls, and fans unlock it by paying directly in VARA tokens. No intermediaries, no censorship, complete privace. Every transaction goes straight from subscriber to creator via smart contracts on Vara's low-fee, high-speed infrastructure. Built for the creator economy of Web3 — where content ownership and monetization finally belong to the people who make it.

### RequestMesage (+value)
- model_id: ActorId
- content_id: ContentId
- pkey: Vec<u8>

### ReplyMessage
- encoded_content: EncodedContent

### ModelRequest
- content_id: ContentId
- pkey: Vec<u8>

## Program Storage

models: BTreeMap<ActorId, ModelProfile>
buys: BTreeMap<(ActorId, ContentId), EncodedContent>

### ModelProfile
- name
- about
- free_сontent: BTreeMap<ContentId, FreeContent>
- paid_content: BTreeMap<ContentId, PaidContent>

### PaidContent
- id: ContentId
- preview
- price

## Program Methods
- model_create_profile(name: string, about: string, free_сontent: FreeContent[])
- model_add_paid_content(paid_content: PaidContent[])
- model_remove_paid_content(paid_content: ContentId[])

- get_profile(model_id: ActorId) -> ModelProfile;
- get_paid_content(model_id: ActorId, content_id: ContentId, pkey: Vec<u8>) -> EncodedContent;