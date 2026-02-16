# Onlyhack Architecture

This document reflects the current implementation in `app/src/lib.rs` and client/test usage.

## Overview

Onlyhack is an on-chain creator content service:
- Models create a profile with open content previews.
- Models add paid content as encrypted `PublicData` plus price.
- Fans buy a specific paid item by sending exact value and a public key.
- The contract asks the model for a `GrantWithProof`, verifies it, stores buyer-specific encrypted grant, and pays out model revenue minus protocol fee.

## Core Types

- `ContentId = u64`
- `PaidContentTuple = (String, String, u128)`
  - `(preview, hex(public_data SCALE bytes), price)`
- `ProfileTuple = (String, String, Vec<(ContentId, String)>, Vec<(ContentId, (String, String, u128))>)`
- `ProfileInfo = (name, about, open_content, purchased_content, hidden_content)`
  - `open_content: Vec<(ContentId, String)>`
  - `purchased_content: Vec<(ContentId, String, u128, Vec<u8>, Vec<u8>)>`
  - `hidden_content: Vec<(ContentId, String, u128)>`

## Storage

`State`:
- `models: BTreeMap<ActorId, ModelProfile>`
- `buys: BTreeMap<ActorId, BTreeMap<(ActorId, ContentId), ElGamalPointCipher>>`
  - Outer key: buyer actor id.
  - Inner key: `(model_id, content_id)`.
  - Value: encrypted point-cipher grant (`enc_m_under_pk`).
- `content_next_id: ContentId`
  - Global monotonically increasing id for both open and paid content.

`ModelProfile`:
- `name: String`
- `about: String`
- `open_content: BTreeMap<ContentId, String>`
- `paid_content: BTreeMap<ContentId, PaidContent>`

`PaidContent`:
- `preview: String`
- `public_data: dleq_secret::PublicData`
- `price: u128`

## Exported Service API

### `model_create_profile(name, about, open_content)`
- Caller becomes the model id (`message_source`).
- Replaces any existing profile for that model.
- Assigns new global `ContentId` values to each open content entry.

### `model_add_paid_content((preview, data_hex, price)) -> ContentId`
- Requires caller already has a profile.
- Requires `price > 0`.
- Decodes `data_hex` into `PublicData` (SCALE bytes encoded as hex string).
- Stores paid content under a new global `ContentId`.

### `get_profile(model_id) -> ProfileTuple`
- Returns model name/about/open content and paid content metadata.
- Paid content returns `(preview, hex(public_data.encode()), price)`.

### `buy_content(model_id, content_id, pk_bytes_hex) -> CommandReply<Vec<u8>>` (payable, async)
- Requires transferred value `> 0` and exactly equal to `paid_content.price`.
- Decodes buyer PK from hex.
- If buyer already purchased `(model_id, content_id)`, returns cached encrypted point and refunds transferred value.
- Otherwise:
  - Sends PK to `model_id` via `send_for_reply`.
  - Expects reply payload decodable as `GrantWithProof`.
  - Verifies proof against stored `public_data` and PK.
  - Stores purchase in `buys`.
  - Splits payment:
    - `fee = transferred / 50` (2%)
    - `model_payout = transferred - fee` (98%)
  - Attempts to send payout to model; if send fails, value remains in contract balance.
- Return payload is SCALE-encoded `Result<ElGamalPointCipher, String>` wrapped in `CommandReply<Vec<u8>>`.

### `get_enc_content(model_id, content_id) -> Vec<u8>`
- Returns `paid_content.public_data.enc_x` bytes.

### `get_profiles() -> Vec<ProfileInfo>`
- Personalizes output for caller:
  - `purchased_content`: only items caller has bought from each model, including encrypted payloads.
  - `hidden_content`: paid items caller has not bought (preview + price only).
- Always includes open content and profile metadata.

## Program Entrypoints

`Program::create()` initializes global `STATE`.

`Program::onlyhack()` returns service instance.

## Not Present In Current Code

These are not implemented in the current contract:
- `model_remove_paid_content`
- `get_paid_content`
- `get_all_purchased_content` (commented out stub exists)
