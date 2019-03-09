<template>
  <div>
    <div class="container">
      <div class="row justify-content-sm-center">
        <div class="col-md-6 col-md-offset-3">
          <h1 class="mt-5 mb-4">Authorization</h1>
          <tabs>
            <tab :is-active="true" title="Register">
              <form @submit.prevent="register">
                <div class="form-group">
                  <label class="control-label">Name:</label>
                  <input v-model="name" type="text" class="form-control" placeholder="Enter name" maxlength="260" required>
                </div>
                <div class="form-group">
                  <label class="control-label">Keys:</label>
                  <input v-model="keys_num" type="number" class="form-control" placeholder="Enter keys number" min="1" required>
                </div>
                <div class="form-group">
                  <label class="control-label">Quorum:</label>
                  <input v-model="quorum" type="number" class="form-control" placeholder="Enter quorum" min="1" required>
                </div>
                <button type="submit" class="btn btn-lg btn-block btn-primary">Register</button>
              </form>
            </tab>
            <tab title="Log in">
              <form @submit.prevent="login">
                <div class="form-group">
                  <label class="control-label">Name:</label>
                  <input v-model="name" type="text" class="form-control" placeholder="Enter name" required>
                  <label class="control-label">Secret key:</label>
                  <input v-model="secretKey" type="text" class="form-control" placeholder="Enter secret key" required>
                </div>
                <button type="submit" class="btn btn-lg btn-block btn-primary">Log in</button>
              </form>
            </tab>
          </tabs>
        </div>
      </div>
    </div>

    <modal :visible="isModalVisible" title="Wallet has been created" action-btn="Log in" @close="closeModal" @submit="proceed">
      <div class="alert alert-warning" role="alert">Save the secret keys in a safe place. You will need it to log in to the demo next time.</div>
      <div class="form-group">
        <label>Secret keys:</label>
        <ul class="list-group list-group-flush">
          <!-- eslint-disable-next-line vue/require-v-for-key -->
          <li v-for="k in keyPairs" class="list-group-item">
            <div><code>{{ k.secretKey }}</code></div>
          </li>
        </ul>
      </div>
    </modal>

    <spinner :visible="isSpinnerVisible"/>
  </div>
</template>

<script>
  import Tab from '../components/Tab.vue'
  import Tabs from '../components/Tabs.vue'
  import Modal from '../components/Modal.vue'
  import Spinner from '../components/Spinner.vue'

  module.exports = {
    components: {
      Tab,
      Tabs,
      Modal,
      Spinner
    },
    data() {
      return {
        name: '',
        secretKey: '',
        keyPair: {},
        isModalVisible: false,
        isSpinnerVisible: false
      }
    },
    methods: {
      login() {
        if (!this.$validateHex(this.secretKey, 64)) {
          return this.$notify('error', 'Invalid secret key is passed')
        }

        this.isSpinnerVisible = true

        this.$store.commit('login', {
            name: this.name,
            keyPair: {
                publicKey: this.secretKey.substr(64),
                secretKey: this.secretKey
            }
        })

        this.$nextTick(function() {
          this.$router.push({ name: 'user' })
        })
      },

      async register() {
        if (!this.name) {
          return this.$notify('error', 'The name is a required field')
        }

        if (this.keys_num < 1) {
          return this.$notify('error', 'Keys number should be >= 1')
        }

        if (this.keys_num < this.quorum || this.quorum < 1) {
          return this.$notify('error', 'Quorum should be <= keys number and > 0')
        }

        this.isSpinnerVisible = true
        this.keyPairs = []
        let pubKeys = []
        for (let i = 0; i < this.keys_num; i++) {
          let pair = this.$blockchain.generateKeyPair();
          this.keyPairs.push(pair);
          pubKeys.push(pair.publicKey);
        }
        this.keyPair = this.keyPairs[0]

        try {
          await this.$blockchain.createWallet(this.keyPair, this.name, pubKeys, this.quorum)
          // this.name = ''
          this.isSpinnerVisible = false
          this.isModalVisible = true
        } catch (error) {
          this.isSpinnerVisible = false
          console.log(error.stack);
          this.$notify('error', error.toString())
        }
      },

      closeModal() {
        this.isModalVisible = false
      },

      proceed() {
        this.isModalVisible = false

        this.$store.commit('login', {
          name: this.name,
          keyPair: this.keyPair
        })

        this.$nextTick(function() {
          this.$router.push({ name: 'user' })
        })
      }
    }
  }
</script>
