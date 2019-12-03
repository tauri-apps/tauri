<template>
  <q-page class="flex flex-center">
    <h1>{{ msg }}</h1>
    <q-btn @click="eventToRust()">SEND MSG</q-btn>
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
  created () {
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
