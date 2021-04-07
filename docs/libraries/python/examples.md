# Examples

> Please note: It is not recommended to store passwords/seeds on host's environment variables or in the source code in a production setup! Please make sure you follow our [backup and security](https://chrysalis.docs.iota.org/guides/backup_security.html) recommendations for production use!

## Connecting to node(s)
All features of `iota.rs` library are accessible via an instance of `Client` class that provides high-level abstraction to all interactions over IOTA network (Tangle). This class has to be instantiated before starting any interactions with the library, or more precisely with [IOTA nodes](https://chrysalis.docs.iota.org/node-software/node-software.html) that power IOTA network.

You may be familiar with a fact that in case of IOTA 1.0 network one had to know an address of IOTA node to start participating to the network. It is no longer needed in IOTA 1.5 (Chrysalis) world. The library is designed to automatically choose a starting IOTA node based on the network type one would like to participate in: `testnet` or `mainnet`.

So very simplistic example how to connect to [IOTA testnet](https://chrysalis.docs.iota.org/testnet.html) is the following one:

```python
{{#include ../../../bindings/python/examples/01_get_info.py}}
```

Output example of `get_info()` function of the `Client` instance:
```json
{
    'name': 'HORNET',
    'version': '0.6.0-alpha',
    'is_healthy': True,
    'network_id': 'testnet6',
    'bech32_hrp': 'atoi',
    'latest_milestone_index': 192448,
    'confirmed_milestone_index': 192448,
    'pruning_index': 174931,
    'features': ['PoW'],
    'min_pow_score': 4000.0
}
```
The most important properties:
* `is_healthy`: indicates whether the given node is in sync with the network and so it is safe to use it. Even if a node is up and running it may not be fully prepared to process your API calls properly. The node should be "synced", meaning should be aware of all TXs in the Tangle. It is better to avoid not fully synced nodes. A node healthiness can be alternatively obtained also with a method `Client.get_health()`
* `bech32_hrp`: it indicates whether the given node is a part of testnet (`atoi`) or mainnet (`iota`). See more info regarding [IOTA address format](../../welcome.md#iota-15-address-anatomy)

_Please note, when using node load balancers then mentioned health check may be quite useless since follow-up API calls may be served by different node behind the load balancer that may have not been actually checked. One should be aware of this fact and trust the given load balancer participates only with nodes that are in healthy state. `iota.rs` library additionally supports a management of internal node pool and so load-balancer-like behavior can be mimicked using this feature locally._

Needless to say, the `Client` class constructor provides several parameters via which the process can be closely managed.

The most common ones:
* `network`: can be `Testnet` or `Mainnet`. It instructs the library whether to automatically select testnet nodes or mainnet nodes
* `node`: specify address of actual running IOTA node that should be used to communicate with (in format `https://node:port`), for ex: https://api.lb-0.testnet.chrysalis2.com:443
* `node_pool_urls`: library also supports a management of pool of nodes. You can provide a list of nodes and library manages access to them automatically (selecting them based on their sync status)
* `local_pow`: `local_pow==True` (by default) means a `Proof-of-work` is done locally and not remotely
* `node_sync_disabled`: `node_sync_disabled==False` (by default) means that library checks for sync status of node(s) periodically before its use. `node_sync_disabled==True` means library also uses nodes that _are not_ in sync with network. This parameter is usually useful if one would like to interact with local test node that is not fully synced. This parameter should not be used in production

If `node_pool_urls` is provided then the library periodically checks in some interval (argument `node_sync_interval`) whether node is in sync or not.

## Generating seed and addresses

Since the IOTA network is permission-less type of network, anybody is able to use it and interact with it. No central authority is required at any stage. So anybody is able to generate own `seed` and then deterministically generate respective private keys/addresses.

> Please note, it is highly recommended to NOT use online seed generators at all. The seed is the only key to the given addresses. Anyone who owns the seed owns also all funds related to respective IOTA addresses (all of them).

> We strongly recommend to use official `wallet.rs` library together with `stronghold.rs` enclave for value-based transfers. This combination incorporates the best security practices while dealing with seeds, related addresses and `UTXO`. See more information on [Chrysalis docs](https://chrysalis.docs.iota.org/libraries/wallet.html).

IOTA 1.5 (Chrysalis) uses `Ed25519` signature scheme and address is usually represented by Bech32 (checksummed base32) format string of 64 characters.

A root of `Ed25519` signature scheme is basically a `32-byte (256-bit)` uniformly randomly generated seed based on which all private keys and corresponding addresses are generated. In the examples below, the seed is represented by a string of 64 characters using `[0-9a-f]` alphabet (32 bytes encoded in hexadecimal).

Seed can be for example generated using SHA256 algorithm on some random input generated by cryptographically secure pseudo-random generator, such as `os.urandom()`:
```python
{{#include ../../../bindings/python/examples/02_generate_seed.py}}
```

Seed examples (a single seed per line):
```plaintext
4892e2265c45734d07f220294b1697244a8ab5beb38ba9a7d57aeebf36b6e84a
37c4aab22a5883595dbc77907c1626c1be39d104df39c5d5708423c0286aea89
e94346bce41402155ef120e2525fad2d0bf30b10a89e4b93fd8471df1e6a0981
...
```

> In modern wallet implementations, such as our [wallet.rs library](https://chrysalis.docs.iota.org/libraries/wallet.html) and [firefly wallet](https://blog.iota.org/firefly-beta-release/), the seed is usually generated from a `seed mnemonic` (`seed phrase`), using [BIP39 standard](https://en.bitcoin.it/wiki/BIP_0039), to be better memorized/stored by humans. It is based on randomly generated list of english words and later used to generate the seed. Either way, the seed is a root for all generated private keys and addresses

### Address/key space
Before an actual address generation process, let's quickly focus on [BIP32](https://github.com/bitcoin/bips/blob/master/bip-0032.mediawiki) standard that describes an approach to _Hierarchical Deterministic Wallets_. The standard was improved by [BIP44](https://github.com/bitcoin/bips/blob/master/bip-0044.mediawiki) lately.

These standards define a tree structure as a base for address and key space generation which is represented by a `derivation path`:
```plaintext
m / purpose / coin_type / account / change / address_index
```
* `m`: a master node (seed)
* `purpose`: constant which is {44}
* `coin_type`: a constant set for each crypto currency. IOTA = 4218, for instance.
* `account`: account index. Zero-based increasing `int`. This level splits the address/key space into independent branches (ex. user identities) which each has own set of addresses/keys
* `change`: change index which is `{0, 1}`, also known as `wallet chain`.<br />
There are two independent chain of addresses/keys. `0` is reserved for public addresses (for coin receival) and `1` is reserved for internal (also known as change) addresses to which transaction change is returned. _In comparison to IOTA 1.0, IOTA 1.5 is totally fine with address reuse, and so it is, technically speaking, totally valid to return transaction change to the same originating address. So it is up to developers whether to leverage it or not. `iota.rs` library and its sibling `wallet.rs` help with either scenario_
* `address_index`: address index. Zero-based increasing `int` that indicates an address index

As outlined, there is a quite large address/key space that is secured by a single unique seed.

And there are few additional interesting notes:
* Each level defines a completely different subtree (subspace) of addresses/keys and those are never mixed up
* The hierarchy is ready to "absorb" addresses/keys for many different coins at the same time (`coin_type`), and all those coins are secured by the same seed.<br />(So basically any BIP32/44-compliant wallet is potentially able to manage any BIP32/44-compliant coin(s))
* There may be also other `purposes` in the future however let's consider a single purpose as of now. The constant `44` stands for BIP44
* The standard was agreed upon different crypto communities, although not all `derivation path` components are always in active use. For example, `account` is not always actively leveraged across crypto space (if this is a case then there is usually `account=0` used)
* Using different `accounts` may be useful to split addresses/key into some independent spaces and it is up to developers to implement.<br />
_Please note, it may have a negative impact on a performance while [account discovery](https://github.com/bitcoin/bips/blob/master/bip-0044.mediawiki#account-discovery) phase. So if you are after using many multiple accounts then you may be interested in our stateful library [wallet.rs](https://chrysalis.docs.iota.org/libraries/wallet.html) that incorporates all business logic needed to efficiently manage independent accounts. Also our [exchange guide](https://chrysalis.docs.iota.org/guides/exchange_guide.html) provides some useful tips how different accounts may be leveraged_

![address_generation](address_generation.svg)

So in case of IOTA 1.5 (Chrysalis), the derivation path of address/key space is `[seed]/44/4218/{int}/{0,1}/{int}`. The levels `purpose` and `coin_type` are given, the rest levels are up to developers to integrate.

### Generating address(es)

IOTA addresses are generated via `Client.get_addresses()` function that returns a list of tuples with generated addresses. Considering the previous chapter about individual address/key spaces, it becomes quite clear what all used input function arguments are for.

_Please note: for the examples outlined below, an example seed `b3d7092195c36d47133ff786d4b0a1ef2ee6a0052f6e87b6dc337935c70c531e` was used via environment variable called `IOTA_SEED_SECRET`. This seed serves for training purposes only._

The whole process is deterministic which means the output is the same as long as the seed is the same:

```python
{{#include ../../../bindings/python/examples/03_generate_addresses.py}}
```

Output example:
```json
[('atoi1qp9427varyc05py79ajku89xarfgkj74tpel5egr9y7xu3wpfc4lkpx0l86', False),
 ('atoi1qzfvkkp398v7hhvu89fu88hxctf7snwc9sf3a3nd7msfv77jk7qk2ah07s3', True),
 ('atoi1qq4t98j5y8wxkaujue99mjwqcp6jvvmsd5lv0755sz7dtjdz3p2lydv76sy', False),
 ('atoi1qrhzhjxc4z8vpwjt3hafs5xpdng5katqe890p0h95mc0l273j8yzxn7r4hc', True),
 ('atoi1qputu0yvfvxd7g39wf4rc67e0f0dyhl6enxu9jxnsrjqmemh067tw7qelyc', False),
 ('atoi1qptg5w2x47qwjf3gpqt3h7d2ey5x7xf8v7qtt29gkxt4mjfjfc28sutvd8a', True),
 ('atoi1qprvelq9paakh72fgm6j2kf8kexadw3t5xljer9dpsep5c7wx5mjwdxch6z', False),
 ('atoi1qrwk37tz47ddng9kpxfflkpz5tplcq7ll56v4acam04307xk70l7uf6wg8j', True),
 ('atoi1qper3zr5xe9x0wqs35ytwh622870g44frkyygdhs0ds8yejle3xujhq7dx3', False),
 ('atoi1qq6lkr9hucfylqjaqphu0stvk8pcmsx98r7ukuq40asszwmqytlnc058thk', True),
 ('atoi1qzpn7se3ryhscmqg404pycxzvfpt8v4xn8aul0tqdh00xsncgnxu7na7zjj', False),
 ('atoi1qz4qqakty9qytw8fk9shelt9lwlvv83s5ggt3wjag9fkgcc74z78w4l86y5', True),
 ('atoi1qp20uddchglqry0l5qnjg5aln8d5rk2v5l45hwrxv9z0daxs7u6xcsh4077', False),
 ('atoi1qrlqm2u5txxxnjx22fxq0jfjzk6l4nwnue6ht5pepk65m2f4xmxqynmxu2m', True),
 ('atoi1qqydc70mpjdvl8l2wyseaseqwzhmedzzxrn4l9g2c8wdcsmhldz0ulwjxpz', False),
 ('atoi1qrkjennxyl2xcqem6x69ya65sasma33z0ux872k846lqft0s3qf7k6lqpft', True),
 ('atoi1qr4yuekp30ff7mnnnjwy9tdhynxmlmkpuxf70qurtwudp2zpf3jeyw4uh37', False),
 ('atoi1qp6m5sz5ayjtccfxapdk5lp4qkheyfg0emzntmulyxzftps730vcul8dmqr', True),
 ('atoi1qzrwhkzhu67fqltfffwljejawdcghedukpgu9x6tzevwlnq89gmfjtayhgz', False),
 ('atoi1qpehxcp24z947dgupjqc9ktkn5ylmdxqqnx83m7xlajnf8005756u4n7z77', True)]
```
* Each tuple contains `address` and `bool` value indicating the given address is a `change` address or not.<br />
`True` means the given address is a change address (internal). So basically we've got two independent sets of addresses (10 items per each)
* This behavior is controlled via `get_all` argument. `get_all=False` (default) means to generate only public addresses

IOTA address is represented by a checksumed base 32 string (Bech32) and you can see a detailed explanation on [Chrysalis docs](https://chrysalis.docs.iota.org/guides/index.html#iota-15-address-anatomy).
Just a recap:
* If an address starts with `atoi` then it means it is related to `testnet`. `iota` stands for mainnet
* Number `1` at 5<sup>th</sup> position is just a separator
* The last 6 characters are reserved for a checksum

To quickly validate any IOTA address, there is a convenience function `Client.is_address_valid()` that returns `bool` value. Needless to say, performing a sanity check of an address before its use is an advisable practice.

## Checking a balance
_In Chrysalis testnet, there is a faucet service that provides test tokens to any testnet address: https://faucet.testnet.chrysalis2.com/_

There are three common api calls that can be leveraged:
* `Client.get_address_balance(address: str)`: it expects a single address in Bech32 format and returns `dict` with a balance for the address
* `Client.get_address_balances(list[str])`: a convenience function that expects `list` of addresses in Bech32 format and returns list of `dict` with balances for all given addresses
* `Client.get_balance(seed, account_index (optional), initial_address_index(optional), gap_limit(optional))`: a convenience function that combines `Client.get_addresses()` and `Client.get_address_balances()` api calls. It returns a combined balance for the provided seed and its wallet account index

_Please note: `Client.get_address_balance()` and `Client.get_address_balances()` return address(es) in hex-encoded Ed25519 address format, which is the format returned by underlying node software:_

```python
{{#include ../../../bindings/python/examples/04_get_balance.py}}
```

Example of output:
```json
Return balance for a single address:
{
    'address_type': 0,
    'address': '4b55799d1930fa049e2f656e1ca6e8d28b4bd55873fa6503293c6e45c14e2bfb',
    'balance': 10000000
}

Return balance for the given seed and account_index:
10000000
```
* `address_type` indicates type of address. Value 0 denotes a Ed25519 address (currently the default for IOTA 1.5 network)

`Client.get_balance()` performs a several tasks under the hood.
It starts generating addresses for the provided `seed` and `account_index` from `initial_address_index`, and checks for a balance of each of the generated addresses. Since it does not know how many addresses are used in fact, there is a condition set by `gap_limit` argument when to stop searching. If `gap_limit` amount of addresses in a row have no balance the function returns result and searching does not continue.

## Messages, payload and transactions
Before we continue, let's introduce some additional terms that describe an unit that is actually broadcasted in IOTA 1.5 network.

In comparison to original IOTA 1.0, IOTA 1.5 introduced some fundamental changes to the underlying data structure. The original concept of `transactions` and `bundles` is gone, and has been replaced by a concept of `messages` and `payloads`.

`Message` is a data structure that is actually being broadcasted in IOTA network and represent a node (vertex) in the Tangle graph. It can refer to up to 8 previous messages and once a message was attached to the Tangle and approved by a milestone, the Tangle structure ensures the content of the message is unaltered. Every message is referenced by `message_id` which is based on a hash algorithm of binary content of the message. `Message` is an atomic unit that is confirmed by network as a whole.

> IOTA is no longer based on ternary. IOTA 1.5 (Chrysalis) uses binary to encode and broadcast all underlying data entities

`Message` is broadcasted using a binary format, is arbitrary size (up to 35 kB) and it can hold a variable sets of information so called `payloads`. Number of payloads a single message can encapsulate is not given (even a message without any `payload` at all is completely valid).

`Payload` represents a layer of concern. Some payloads may change a state of the ledger (ex. `transactions`) and some may provide extra features to some specific applications and business use cases (ex. `indexed data`).

There are already implemented core payloads, such as `SignedTransaction`, `MilestonePayload` and `IndexationPayload` but the message and payload definition is generic enough to incorporate any future payload(s) the community agrees upon.

Needless to say, IOTA network ensures the outer structure of message itself is valid and definitely aligned with a network consensus protocol, however the inner structure is very flexible, future-proof, and offer an unmatched network extensibility.

![messages_in_tangle](messages_in_tangle.svg)

The current IOTA 1.5 network incorporates the following core payloads:
* `SignedTransaction`: payload that describes `UTXO` transactions that are cornerstone of value-based transfers in IOTA network. Via this payload, `message` can be also cryptographically signed
* `MilestonePayload`: payload that is emitted by Coordinator
* `IndexationPayload`: payload that enables addition of an index to the encapsulating message, as well as some arbitrary data. The given index can be later used to search the message(s)

### Unspent Transaction Output (UTXO)
Originally, the IOTA used an `account-based model` for tracking individual iota tokens: _each IOTA address holds a number of tokens and aggregated number of tokens from all iota addresses is equal to total supply._

In contrary, IOTA 1.5 uses `unspent transaction output` model, so called `UTXO`. It is based on an idea to track unspent amount of tokens via data structure called `output`.

Simplified analogy:
* There is 100 tokens recorded in the ledger as `Output A` and this output belongs to Alice. So **initial state of ledger**: `Output A` = 100 tokens
* Alice sends 20 tokens to Paul, 30 tokens to Linda and keeps 50 tokens at her disposal
* Her 100 tokens are recorded as `Output A` and so she has to divide (spent) tokens and create three new outputs:<br />`Output B` with 20 tokens that goes to Paul, `Output C` with 30 tokens that goes to Linda and finally `Output D` with the rest 50 tokens that she keep for herself
* **Original `Output A`** was completely spent and can't be used any more. It has been spent and so **becomes irrelevant** to ledger state
* **New state of ledger**: `Output B` = 20 tokens, `Output C` = 30 tokens and `Output D` = 50 tokens
* Total supply remains the same. Just number of outputs differs and some outputs were replaced by other outputs in the process

![utxo](utxo.svg)

The key takeaway of the outlined process is the fact that each unique `output` can be spent **only once**. Once the given `output` is spent, can't be used any more and is irrelevant in regards to the ledger state.

So even if Alice still wants to keep remaining tokens at her fingertips, those tokens have to be moved to completely new `output` that can be for instance still tight to the same Alice's iota address as before.

Every `output` stores also information about an IOTA address to which it is coupled with. So addresses and tokens are indirectly coupled via `outputs`.
So basically sum of outputs and their amounts under the given address is a balance of the given address, ie. the number of tokens the given address can spend. And sum of all unspent outputs and theirs amounts is equal to the total supply.

Before the chapter is wrapped up, one thing was left unexplained: _"how outputs are being sent and broadcasted to network?"_ `Outputs` are being sent encapsulated in a `message` as a part of `SignedTransaction` payload.

## Outputs
There are three functions to get `UTXO` outputs (related to the given address):
* `Client.get_address_outputs(str)`: it expects address in Bech32 format and returns `list[dict]` of `transaction_ids` and respective `indexes`
* `Client.get_output(str)`: it expects `output_id` and returns the UTXO output associated with it
* `Client.find_outputs(output_ids (optional), addresses (optional))`: it is a bit more general and it searches for `UTXO` outputs associated with the given `output_ids` and/or `addresses`

`Client.get_address_outputs(str)` returns `transaction_ids` and `indexes` in a raw form (in bytes) defined on protocol level and so usually some quick conversion is needed:
```python
{{#include ../../../bindings/python/examples/05a_get_address_outputs.py}}
```

Output example:
```plaintext
Output index: 0; raw transaction id: [162, 44, 186, 6, 103, 201, 34, 203, 177, 248, 189, 202, 249, 112, 178, 168, 129, 204, 214, 232, 142, 47, 204, 229, 3, 116, 222, 42, 172, 124, 55, 114]
`output_id` encoded in hex: a22cba0667c922cbb1f8bdcaf970b2a881ccd6e88e2fcce50374de2aac7c37720000
```
* as a result, `UTXO` output is represented by output `index` and `transaction_id`. `transaction_id` is basically a list of 32 `bytes`. `index` is 2-bytes (16bits) `uint`
* `index` and `transaction_id` is usually combined into single hex string of 68 characters = 32 * 2 chars (`transaction_id`; 32 bytes in hex) + 4 chars (`index`; 2 bytes in hex).<br />
The resulting `output_id` is the unique id of the given `output`

Then the function `Client.get_output(str)` can be used to get details about the given `output_id`:
```python
{{#include ../../../bindings/python/examples/05b_get_output.py}}
```

Output example:
```json
{'message_id': 'f51fb2839e0a24d5b4a97f1f5721fdac0f1eeafd77645968927f7c2f4b46565b',
 'transaction_id': 'a22cba0667c922cbb1f8bdcaf970b2a881ccd6e88e2fcce50374de2aac7c3772',
 'output_index': 0,
 'is_spent': False,
 'output': {'treasury': None,
  'signature_locked_single': {'kind': 0,
   'address': {'ed25519': {'kind': 0,
     'address': '4b55799d1930fa049e2f656e1ca6e8d28b4bd55873fa6503293c6e45c14e2bfb'}},
   'amount': 10000000},
  'signature_locked_dust_allowance': None}
}
```

A function `Client.find_outputs()` is a convenient shortcut combining both mentioned methods in a single call:
```python
{{#include ../../../bindings/python/examples/05c_find_outputs.py}}
```
* it supports two arguments, a list of `output_ids` or a list of `addresses`

Output example:
```json
{'message_id': 'f51fb2839e0a24d5b4a97f1f5721fdac0f1eeafd77645968927f7c2f4b46565b',
 'transaction_id': 'a22cba0667c922cbb1f8bdcaf970b2a881ccd6e88e2fcce50374de2aac7c3772',
 'output_index': 0,
 'is_spent': False,
 'output': {'treasury': None,
  'signature_locked_single': {'kind': 0,
   'address': {'ed25519': {'kind': 0,
     'address': '4b55799d1930fa049e2f656e1ca6e8d28b4bd55873fa6503293c6e45c14e2bfb'}},
   'amount': 10000000},
  'signature_locked_dust_allowance': None}
}
```
* `message_id`: refer to the encapsulating message in which the transaction was sent
* `transaction_id`, `output_index`: refer to the given output within the `SignedTransaction` payload. There may be several different `outputs` involved in a single transaction and so just `transaction_id` is not enough
* `output`: this section provides details about the iota address to which the given `unspent transaction output` is coupled with
* `amount`: state an amount of tokens related to the `output`
* `is_spent`: of course, very important one indicating whether the given `output` is a part of the actual ledger state or not. As mentioned above, if an output was already spent, it is not part of ledger state any more and was replaced by some other `output(s)` in the process

So this is quite interesting part, notice the `output_id` that was used in a function call to get output details is the same as a combination of `transaction_id` and `output index`.

This way a transaction is tightly coupled with `outputs` since `SignedTransaction` payload is a main vehicle how `outputs` are being created and spent, and altogether everything is encapsulated in a `message`.

## Messages
As mentioned above, the `message` is encapsulating data structure that is being actually broadcasted across network. It is an atomic unit that is accepted/rejected as a whole.

There is a convenient function `Client.message()` that prepares a message instance and sends it over a network. It accepts wide range of input parameters and can help with any kind of message type to be broadcasted.

The simplest message that can be broadcasted is a message without any particular payload:

```python
{{#include ../../../bindings/python/examples/06_simple_message.py}}
```

Output example:
```json
{'message_id': 'e2daa4c6b012b615becd6c12189b2c9e701ba0d53b31a15425b21af5105fc086',
 'network_id': 7712883261355838377,
 'parents': ['0e2705ce50fec88f896663d4b7d562e74cbcfdd951ac482b1f03cfa5f27396d7',
  '0f5a0b2041766127c3f3bff2dd653b450b72e364765fcc805a40423c59ed01f9',
  '20635b30aee437575d7e6abdf6629eec80543bee30848b0abdda2200fc11a977',
  'da97cd6cfcbb854b8fd3f064c8459c5c9eae80dbd5ef594a3e1a26dcb8fc078c'],
 'payload': None,
 'nonce': 2305843009213869242}
```
* `message_id` is an unique id that refers to the given message in network
* as mentioned above, every message in the Tangle should refer to up to 8 other messages, those are indicated in the section `parents`
* no actual `payload` was given in this example message (`payload=None`)
* `nonce` refer to a result of proof-of-work

Once a message is broadcasted, there are two main functions that can be used to read all information about the given message from the Tangle (`Client.get_message_data()` and `Client.get_message_metadata()`):
```python
{{#include ../../../bindings/python/examples/07_get_message_data.py}}
```

Output example:
```json
Message meta data:
{'message_id': 'e2daa4c6b012b615becd6c12189b2c9e701ba0d53b31a15425b21af5105fc086',
 'parent_message_ids': ['0e2705ce50fec88f896663d4b7d562e74cbcfdd951ac482b1f03cfa5f27396d7',
  '0f5a0b2041766127c3f3bff2dd653b450b72e364765fcc805a40423c59ed01f9',
  '20635b30aee437575d7e6abdf6629eec80543bee30848b0abdda2200fc11a977',
  'da97cd6cfcbb854b8fd3f064c8459c5c9eae80dbd5ef594a3e1a26dcb8fc078c'],
 'is_solid': True,
 'referenced_by_milestone_index': 284866,
 'milestone_index': None,
 'ledger_inclusion_state': {'state': 'NoTransaction'},
 'conflict_reason': None,
 'should_promote': None,
 'should_reattach': None}

Message data:
 {'message_id': 'e2daa4c6b012b615becd6c12189b2c9e701ba0d53b31a15425b21af5105fc086',
 'network_id': 7712883261355838377,
 'parents': ['0e2705ce50fec88f896663d4b7d562e74cbcfdd951ac482b1f03cfa5f27396d7',
  '0f5a0b2041766127c3f3bff2dd653b450b72e364765fcc805a40423c59ed01f9',
  '20635b30aee437575d7e6abdf6629eec80543bee30848b0abdda2200fc11a977',
  'da97cd6cfcbb854b8fd3f064c8459c5c9eae80dbd5ef594a3e1a26dcb8fc078c'],
 'payload': None,
 'nonce': 2305843009213869242}
```
* `Client.get_message_metadata` provides information how the given message fits to network structures such as `ledger_inclusion_state`, etc.
* `Client.get_message_data` provides all data that relates to the given message and its payload(s)

### IndexationPayload
`IndexationPayload` is a payload type that can be used to attach an arbitrary `data` and key `index` to a message. At least `index` should be provided in order to send the given payload. Data part (as `list[bytes]`) is optional one:

```python
{{#include ../../../bindings/python/examples/08_data_message.py}}
```

Output example:
```json
{'message_id': '8d4fa37be3c00691131c2c3e03e7b8b956c9118a2ce4be3a8597d51d82ed2de9',
 'network_id': 7712883261355838377,
 'parents': ['3719d308ae14b7ef1ed5a3a1604228e97587b9da487db10bc6e4a4f800083da0',
  '4431e2f776db888488728e0aa34c94975e65d6fa74893aa675172af6b9f37257',
  '8f9fa84954c58bcfc9acc33ca827b4ea35c2caae88db736399a031120e85eebf',
  'f63d416de97e6a9fd1314fbbbbb263f30dff260f3075f9a65e7dfe1f2cc56ce3'],
 'payload': {'transaction': None,
  'milestone': None,
  'indexation': [{'index': '736f6d655f646174615f696e646578',
    'data': [115,
     111,
     109,
     101,
     32,
     117,
     116,
     102,
     32,
     98,
     97,
     115,
     101,
     100,
     32,
     100,
     97,
     116,
     97]}],
  'receipt': None,
  'treasury_transaction': None},
 'nonce': 6917529027641573188}
```
* Feel free to check the given message using its `message_id` via [Tangle explorer](https://explorer.iota.org/chrysalis/message/8d4fa37be3c00691131c2c3e03e7b8b956c9118a2ce4be3a8597d51d82ed2de9)
* In comparison to an empty message sent in the previous chapter, the `payload` section looks more interesting
* There are three payloads prepared (`transaction`, `milestone` and `indexation`) however only `indexation` payload is leveraged this time
* `index` was simply encoded to `list[bytes]` in hex (no hash algorithm) and the resulting string can be leveraged as an additional way how to search for a set of indexed messages with the same key index via [Tangle explorer](https://explorer.iota.org/chrysalis/indexed/736f6d655f646174615f696e646578) or `Client.find_messages()` API call
* `data` contains an arbitrary data encoded in bytes
* In comparison to IOTA 1.0, please note there is no IOTA address involved while sending data messages via network in case of IOTA 1.5. Such messages are referenced using `message_id` or key `index`
* IOTA addresses are part of `UTXO` data structure that is sent using `SignedTransaction` payload explained below

### SignedTransaction
`SignedTransaction` is a payload type that is used to transfer value-based messages as `UTXO` (Unspent Transaction Output).

As mentioned above, this core payload changes the ledger state as old `outputs` are being spent (replaced) and new `outputs` are being created:

```python
import iota_client
client = iota_client.Client()

client.get_message_data("f51fb2839e0a24d5b4a97f1f5721fdac0f1eeafd77645968927f7c2f4b46565b")
```

Example of a message with `SignedTransaction` payload:
```json
{
    'message_id': 'f51fb2839e0a24d5b4a97f1f5721fdac0f1eeafd77645968927f7c2f4b46565b',
    'network_id': 7712883261355838377,
    'parents': [
        '4a84bf1d345a441cfdefd0e71d6efe820c1077e5dda9122a09cbf026132d208c',
        '6e9153884fd1983be4c27c3ccdc69760b4775484eea498ec0707c2ff8901995e',
        '7ac1407c88007a54d603400b558d5110f2bbf93a68100fb34f0b40cece9d0868',
        '9ac0fd457998a1b3ddab9c0014f41344475358ad36c64a4b763de3b51f47c09a'
    ],
    'payload': {
        'transaction': [
            {
                'essence': {
                    'inputs': [
                        {'transaction_id': '4a34274992474d91cf45366425ad1d4df6042cba64f3b6c07d297a2e6b7154a9', 'index': 0}
                    ],
                    'outputs': [
                        {'address': 'atoi1qp9427varyc05py79ajku89xarfgkj74tpel5egr9y7xu3wpfc4lkpx0l86', 'amount': 10000000},
                        {'address': 'atoi1qzdnav0zdgd4grn25cnwcuudtahvlhgh0r349ur749y9l03vadrfurhkxwj', 'amount': 100016136757200}
                    ],
                    'payload': {
                        'transaction': None,
                        'milestone': None,
                        'indexation': [
                            {'index': '54414e474c454b495420464155434554', 'data': []}
                        ],
                        'receipt': None,
                        'treasury_transaction': None
                    }
                },
                'unlock_blocks': [
                    {
                        'signature': {
                            'public_key': [...],
                            'signature': [...]
                        },
                        'reference': None
                    }
                ]
            }
        ],
        'milestone': None,
        'indexation': None,
        'receipt': None,
        'treasury_transaction': None
    },
    'nonce': 1146102
}
```

Each `transaction` includes the following set of information:
* `inputs`: list of valid `outputs` that should be used to fund the given message. Those `outputs` will be spent and once the message is confirmed, those outputs are not valid anymore. Outputs are uniquely referenced via `transaction_id` and inner `index`. At least one output has to be given with enough balance to source all `outputs` of the given message
* `outputs`: list of IOTA address(es) and related amount(s) the input `outputs` should be split among. Based on this information, new `UTXO` entities (outputs) are being created
* `unlock_blocks`: it includes a transaction signature(s) (currently based on `Ed25519` scheme) that proofs token ownership based on a valid seed. Needless to say, only valid seed owner is able to correctly sign the given transaction and proofs the ownership of tokens under the given output(s). Each input `output` has to have a corresponding `unblock_block` entry in case more `outputs` are used to fund the operation either using the given signature or as a reference to existing signature
* `payload`: each `SignedTransaction` can include additional payload(s) such as `IndexationPayload`, etc. Meaning, any value-based messages can also contain arbitrary data and its key index. It is also an example how individual payloads can be encapsulated on different levels of concern

Sending value-based messages is also very straightforward process.

As a minimum, it needs a valid seed, output addresses and amount. The method finds valid output(s) that can be used to fund the given amount(s) and the unspent amount is sent to the same address:

```python
{{#include ../../../bindings/python/examples/09_transaction.py}}
```

Output example:
```json
{
    'message_id': '7c47db1c4555348c260d91e90cc10fd66c2e73a84ec24bf9533e440f6d945d42',
    'network_id': 7712883261355838377,
    'parents': [
        '0ec0cd3c0303845980981bf7cc72371a8cd6e38c15924a2950fb15c5ecf4a53b',
        '4011f7724f96b6e39cdf9987ee650c0552d4fc63c09dd72b9be30a3cc7b53806',
        '5730d5bd607c6125130df30204c995db5edcbd16c4ab150946dffac37ace26f9',
        '8c1982682dbfa0abdd8772e38d044dbfcbea5ebb99bbe7174c07d81adda62419'
    ],
    'payload': {
        'transaction': [
            {
                'essence': {
                    'inputs': [
                        {'transaction_id': 'a22cba0667c922cbb1f8bdcaf970b2a881ccd6e88e2fcce50374de2aac7c3772', 'index': 0}
                    ],
                    'outputs': [
                        {'address': 'atoi1qqydc70mpjdvl8l2wyseaseqwzhmedzzxrn4l9g2c8wdcsmhldz0ulwjxpz', 'amount': 1000000},
                        {'address': 'atoi1qp9427varyc05py79ajku89xarfgkj74tpel5egr9y7xu3wpfc4lkpx0l86', 'amount': 9000000}
                    ],
                    'payload': None
                },
                'unlock_blocks': [
                    {'signature': {
                        'public_key': [
                        243,...<trimmed>
                        ],
                        'signature': [
                                64,...<trimmed>
                            ]
                        },
                        'reference': None
                    }
                ]
            }
        ],
        'milestone': None,
        'indexation': None,
        'receipt': None,
        'treasury_transaction': None
    },
    'nonce': 9223372036854802939
}
```

> We recommend to use official `wallet.rs` library together with `stronghold.rs` enclave for value-based transfers. This combination incorporates the best security practices while dealing with seeds, related addresses and `UTXO`. See more information on [Chrysalis docs](https://chrysalis.docs.iota.org/libraries/wallet.html).

#### Dust protection
Please note, there is also implemented a [dust protection](https://chrysalis.docs.iota.org/guides/dev_guide.html#dust-protection) mechanism in the network protocol to avoid malicious actors to spam network in order to decrease node performance while keeping track of unspent amount (`UTXO`):
> "... microtransaction below 1Mi of IOTA tokens [can be sent] to another address if there is already at least 1Mi on that address"
That's why we did send 1Mi in the given example to comply with the protection."
