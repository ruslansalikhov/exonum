import Vue from 'vue/dist/vue'
import axios from 'axios'
import MockAdapter from 'axios-mock-adapter'
import * as Blockchain from '../src/plugins/blockchain.js'
import actual from './data/actual.json'
import proof from './data/proof.json'

const mock = new MockAdapter(axios)
const bigIntRegex = /[0-9]+/i;
const hexRegex = /[0-9A-Fa-f]+/i;
const TRANSACTION_URL = '/api/explorer/v1/transactions'
const TRANSACTION_EXPLORER_URL = '/api/explorer/v1/transactions?hash='
const PROOF_URL = '/api/services/cryptocurrency/v1/wallets/info?name='
const name = "John Doe"
const keyPair = {
  publicKey: '33ddfc87b274213ab42845d91570af26fb0cf3d28c147d08e44e96501b78cff5',
  secretKey: '888398232761ee1cf5bdff3bf306d9951d7b3f535f2d78edff4fb7d4e8a78e2833ddfc87b274213ab42845d91570af26fb0cf3d28c147d08e44e96501b78cff5'
}


Vue.use(Blockchain)

// Mock `createWallet` transaction
const createWalletTxHash = 'f1a4670f1895b803499fff9a6bf707353a373ef08b74e1631bef7f780b0fbd8d'
mock.onPost(TRANSACTION_URL, {
  'tx_body': '""33ddfc87b274213ab42845d91570af26fb0cf3d28c147d08e44e96501b78cff50000800002000a084a6f686e20446f6512220a2033ddfc87b274213ab42845d91570af26fb0cf3d28c147d08e44e96501b78cff518015fbd5b0acb9e9fe8637a6181134c6ead196774eb0289bb3a6b3ca16fd1bbf1206e19ed807e84aecfdea2cff815ab66dd39619a6b87e3dfd9f4991105848a2c0c""'
}).replyOnce(200)

mock.onGet(`${TRANSACTION_EXPLORER_URL}${createWalletTxHash}`).replyOnce(200, { 'type': 'in-pool' })

mock.onGet(`${TRANSACTION_EXPLORER_URL}${createWalletTxHash}`).replyOnce(200, { 'type': 'committed' })

// Mock `addFunds` transaction
const addFundsTxHash = 'b26f1e9e01a6f7f07d6224597992bb04fd5a4bd633faf0a28c384fa2b99ba322'
mock.onPost(TRANSACTION_URL, {
  'tx_body': '78cf8b5e5c020696319eb32a1408e6c65e7d97733d34528fbdce08438a0243e800008000010008321080d0b6db99b1c3f1890106ecdedffe9d00b6c1911e7a75f8c0fea17554f31497c914686bc63ad175cabfb02eaa40230573bb1ff1c4d98cd996c9c7c0eb54843f306d03ae4bf24aa72408'
}).replyOnce(200)

mock.onGet(`${TRANSACTION_EXPLORER_URL}${addFundsTxHash}`).replyOnce(200, { 'type': 'committed' })

// Mock `transfer` transaction
const transferTxHash = '85e2c97aab7d2b6518850b3c9f647b1bb2fa7f8370f33c6f9b6c11cfa6371969'
mock.onPost(TRANSACTION_URL, {
  'tx_body': '78cf8b5e5c020696319eb32a1408e6c65e7d97733d34528fbdce08438a0243e80000800000000a220a20278663010ebe1136011618ad5be1b9d6f51edc5b6c6b51b5450ffc72f54a57df10191880a0db99c6b080bc6ba0bfeb12fc750df184136bd8d9a4f33676b8ee6e1e40754d7d19f0cb4f62db67e36e83253e737dce0ec3a6566857ef71de440d329fd470e77fed232d2411590c'
}).replyOnce(200)

mock.onGet(`${TRANSACTION_EXPLORER_URL}${transferTxHash}`).replyOnce(200, { 'type': 'committed' })

// Mock proof
mock.onGet('/api/services/configuration/v1/configs/actual').reply(200, actual)

mock.onGet(`${PROOF_URL}${name}`).replyOnce(200, proof)

describe('Interaction with blockchain', () => {
  it('should generate new signing key pair', () => {
    const keyPair = Vue.prototype.$blockchain.generateKeyPair()

    expect(keyPair.publicKey).toMatch(hexRegex)
    expect(keyPair.publicKey).toHaveLength(64)
    expect(keyPair.secretKey).toMatch(hexRegex)
    expect(keyPair.secretKey).toHaveLength(128)
  })

  it('should generate new random seed', () => {
    const seed = Vue.prototype.$blockchain.generateSeed()

    expect(seed).toMatch(bigIntRegex)
  })

  it('should create new wallet', async () => {
    await expect(Vue.prototype.$blockchain.createWallet(keyPair, name, [keyPair.publicKey], 1)).resolves
  })

  it('should add funds', async () => {
    const amountToAdd = '50'
    const seed = '9935800087578782468'

    await expect(Vue.prototype.$blockchain.addFunds(keyPair, name, amountToAdd, seed)).resolves
  })

  it('should transfer funds', async () => {
    const receiver = 'Bob'
    const amountToTransfer = '25'
    const seed = '7743941227375415562'

    await expect(Vue.prototype.$blockchain.transfer(keyPair, name, receiver, amountToTransfer, seed)).resolves
  })

  it('should get wallet proof and verify it', async () => {
    const data = await Vue.prototype.$blockchain.getWallet(name);

    expect(data.wallet).toEqual({
      "name": "John Doe",
      "pub_keys": [
        {
          "data": [51,221,252,135,178,116,33,58,180,40,69,217,21,112,175,38,251,12,243,210,140,20,125,8,228,78,150,80,27,120,207,245]
        }],
      "quorum": 1,
      "balance": 100,
      "history_len": 1,
      "history_hash": {
        "data": [241,164,103,15,24,149,184,3,73,159,255,154,107,247,7,53,58,55,62,240,139,116,225,99,27,239,127,120,11,15,189,141]
      }
    })
  })
})
