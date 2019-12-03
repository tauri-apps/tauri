<template>
  <q-page class="flex flex-center column">
    <h3 class="row text-center">{{ msg }}</h3>
    <div class="row">
      <q-btn @click="eventToRust()">SEND MSG</q-btn>
    </div>
  </q-page>
</template>

<script>
require('../../src-tauri/tauri.js')
import { uid } from 'quasar'

export default {
  name: 'HelloWorld',
  data () {
    return {
      msg: 'waiting for rust'
    }
  },
  mounted () {
    window.tauri.setup()
    window.tauri.listen('reply', res => {
      this.msg = res.payload.msg
    }, false)
  },
  methods: {
    // set up an event listener
    eventToRust () {
      window.tauri.emit('hello', uid())
    }
  }
}
</script>
