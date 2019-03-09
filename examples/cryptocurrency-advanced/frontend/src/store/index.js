import Vue from 'vue'
import Vuex from 'vuex'

//const KEY = 'cryptocurrency'
const NAME_KEY = 'name'
const KEYPAIR_KEY = 'key_pair'
const name = JSON.parse(localStorage.getItem(NAME_KEY))
const keyPair = JSON.parse(localStorage.getItem(KEYPAIR_KEY))

Vue.use(Vuex)

export default new Vuex.Store({
  state: {
    name: name,
    keyPair: keyPair
  },
  mutations: {
    login: (state, data) => {
      localStorage.setItem(NAME_KEY, JSON.stringify(data.name))
      localStorage.setItem(KEYPAIR_KEY, JSON.stringify(data.keyPair))
      state.name = data.name
      state.keyPair = data.keyPair
      console.log("D:", data)
    },
    logout: state => {
      localStorage.removeItem(NAME_KEY)
      localStorage.removeItem(KEYPAIR_KEY)
      state.keyPair = null
      state.name = null
    }
  }
})
